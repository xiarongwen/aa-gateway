//! Provider 路由模块

use std::collections::HashMap;
use super::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};

/// Provider 路由器
pub struct ProviderRouter {
    breakers: HashMap<String, CircuitBreaker>,
}

impl ProviderRouter {
    pub fn new() -> Self {
        Self {
            breakers: HashMap::new(),
        }
    }

    pub fn get_breaker(&mut self, provider_id: &str) -> &mut CircuitBreaker {
        self.breakers
            .entry(provider_id.to_string())
            .or_insert_with(|| CircuitBreaker::new(CircuitBreakerConfig::default()))
    }

    pub fn reset_breaker(&mut self, provider_id: &str) {
        self.breakers.remove(provider_id);
    }
}
