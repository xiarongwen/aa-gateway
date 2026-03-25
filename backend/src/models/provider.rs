//! Provider 模型定义

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 供应商类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderType {
    #[serde(rename = "openai")]
    OpenAI,
    #[serde(rename = "anthropic")]
    Anthropic,
    #[serde(rename = "gemini")]
    Gemini,
    #[serde(rename = "azure")]
    Azure,
    #[serde(rename = "bedrock")]
    Bedrock,
    #[serde(rename = "custom")]
    Custom,
    #[serde(rename = "ollama")]
    Ollama,
}

/// 供应商分类
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderCategory {
    Official,      // 官方
    CloudProvider, // 云服务商
    Aggregator,    // 聚合网站
    ThirdParty,    // 第三方
    Custom,        // 自定义
}

/// 供应商模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub id: String,
    pub name: String,
    pub provider_type: String,
    pub base_url: String,
    pub api_key: String, // 注意：实际存储应该加密
    pub models: String,  // JSON 字符串
    pub config: Option<String>, // JSON 字符串，额外配置
    pub category: Option<String>,
    pub is_default: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 模型配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_window: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_cost_per_1k: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_cost_per_1k: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<Vec<String>>,
}

/// 供应商配置（JSON 字段）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_retries: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<std::collections::HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,
}

/// 创建供应商请求
#[derive(Debug, Deserialize)]
pub struct CreateProviderRequest {
    pub name: String,
    #[serde(rename = "providerType")]
    pub provider_type: String,
    #[serde(rename = "baseUrl")]
    pub base_url: String,
    #[serde(rename = "apiKey")]
    pub api_key: String,
    pub models: Vec<ModelConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<ProviderConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

/// 更新供应商请求
#[derive(Debug, Deserialize)]
pub struct UpdateProviderRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "baseUrl", skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(rename = "apiKey", skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub models: Option<Vec<ModelConfig>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<ProviderConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(rename = "isDefault", skip_serializing_if = "Option::is_none")]
    pub is_default: Option<bool>,
}

impl Provider {
    pub fn new(
        name: String,
        provider_type: String,
        base_url: String,
        api_key: String,
        models: Vec<ModelConfig>,
    ) -> anyhow::Result<Self> {
        let now = Utc::now().timestamp_millis();
        Ok(Self {
            id: Uuid::new_v4().to_string(),
            name,
            provider_type,
            base_url,
            api_key,
            models: serde_json::to_string(&models)?,
            config: None,
            category: Some("custom".to_string()),
            is_default: false,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn get_models(&self) -> anyhow::Result<Vec<ModelConfig>> {
        Ok(serde_json::from_str(&self.models)?)
    }

    pub fn get_config(&self) -> anyhow::Result<Option<ProviderConfig>> {
        match &self.config {
            Some(config_str) => Ok(Some(serde_json::from_str(config_str)?)),
            None => Ok(None),
        }
    }
}
