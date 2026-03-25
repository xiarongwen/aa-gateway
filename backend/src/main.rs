mod database;
mod mcp;
mod models;
mod providers;
mod proxy;
mod server;

use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber::EnvFilter;

use crate::database::Database;
use crate::server::create_app;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive(Level::INFO.into())
                .add_directive("ai_gateway=debug".parse()?),
        )
        .init();

    info!("🚀 AI Gateway 启动中...");

    // 初始化数据库
    let db = Arc::new(Database::init().await?);
    info!("✓ 数据库初始化完成");

    // 创建应用
    let app = create_app(db.clone()).await?;

    // 从环境变量或配置文件读取端口
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()?;

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("🌐 HTTP 服务监听于 http://{}", addr);

    // 启动代理服务（可选）
    if std::env::var("ENABLE_PROXY").unwrap_or_else(|_| "true".to_string()) == "true" {
        let proxy_port = std::env::var("PROXY_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()?;
        let proxy_db = db.clone();
        tokio::spawn(async move {
            if let Err(e) = start_proxy_service(proxy_db, proxy_port).await {
                tracing::error!("代理服务启动失败: {}", e);
            }
        });
    }

    // 启动 HTTP 服务
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn start_proxy_service(
    db: Arc<Database>,
    port: u16,
) -> anyhow::Result<()> {
    info!("🔌 代理服务启动中，端口: {}", port);
    // TODO: 实现代理服务启动
    // 这里将复用 cc-switch 的代理逻辑
    Ok(())
}
