//! 代理服务模块
//!
//! 基于 Axum 的 HTTP 代理服务，支持多 Provider 故障转移和请求透传
//! 复用 cc-switch 的代理逻辑

use std::sync::Arc;
use crate::database::Database;

pub mod types;
pub mod server;
pub mod forwarder;
pub mod circuit_breaker;
pub mod router;
pub mod http_client;

pub use types::*;
pub use server::ProxyServer;
