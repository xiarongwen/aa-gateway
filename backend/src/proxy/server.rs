//! 代理服务器实现
//!
//! 基于 Axum 的 HTTP 代理服务器，支持：
//! - 多 Provider 负载均衡
//! - 熔断器保护
//! - 请求格式转换 (OpenAI ↔ Anthropic)
//! - 流式响应

use axum::{
    body::Body,
    extract::{State, Path, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{post, get},
    Router,
};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tower_http::cors::{Any, CorsLayer};

use crate::database::Database;
use crate::models::provider::Provider;
use super::{
    types::*,
    forwarder::{forward_request, ProviderInfo, RequestContext},
    router::ProviderRouter,
    http_client::build_headers,
};

/// 代理服务器状态
pub struct ProxyServer {
    config: ProxyConfig,
    state: Arc<ProxyState>,
    shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
    server_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

/// 共享状态
pub struct ProxyState {
    pub db: Arc<Database>,
    pub config: ProxyConfig,
    pub provider_router: Arc<Mutex<ProviderRouter>>,
    pub status: Arc<Mutex<ProxyStatus>>,
}

impl ProxyServer {
    pub fn new(db: Arc<Database>, config: ProxyConfig) -> Self {
        let state = Arc::new(ProxyState {
            db: db.clone(),
            config: config.clone(),
            provider_router: Arc::new(Mutex::new(ProviderRouter::new())),
            status: Arc::new(Mutex::new(ProxyStatus::default())),
        });

        Self {
            config,
            state,
            shutdown_tx: Arc::new(Mutex::new(None)),
            server_handle: Arc::new(Mutex::new(None)),
        }
    }

    /// 启动代理服务器
    pub async fn start(&self) -> Result<ProxyServerInfo, ProxyError> {
        // 检查是否已在运行
        if self.shutdown_tx.lock().unwrap().is_some() {
            return Err(ProxyError::AlreadyRunning);
        }

        let addr: SocketAddr = format!("{}:{}",
            self.config.listen_address,
            self.config.listen_port
        ).parse().map_err(|e| ProxyError::BindFailed(format!("Invalid address: {}", e)))?;

        // 创建关闭通道
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        // 构建路由
        let app = self.build_router();

        // 绑定监听器
        let listener = tokio::net::TcpListener::bind(&addr).await
            .map_err(|e| ProxyError::BindFailed(e.to_string()))?;

        tracing::info!("[PROXY] 代理服务器启动于 {}", addr);

        // 保存关闭句柄
        *self.shutdown_tx.lock().unwrap() = Some(shutdown_tx);

        // 更新状态
        {
            let mut status = self.state.status.lock().unwrap();
            status.running = true;
            status.address = self.config.listen_address.clone();
            status.port = self.config.listen_port;
        }

        // 启动服务器
        let state = self.state.clone();
        let handle = tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    shutdown_rx.await.ok();
                })
                .await
                .ok();

            // 服务器停止后更新状态
            let mut status = state.status.lock().unwrap();
            status.running = false;
        });

        // 保存服务器任务句柄
        *self.server_handle.lock().unwrap() = Some(handle);

        Ok(ProxyServerInfo {
            address: self.config.listen_address.clone(),
            port: self.config.listen_port,
            started_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// 停止代理服务器
    pub async fn stop(&self) -> Result<(), ProxyError> {
        // 1. 发送关闭信号
        if let Some(tx) = self.shutdown_tx.lock().unwrap().take() {
            let _ = tx.send(());
        } else {
            return Err(ProxyError::NotRunning);
        }

        // 2. 等待服务器任务结束
        if let Some(handle) = self.server_handle.lock().unwrap().take() {
            match tokio::time::timeout(
                std::time::Duration::from_secs(5),
                handle
            ).await {
                Ok(Ok(())) => {
                    tracing::info!("[PROXY] 代理服务器已完全停止");
                    Ok(())
                }
                Ok(Err(e)) => {
                    tracing::warn!("[PROXY] 代理服务器任务异常终止: {}", e);
                    Err(ProxyError::StopFailed(e.to_string()))
                }
                Err(_) => {
                    tracing::warn!("[PROXY] 代理服务器停止超时");
                    Err(ProxyError::StopTimeout)
                }
            }
        } else {
            Ok(())
        }
    }

    /// 获取代理状态
    pub fn get_status(&self) -> ProxyStatus {
        self.state.status.lock().unwrap().clone()
    }

    /// 构建路由
    fn build_router(&self) -> Router {
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        let state = self.state.clone();

        Router::new()
            // 健康检查
            .route("/health", get(health_check))
            .route("/status", get(get_status_handler))
            // Claude API
            .route("/v1/messages", post(handle_claude_messages))
            .route("/claude/v1/messages", post(handle_claude_messages))
            // OpenAI API (Codex CLI)
            .route("/chat/completions", post(handle_openai_chat))
            .route("/v1/chat/completions", post(handle_openai_chat))
            .route("/v1/v1/chat/completions", post(handle_openai_chat))
            .route("/codex/v1/chat/completions", post(handle_openai_chat))
            // Gemini API
            .route("/v1beta/*path", post(handle_gemini))
            .route("/gemini/v1beta/*path", post(handle_gemini))
            .layer(cors)
            .with_state(state)
    }
}

/// 健康检查
async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

/// 获取状态
async fn get_status_handler(
    State(state): State<Arc<ProxyState>>,
) -> impl IntoResponse {
    let status = state.status.lock().unwrap().clone();
    axum::Json(status)
}

/// 处理 Claude 消息请求
async fn handle_claude_messages(
    State(state): State<Arc<ProxyState>>,
    request: Request,
) -> Response {
    let headers = request.headers().clone();
    let body = request.into_body();

    let context = RequestContext {
        provider_id: None, // 使用默认 Provider
        model: None,
        app_type: "claude".to_string(),
    };

    handle_proxy_request(state, body, headers, context).await
}

/// 处理 OpenAI 聊天请求
async fn handle_openai_chat(
    State(state): State<Arc<ProxyState>>,
    request: Request,
) -> Response {
    let headers = request.headers().clone();
    let body = request.into_body();

    let context = RequestContext {
        provider_id: None,
        model: None,
        app_type: "codex".to_string(),
    };

    handle_proxy_request(state, body, headers, context).await
}

/// 处理 Gemini 请求
async fn handle_gemini(
    State(state): State<Arc<ProxyState>>,
    Path(_path): Path<String>,
    request: Request,
) -> Response {
    let headers = request.headers().clone();
    let body = request.into_body();

    let context = RequestContext {
        provider_id: None,
        model: None,
        app_type: "gemini".to_string(),
    };

    handle_proxy_request(state, body, headers, context).await
}

/// 统一处理代理请求
async fn handle_proxy_request(
    state: Arc<ProxyState>,
    body: Body,
    headers: reqwest::header::HeaderMap,
    context: RequestContext,
) -> Response {
    // 从数据库加载 Providers
    let providers = match load_providers(&state.db).await {
        Ok(providers) => providers,
        Err(e) => {
            tracing::error!("Failed to load providers: {}", e);
            return error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to load providers");
        }
    };

    if providers.is_empty() {
        return error_response(StatusCode::SERVICE_UNAVAILABLE, "No providers configured");
    }

    // 转发请求
    match forward_request(
        body,
        headers,
        context,
        state.provider_router.clone(),
        providers,
    ).await {
        Ok(result) => {
            // 更新统计
            update_stats(&state, &result.provider_id, true).await;
            result.response
        }
        Err(err) => {
            if let Some(pid) = &err.provider_id {
                update_stats(&state, pid, false).await;
            }
            error_response(
                StatusCode::BAD_GATEWAY,
                &format!("Proxy error: {}", err.error)
            )
        }
    }
}

/// 从数据库加载 Providers
async fn load_providers(
    db: &Arc<Database>,
) -> anyhow::Result<Vec<ProviderInfo>> {
    let conn = db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, name, provider_type, base_url, api_key, is_default
         FROM providers WHERE is_default = TRUE"
    )?;

    let providers: Vec<ProviderInfo> = stmt
        .query_map([], |row| {
            Ok(ProviderInfo {
                id: row.get(0)?,
                name: row.get(1)?,
                provider_type: row.get(2)?,
                base_url: row.get(3)?,
                api_key: row.get(4)?,
                is_default: row.get(5)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    // 如果没有默认 Provider，加载所有 Provider
    if providers.is_empty() {
        let mut stmt = conn.prepare(
            "SELECT id, name, provider_type, base_url, api_key, is_default
             FROM providers"
        )?;

        let all_providers: Vec<ProviderInfo> = stmt
            .query_map([], |row| {
                Ok(ProviderInfo {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    provider_type: row.get(2)?,
                    base_url: row.get(3)?,
                    api_key: row.get(4)?,
                    is_default: row.get(5)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        return Ok(all_providers);
    }

    Ok(providers)
}

/// 更新统计信息
async fn update_stats(
    state: &Arc<ProxyState>,
    provider_id: &str,
    success: bool,
) {
    let mut status = state.status.lock().unwrap();
    status.total_requests += 1;
    if !success {
        // 可以在这里记录失败统计
    }
}

/// 错误响应
fn error_response(status: StatusCode, message: &str) -> Response {
    Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"error": "{}"}}"#,
            message.replace('"', "\\\"")
        )))
        .unwrap()
}
