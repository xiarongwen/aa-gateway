//! Provider 服务实现

use super::*;

pub struct ProviderManager {
    db: Arc<Database>,
}

impl ProviderManager {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }
}
