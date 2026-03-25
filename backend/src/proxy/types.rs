//! 代理服务类型定义

use serde::{Deserialize, Serialize};

// ProxyConfig 从 models::proxy 重新导出，以保持类型一致性
pub use crate::models::proxy::{ProxyConfig, ProxyAdvancedConfig};

/// 代理错误类型
#[derive(Debug, thiserror::Error)]
pub enum ProxyError {
    #[error("代理服务已在运行")]
    AlreadyRunning,
    #[error("代理服务未运行")]
    NotRunning,
    #[error("绑定失败: {0}")]
    BindFailed(String),
    #[error("停止失败: {0}")]
    StopFailed(String),
    #[error("停止超时")]
    StopTimeout,
    #[error("请求转发失败: {0}")]
    ForwardFailed(String),
    #[error("Provider 未找到")]
    ProviderNotFound,
    #[error("认证失败")]
    AuthFailed,
}

/// 代理服务器信息
#[derive(Debug, Serialize, Deserialize)]
pub struct ProxyServerInfo {
    pub address: String,
    pub port: u16,
    pub started_at: String,
}

/// 代理请求上下文
#[derive(Debug, Clone)]
pub struct ProxyContext {
    pub provider_id: String,
    pub model: Option<String>,
    pub request_id: String,
    pub start_time: std::time::Instant,
}

// ProxyStatus 从 models::proxy 重新导出
pub use crate::models::proxy::ProxyStatus;
