//! MCP 服务器模型定义

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// MCP 服务器类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum McpServerType {
    Stdio,
    Http,
    Sse,
}

/// MCP 服务器模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServer {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub server_type: String,
    pub command: Option<String>,
    pub args: Option<String>, // JSON 数组
    pub env: Option<String>, // JSON 对象
    pub url: Option<String>,
    pub headers: Option<String>, // JSON 对象
    pub enabled: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

/// MCP 服务器配置（stdio 类型）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpStdioConfig {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env: Option<std::collections::HashMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
}

/// MCP 服务器配置（http/sse 类型）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpHttpConfig {
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub headers: Option<std::collections::HashMap<String, String>>,
}

/// 创建 MCP 服务器请求
#[derive(Debug, Deserialize)]
pub struct CreateMcpServerRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "serverType")]
    pub server_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env: Option<std::collections::HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub headers: Option<std::collections::HashMap<String, String>>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

/// 更新 MCP 服务器请求
#[derive(Debug, Deserialize)]
pub struct UpdateMcpServerRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "serverType", skip_serializing_if = "Option::is_none")]
    pub server_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env: Option<std::collections::HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub headers: Option<std::collections::HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

fn default_true() -> bool {
    true
}

impl McpServer {
    pub fn new_stdio(
        name: String,
        description: Option<String>,
        command: String,
        args: Vec<String>,
        env: Option<std::collections::HashMap<String, String>>,
    ) -> anyhow::Result<Self> {
        let now = Utc::now().timestamp_millis();
        Ok(Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            server_type: "stdio".to_string(),
            command: Some(command),
            args: Some(serde_json::to_string(&args)?),
            env: env.map(|e| serde_json::to_string(&e).ok()).flatten(),
            url: None,
            headers: None,
            enabled: true,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn new_http(
        name: String,
        description: Option<String>,
        url: String,
        headers: Option<std::collections::HashMap<String, String>>,
        is_sse: bool,
    ) -> anyhow::Result<Self> {
        let now = Utc::now().timestamp_millis();
        Ok(Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            server_type: if is_sse { "sse".to_string() } else { "http".to_string() },
            command: None,
            args: None,
            env: None,
            url: Some(url),
            headers: headers.map(|h| serde_json::to_string(&h).ok()).flatten(),
            enabled: true,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn get_args(&self) -> anyhow::Result<Option<Vec<String>>> {
        match &self.args {
            Some(args_str) => Ok(Some(serde_json::from_str(args_str)?)),
            None => Ok(None),
        }
    }

    pub fn get_env(&self) -> anyhow::Result<Option<std::collections::HashMap<String, String>>> {
        match &self.env {
            Some(env_str) => Ok(Some(serde_json::from_str(env_str)?)),
            None => Ok(None),
        }
    }

    pub fn get_headers(&self) -> anyhow::Result<Option<std::collections::HashMap<String, String>>> {
        match &self.headers {
            Some(headers_str) => Ok(Some(serde_json::from_str(headers_str)?)),
            None => Ok(None),
        }
    }
}
