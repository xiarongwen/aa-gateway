//! 请求转发模块

use super::types::*;
use super::http_client::{global_client, build_headers};
use super::circuit_breaker::{CircuitBreaker, CircuitState};
use super::router::ProviderRouter;
use axum::body::{self, Body};
use axum::response::Response;
use std::sync::{Arc, Mutex};

/// 转发结果
pub struct ForwardResult {
    pub response: Response,
    pub provider_id: String,
}

/// 转发错误
pub struct ForwardError {
    pub error: ProxyError,
    pub provider_id: Option<String>,
}

/// 请求上下文
#[derive(Clone)]
pub struct RequestContext {
    pub provider_id: Option<String>,
    pub model: Option<String>,
    pub app_type: String, // claude, codex, gemini
}

/// 转发请求到 Provider
pub async fn forward_request(
    body: Body,
    headers: reqwest::header::HeaderMap,
    context: RequestContext,
    provider_router: Arc<Mutex<ProviderRouter>>,
    providers: Vec<ProviderInfo>,
) -> Result<ForwardResult, ForwardError> {
    // 选择 Provider
    let provider = if let Some(ref pid) = context.provider_id {
        providers.iter().find(|p| p.id == *pid).cloned()
    } else {
        // 选择默认 Provider
        providers.iter().find(|p| p.is_default).cloned()
            .or_else(|| providers.first().cloned())
    };

    let provider = provider.ok_or_else(|| ForwardError {
        error: ProxyError::ProviderNotFound,
        provider_id: None,
    })?;

    // 检查熔断器状态
    {
        let mut router = provider_router.lock().unwrap();
        let breaker = router.get_breaker(&provider.id);
        if breaker.state() == CircuitState::Open {
            return Err(ForwardError {
                error: ProxyError::ForwardFailed("Circuit breaker is open".to_string()),
                provider_id: Some(provider.id.clone()),
            });
        }
    }

    // 转换请求路径
    let upstream_url = build_upstream_url(&provider, &context.app_type);

    // 读取请求体
    let body_bytes = match body::to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(e) => return Err(ForwardError {
            error: ProxyError::ForwardFailed(format!("Failed to read body: {}", e)),
            provider_id: Some(provider.id.clone()),
        }),
    };

    // 转换请求格式（如果需要）
    let body_bytes = transform_request(body_bytes, &provider.provider_type, &context.app_type);

    // 构建请求头
    let mut request_headers = build_headers(&provider.api_key, &provider.provider_type);

    // 添加原始请求中的其他 headers（排除黑名单）
    const HEADER_BLACKLIST: &[&str] = &[
        "host",
        "content-length",
        "transfer-encoding",
        "authorization",
        "x-api-key",
        "anthropic-beta",
        "anthropic-version",
    ];

    for (key, value) in headers.iter() {
        let key_str = key.as_str().to_lowercase();
        if !HEADER_BLACKLIST.contains(&key_str.as_str()) {
            request_headers.insert(key.clone(), value.clone());
        }
    }

    // 发送请求
    let client = global_client();
    let response = match client
        .post(&upstream_url)
        .headers(request_headers)
        .body(body_bytes)
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            // 记录失败到熔断器
            let mut router = provider_router.lock().unwrap();
            let breaker = router.get_breaker(&provider.id);
            breaker.record_failure();

            return Err(ForwardError {
                error: ProxyError::ForwardFailed(format!("Request failed: {}", e)),
                provider_id: Some(provider.id.clone()),
            });
        }
    };

    // 检查响应状态
    let status = response.status();
    if status.is_server_error() || status.is_client_error() {
        let mut router = provider_router.lock().unwrap();
        let breaker = router.get_breaker(&provider.id);
        breaker.record_failure();
    } else {
        let mut router = provider_router.lock().unwrap();
        let breaker = router.get_breaker(&provider.id);
        breaker.record_success();
    }

    // 转换为 Axum Response
    let mut builder = Response::builder().status(response.status());

    // 复制响应头
    for (key, value) in response.headers() {
        if key != "transfer-encoding" { // 排除 transfer-encoding，Axum 会自动处理
            builder = builder.header(key.as_str(), value.as_bytes());
        }
    }

    // 处理流式响应
    let stream = response.bytes_stream();
    let body = Body::from_stream(stream);

    let axum_response = builder.body(body).map_err(|e| ForwardError {
        error: ProxyError::ForwardFailed(format!("Failed to build response: {}", e)),
        provider_id: Some(provider.id.clone()),
    })?;

    Ok(ForwardResult {
        response: axum_response,
        provider_id: provider.id,
    })
}

/// Provider 信息
#[derive(Clone)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub provider_type: String,
    pub base_url: String,
    pub api_key: String,
    pub is_default: bool,
}

/// 构建上游 URL
fn build_upstream_url(provider: &ProviderInfo, app_type: &str) -> String {
    let base = provider.base_url.trim_end_matches('/');

    match app_type {
        "claude" => {
            match provider.provider_type.as_str() {
                "anthropic" => format!("{}/v1/messages", base),
                _ => format!("{}/v1/chat/completions", base), // OpenAI 兼容格式
            }
        }
        "codex" => {
            format!("{}/v1/chat/completions", base)
        }
        "gemini" => {
            if provider.provider_type == "gemini" {
                format!("{}/v1beta/models/gemini-pro:generateContent?key={}", base, provider.api_key)
            } else {
                format!("{}/v1/chat/completions", base)
            }
        }
        _ => format!("{}/v1/chat/completions", base),
    }
}

/// 转换请求格式
fn transform_request(
    body_bytes: bytes::Bytes,
    provider_type: &str,
    app_type: &str,
) -> bytes::Bytes {
    // 如果目标格式相同，直接返回
    if app_type == "claude" && provider_type == "anthropic" {
        return body_bytes;
    }
    if (app_type == "codex" || app_type == "gemini") && provider_type == "openai" {
        return body_bytes;
    }

    // 尝试进行格式转换
    let body_str = match std::str::from_utf8(&body_bytes) {
        Ok(s) => s,
        Err(_) => return body_bytes,
    };

    let json: serde_json::Value = match serde_json::from_str(body_str) {
        Ok(v) => v,
        Err(_) => return body_bytes,
    };

    // Claude -> OpenAI 格式转换
    if app_type == "claude" && provider_type != "anthropic" {
        if let Some(converted) = convert_claude_to_openai(&json) {
            return bytes::Bytes::from(converted.to_string());
        }
    }

    // OpenAI -> Claude 格式转换
    if app_type != "claude" && provider_type == "anthropic" {
        if let Some(converted) = convert_openai_to_claude(&json) {
            return bytes::Bytes::from(converted.to_string());
        }
    }

    body_bytes
}

/// Claude 格式转 OpenAI 格式
fn convert_claude_to_openai(claude_req: &serde_json::Value) -> Option<serde_json::Value> {
    let messages = claude_req.get("messages")?;
    let model = claude_req.get("model")?.as_str()?;
    let max_tokens = claude_req.get("max_tokens")?;

    let mut openai_req = serde_json::json!({
        "model": model,
        "messages": messages,
        "max_tokens": max_tokens,
    });

    // 复制可选字段
    if let Some(stream) = claude_req.get("stream") {
        openai_req["stream"] = stream.clone();
    }
    if let Some(temperature) = claude_req.get("temperature") {
        openai_req["temperature"] = temperature.clone();
    }

    Some(openai_req)
}

/// OpenAI 格式转 Claude 格式
fn convert_openai_to_claude(openai_req: &serde_json::Value) -> Option<serde_json::Value> {
    let messages = openai_req.get("messages")?;
    let model = openai_req.get("model")?.as_str()?;

    // 计算 max_tokens（Claude 需要）
    let max_tokens = openai_req
        .get("max_tokens")
        .and_then(|v| v.as_i64())
        .unwrap_or(4096);

    let mut claude_req = serde_json::json!({
        "model": model,
        "messages": messages,
        "max_tokens": max_tokens,
    });

    // 复制可选字段
    if let Some(stream) = openai_req.get("stream") {
        claude_req["stream"] = stream.clone();
    }
    if let Some(temperature) = openai_req.get("temperature") {
        claude_req["temperature"] = temperature.clone();
    }

    Some(claude_req)
}
