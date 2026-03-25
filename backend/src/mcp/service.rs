//! MCP 服务实现

use super::*;

pub struct McpManager {
    db: Arc<Database>,
}

impl McpManager {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
}
