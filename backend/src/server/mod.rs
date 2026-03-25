//! HTTP 服务器模块

use axum::{
    routing::{get},
    Router,
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use tower_http::cors::{CorsLayer, Any};
use tower_http::services::ServeDir;

use crate::database::Database;

mod providers;
mod mcp;
mod proxy;
mod cli_tools;

use providers::provider_routes;
use mcp::mcp_routes;
use proxy::proxy_routes;
use cli_tools::cli_tool_routes;

/// 创建应用路由
pub async fn create_app(db: Arc<Database>) -> anyhow::Result<Router> {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 静态文件目录（Docker 环境中为 /app/static，开发环境可能不存在）
    let static_dir = std::env::var("STATIC_DIR").unwrap_or_else(|_| {
        // 检查是否是 Docker 环境
        if std::path::Path::new("/app/static").exists() {
            "/app/static".to_string()
        } else {
            // 开发环境：使用前端开发服务器
            "frontend/dist".to_string()
        }
    });

    // 先创建带状态的路由
    let api_router = api_routes().with_state(db.clone());

    // 静态文件服务
    let static_service = ServeDir::new(&static_dir)
        .append_index_html_on_directories(true);

    let app = Router::new()
        .nest("/api", api_router)
        .route("/health", get(health_check))
        .fallback_service(static_service)
        .layer(cors)
        .with_state(db);

    Ok(app)
}

/// API 路由
fn api_routes() -> Router<Arc<Database>> {
    Router::new()
        .nest("/providers", provider_routes())
        .nest("/mcp", mcp_routes())
        .nest("/proxy", proxy_routes())
        .nest("/cli-tools", cli_tool_routes())
}

async fn health_check() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok",
            "service": "ai-gateway",
            "version": env!("CARGO_PKG_VERSION"),
        }))
    )
}

