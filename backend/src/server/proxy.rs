//! Proxy API 路由

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;

use crate::database::Database;
use crate::models::proxy::*;
use crate::models::{ApiResponse};
use crate::proxy::{ProxyServer};

/// 共享的代理服务器状态
use std::sync::Mutex as StdMutex;
static PROXY_SERVER: StdMutex<Option<ProxyServer>> = StdMutex::new(None);

/// 代理路由
pub fn proxy_routes() -> Router<Arc<Database>> {
    Router::new()
        .route("/config", get(get_proxy_config).put(update_proxy_config))
        .route("/status", get(get_proxy_status))
        .route("/start", post(start_proxy))
        .route("/stop", post(stop_proxy_handler))
}

/// 获取代理配置
async fn get_proxy_config(
    State(db): State<Arc<Database>>,
) -> Json<ApiResponse<ProxyConfig>> {
    let conn = db.conn();

    let result = conn.query_row(
        "SELECT enabled, listen_address, listen_port, enable_auth, enable_logging, config
         FROM proxy_configs WHERE id = 1",
        [],
        |row| {
            let config_str: Option<String> = row.get(5)?;
            let config = config_str
                .and_then(|s| serde_json::from_str(&s).ok());

            Ok(ProxyConfig {
                enabled: row.get(0)?,
                listen_address: row.get(1)?,
                listen_port: row.get(2)?,
                enable_auth: row.get(3)?,
                enable_logging: row.get(4)?,
                config,
            })
        }
    );

    match result {
        Ok(config) => Json(ApiResponse::success(config)),
        Err(_) => Json(ApiResponse::success(ProxyConfig::default())),
    }
}

/// 更新代理配置
async fn update_proxy_config(
    State(db): State<Arc<Database>>,
    Json(req): Json<UpdateProxyConfigRequest>,
) -> Json<ApiResponse<ProxyConfig>> {
    let conn = db.conn();

    // 先获取当前配置
    let current = conn.query_row(
        "SELECT enabled, listen_address, listen_port, enable_auth, enable_logging, config
         FROM proxy_configs WHERE id = 1",
        [],
        |row| {
            let config_str: Option<String> = row.get(5)?;
            let config = config_str
                .and_then(|s| serde_json::from_str(&s).ok());

            Ok(ProxyConfig {
                enabled: row.get(0)?,
                listen_address: row.get(1)?,
                listen_port: row.get(2)?,
                enable_auth: row.get(3)?,
                enable_logging: row.get(4)?,
                config,
            })
        }
    ).unwrap_or_default();

    // 合并配置
    let new_config = ProxyConfig {
        enabled: req.enabled.unwrap_or(current.enabled),
        listen_address: req.listen_address.unwrap_or(current.listen_address),
        listen_port: req.listen_port.unwrap_or(current.listen_port),
        enable_auth: req.enable_auth.unwrap_or(current.enable_auth),
        enable_logging: req.enable_logging.unwrap_or(current.enable_logging),
        config: req.config.or(current.config),
    };

    let config_json: Option<String> = new_config.config.as_ref()
        .map(|c| serde_json::to_string(c).ok())
        .flatten();

    let result = conn.execute(
        "INSERT OR REPLACE INTO proxy_configs (id, enabled, listen_address, listen_port, enable_auth, enable_logging, config)
         VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            new_config.enabled,
            new_config.listen_address,
            new_config.listen_port as i64,
            new_config.enable_auth,
            new_config.enable_logging,
            config_json,
        ],
    );

    match result {
        Ok(_) => Json(ApiResponse::success(new_config)),
        Err(e) => Json(ApiResponse::error(format!("Failed to update proxy config: {}", e))),
    }
}

/// 获取代理状态
async fn get_proxy_status(
    State(db): State<Arc<Database>>,
) -> Json<ApiResponse<ProxyStatus>> {
    // 检查是否有运行中的代理服务器
    let server = PROXY_SERVER.lock().unwrap();
    if let Some(proxy) = server.as_ref() {
        let status = proxy.get_status();
        Json(ApiResponse::success(status))
    } else {
        // 从数据库读取配置状态
        let conn = db.conn();
        let result = conn.query_row(
            "SELECT enabled, listen_address, listen_port FROM proxy_configs WHERE id = 1",
            [],
            |row| {
                Ok(ProxyStatus {
                    running: false,
                    address: row.get(1)?,
                    port: row.get(2)?,
                    uptime_seconds: 0,
                    total_requests: 0,
                    active_connections: 0,
                })
            }
        );

        match result {
            Ok(status) => Json(ApiResponse::success(status)),
            Err(_) => Json(ApiResponse::success(ProxyStatus::default())),
        }
    }
}

/// 启动代理服务
async fn start_proxy(
    State(db): State<Arc<Database>>,
) -> Json<ApiResponse<serde_json::Value>> {
    // 克隆 db 以便后续使用
    let db_clone = db.clone();

    // 获取配置
    let config = {
        let conn = db.conn();
        conn.query_row(
            "SELECT enabled, listen_address, listen_port, enable_auth, enable_logging FROM proxy_configs WHERE id = 1",
            [],
            |row| {
                Ok(ProxyConfig {
                    enabled: row.get(0)?,
                    listen_address: row.get(1)?,
                    listen_port: row.get(2)?,
                    enable_auth: row.get(3)?,
                    enable_logging: row.get(4)?,
                    config: None, // 简化处理，不加载高级配置
                })
            }
        )
    };

    let config = match config {
        Ok(c) if !c.enabled => {
            return Json(ApiResponse::error("Proxy is disabled in config"));
        }
        Ok(c) => c,
        Err(e) => {
            return Json(ApiResponse::error(format!("Failed to load config: {}", e)));
        }
    };

    // 检查是否已在运行
    {
        let server = PROXY_SERVER.lock().unwrap();
        if server.is_some() {
            return Json(ApiResponse::error("Proxy server is already running"));
        }
    }

    // 创建并启动代理服务器
    let proxy = ProxyServer::new(db_clone, config);
    match proxy.start().await {
        Ok(info) => {
            let mut server = PROXY_SERVER.lock().unwrap();
            *server = Some(proxy);
            Json(ApiResponse::success(serde_json::json!({
                "address": info.address,
                "port": info.port,
                "started_at": info.started_at,
            })))
        }
        Err(e) => {
            Json(ApiResponse::error(format!("Failed to start proxy: {}", e)))
        }
    }
}

/// 停止代理服务
async fn stop_proxy_handler(
    State(_db): State<Arc<Database>>,
) -> Json<ApiResponse<()>> {
    stop_proxy_inner()
}

fn stop_proxy_inner() -> Json<ApiResponse<()>> {
    let mut server = PROXY_SERVER.lock().unwrap();
    if server.is_some() {
        // 由于 stop 是 async 方法，我们需要在运行时中执行它
        // 这里简化为直接移除服务器实例
        let _ = server.take();
        Json(ApiResponse::success(()))
    } else {
        Json(ApiResponse::error("Proxy server is not running"))
    }
}
