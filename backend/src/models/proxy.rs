//! 代理配置模型定义

use serde::{Deserialize, Serialize};

/// 代理服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub enabled: bool,
    #[serde(rename = "listenAddress")]
    pub listen_address: String,
    #[serde(rename = "listenPort")]
    pub listen_port: u16,
    #[serde(rename = "enableAuth")]
    pub enable_auth: bool,
    #[serde(rename = "enableLogging")]
    pub enable_logging: bool,
    pub config: Option<ProxyAdvancedConfig>,
}

/// 代理高级配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyAdvancedConfig {
    /// 请求超时（秒）
    #[serde(rename = "requestTimeoutSecs", skip_serializing_if = "Option::is_none")]
    pub request_timeout_secs: Option<u64>,
    /// 连接超时（秒）
    #[serde(rename = "connectTimeoutSecs", skip_serializing_if = "Option::is_none")]
    pub connect_timeout_secs: Option<u64>,
    /// 最大连接数
    #[serde(rename = "maxConnections", skip_serializing_if = "Option::is_none")]
    pub max_connections: Option<usize>,
    /// 是否启用负载均衡
    #[serde(rename = "enableLoadBalance", skip_serializing_if = "Option::is_none")]
    pub enable_load_balance: Option<bool>,
    /// 是否启用故障转移
    #[serde(rename = "enableFailover", skip_serializing_if = "Option::is_none")]
    pub enable_failover: Option<bool>,
    /// 故障转移阈值（连续失败次数）
    #[serde(rename = "failoverThreshold", skip_serializing_if = "Option::is_none")]
    pub failover_threshold: Option<u32>,
    /// 熔断器配置
    #[serde(rename = "circuitBreaker", skip_serializing_if = "Option::is_none")]
    pub circuit_breaker: Option<CircuitBreakerConfig>,
}

/// 熔断器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// 失败率阈值（0.0 - 1.0）
    #[serde(rename = "failureRateThreshold")]
    pub failure_rate_threshold: f64,
    /// 慢调用率阈值
    #[serde(rename = "slowCallRateThreshold")]
    pub slow_call_rate_threshold: f64,
    /// 慢调用时间阈值（毫秒）
    #[serde(rename = "slowCallDurationThresholdMs")]
    pub slow_call_duration_threshold_ms: u64,
    /// 半开状态下的允许请求数
    #[serde(rename = "permittedCallsInHalfOpenState")]
    pub permitted_calls_in_half_open_state: u32,
    /// 等待持续时间（毫秒）
    #[serde(rename = "waitDurationInOpenStateMs")]
    pub wait_duration_in_open_state_ms: u64,
    /// 滑动窗口大小
    #[serde(rename = "slidingWindowSize")]
    pub sliding_window_size: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_rate_threshold: 0.5,
            slow_call_rate_threshold: 0.5,
            slow_call_duration_threshold_ms: 60000,
            permitted_calls_in_half_open_state: 5,
            wait_duration_in_open_state_ms: 30000,
            sliding_window_size: 100,
        }
    }
}

/// 代理服务器状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyStatus {
    pub running: bool,
    pub address: String,
    pub port: u16,
    #[serde(rename = "uptimeSeconds")]
    pub uptime_seconds: u64,
    #[serde(rename = "totalRequests")]
    pub total_requests: u64,
    #[serde(rename = "activeConnections")]
    pub active_connections: u32,
}

impl Default for ProxyStatus {
    fn default() -> Self {
        Self {
            running: false,
            address: "0.0.0.0".to_string(),
            port: 8080,
            uptime_seconds: 0,
            total_requests: 0,
            active_connections: 0,
        }
    }
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            listen_address: "0.0.0.0".to_string(),
            listen_port: 8080,
            enable_auth: true,
            enable_logging: true,
            config: None,
        }
    }
}

/// 更新代理配置请求
#[derive(Debug, Deserialize)]
pub struct UpdateProxyConfigRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(rename = "listenAddress", skip_serializing_if = "Option::is_none")]
    pub listen_address: Option<String>,
    #[serde(rename = "listenPort", skip_serializing_if = "Option::is_none")]
    pub listen_port: Option<u16>,
    #[serde(rename = "enableAuth", skip_serializing_if = "Option::is_none")]
    pub enable_auth: Option<bool>,
    #[serde(rename = "enableLogging", skip_serializing_if = "Option::is_none")]
    pub enable_logging: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<ProxyAdvancedConfig>,
}
