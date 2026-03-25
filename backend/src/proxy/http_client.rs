//! HTTP 客户端模块

use reqwest::{Client, ClientBuilder};
use std::time::Duration;

/// 创建 HTTP 客户端
pub fn create_http_client(
    timeout_secs: u64,
    connect_timeout_secs: u64,
) -> reqwest::Result<Client> {
    ClientBuilder::new()
        .timeout(Duration::from_secs(timeout_secs))
        .connect_timeout(Duration::from_secs(connect_timeout_secs))
        .pool_max_idle_per_host(32)
        .http2_keep_alive_interval(Some(Duration::from_secs(30)))
        .build()
}

/// 全局 HTTP 客户端（懒加载）
static mut GLOBAL_CLIENT: Option<Client> = None;
static INIT: std::sync::Once = std::sync::Once::new();

pub fn global_client() -> &'static Client {
    unsafe {
        INIT.call_once(|| {
            GLOBAL_CLIENT = Some(
                create_http_client(300, 30).expect("Failed to create HTTP client")
            );
        });
        GLOBAL_CLIENT.as_ref().unwrap()
    }
}

/// 根据提供商类型构建请求头
pub fn build_headers(
    api_key: &str,
    provider_type: &str,
) -> reqwest::header::HeaderMap {
    use reqwest::header::*;

    let mut headers = HeaderMap::new();

    match provider_type {
        "openai" | "azure" => {
            headers.insert(
                AUTHORIZATION,
                format!("Bearer {}", api_key).parse().unwrap(),
            );
        }
        "anthropic" => {
            headers.insert(
                "x-api-key",
                api_key.parse().unwrap(),
            );
            headers.insert(
                "anthropic-version",
                "2023-06-01".parse().unwrap(),
            );
        }
        "gemini" => {
            // Gemini 使用 query 参数传 key
        }
        _ => {
            // 默认使用 Bearer token
            headers.insert(
                AUTHORIZATION,
                format!("Bearer {}", api_key).parse().unwrap(),
            );
        }
    }

    headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
    headers.insert(ACCEPT, "application/json".parse().unwrap());

    headers
}
