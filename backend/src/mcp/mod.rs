//! MCP 管理模块

pub mod service;

use crate::database::Database;
use std::sync::Arc;

/// MCP 服务
pub struct McpService {
    db: Arc<Database>,
}

impl McpService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
}
