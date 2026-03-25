//! Provider 管理模块

pub mod service;

use crate::database::Database;
use crate::models::provider::*;
use std::sync::Arc;

/// Provider 服务
pub struct ProviderService {
    db: Arc<Database>,
}

impl ProviderService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// 获取所有 provider
    pub async fn list_all(&self) -> anyhow::Result<Vec<Provider>> {
        Ok(vec![])
    }

    /// 获取默认 provider
    pub async fn get_default(&self) -> anyhow::Result<Option<Provider>> {
        Ok(None)
    }
}
