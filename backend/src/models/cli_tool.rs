//! CLI 工具配置模型定义

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// CLI 工具类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CliToolType {
    #[serde(rename = "claude_code")]
    ClaudeCode,   // ~/.claude/settings.json
    #[serde(rename = "codex")]
    Codex,        // ~/.codex/config.json
    #[serde(rename = "gemini_cli")]
    GeminiCli,    // ~/.gemini/config.json
    #[serde(rename = "opencode")]
    OpenCode,     // ~/.opencode/config.json
    #[serde(rename = "openclaw")]
    OpenClaw,     // ~/.openclaw/config.json
}

impl CliToolType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CliToolType::ClaudeCode => "claude_code",
            CliToolType::Codex => "codex",
            CliToolType::GeminiCli => "gemini_cli",
            CliToolType::OpenCode => "opencode",
            CliToolType::OpenClaw => "openclaw",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            CliToolType::ClaudeCode => "Claude Code",
            CliToolType::Codex => "Codex",
            CliToolType::GeminiCli => "Gemini CLI",
            CliToolType::OpenCode => "OpenCode",
            CliToolType::OpenClaw => "OpenClaw",
        }
    }

    pub fn config_path(&self) -> &'static str {
        match self {
            CliToolType::ClaudeCode => "~/.claude.json",
            CliToolType::Codex => "~/.codex/config.json",
            CliToolType::GeminiCli => "~/.gemini/config.json",
            CliToolType::OpenCode => "~/.opencode/config.json",
            CliToolType::OpenClaw => "~/.openclaw/config.json",
        }
    }
}

/// CLI 工具配置模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliTool {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub tool_type: String,
    pub provider_id: String,
    pub api_key: String,
    pub api_url: String,
    pub model: String,
    pub enabled: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

/// CLI 工具配置内容（用于生成配置文件）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliToolConfig {
    pub api_key: String,
    pub api_url: String,
    pub model: String,
}

/// 创建 CLI 工具配置请求
#[derive(Debug, Deserialize)]
pub struct CreateCliToolRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "toolType")]
    pub tool_type: String,
    #[serde(rename = "providerId")]
    pub provider_id: String,
    #[serde(rename = "apiKey")]
    pub api_key: String,
    #[serde(rename = "apiUrl")]
    pub api_url: String,
    pub model: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

/// 更新 CLI 工具配置请求
#[derive(Debug, Deserialize)]
pub struct UpdateCliToolRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "providerId", skip_serializing_if = "Option::is_none")]
    pub provider_id: Option<String>,
    #[serde(rename = "apiKey", skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(rename = "apiUrl", skip_serializing_if = "Option::is_none")]
    pub api_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

/// 生成配置文件响应
#[derive(Debug, Serialize)]
pub struct CliToolConfigOutput {
    pub tool_type: String,
    pub config_path: String,
    pub config_content: String,
    pub env_vars: Option<Vec<(String, String)>>,
}

fn default_true() -> bool {
    true
}

impl CliTool {
    pub fn new(
        name: String,
        description: Option<String>,
        tool_type: String,
        provider_id: String,
        api_key: String,
        api_url: String,
        model: String,
    ) -> anyhow::Result<Self> {
        let now = Utc::now().timestamp_millis();
        Ok(Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            tool_type,
            provider_id,
            api_key,
            api_url,
            model,
            enabled: true,
            created_at: now,
            updated_at: now,
        })
    }

    /// 生成对应 CLI 工具的配置文件内容
    pub fn generate_config(&self) -> anyhow::Result<CliToolConfigOutput> {
        let tool_type = CliToolType::try_from(self.tool_type.as_str())?;

        match tool_type {
            CliToolType::ClaudeCode => self.generate_claude_code_config(),
            CliToolType::Codex => self.generate_codex_config(),
            CliToolType::GeminiCli => self.generate_gemini_config(),
            CliToolType::OpenCode => self.generate_opencode_config(),
            CliToolType::OpenClaw => self.generate_openclaw_config(),
        }
    }

    fn generate_claude_code_config(&self) -> anyhow::Result<CliToolConfigOutput> {
        // Claude Code 使用 ~/.claude.json 格式
        // 完整配置支持第三方 API 和跳过登录/权限引导
        let config = serde_json::json!({
            "primaryApiKey": self.api_key,
            "anthropicBaseUrl": self.api_url,
            "defaultModel": self.model,
            "env": {
                "ANTHROPIC_AUTH_TOKEN": self.api_key,
                "ANTHROPIC_BASE_URL": self.api_url,
                "ANTHROPIC_MODEL": self.model,
                "ANTHROPIC_SMALL_FAST_MODEL": self.model,
            },
            "permissions": {
                "defaultMode": "bypassPermissions"
            },
            "skipDangerousModePermissionPrompt": true,
        });

        Ok(CliToolConfigOutput {
            tool_type: self.tool_type.clone(),
            config_path: CliToolType::ClaudeCode.config_path().to_string(),
            config_content: serde_json::to_string_pretty(&config)?,
            env_vars: Some(vec![
                ("ANTHROPIC_AUTH_TOKEN".to_string(), self.api_key.clone()),
                ("ANTHROPIC_BASE_URL".to_string(), self.api_url.clone()),
                ("ANTHROPIC_MODEL".to_string(), self.model.clone()),
                ("ANTHROPIC_SMALL_FAST_MODEL".to_string(), self.model.clone()),
            ]),
        })
    }

    fn generate_codex_config(&self) -> anyhow::Result<CliToolConfigOutput> {
        // Codex 使用环境变量方式
        let config = serde_json::json!({
            "model": self.model,
            "provider": "custom",
        });

        Ok(CliToolConfigOutput {
            tool_type: self.tool_type.clone(),
            config_path: CliToolType::Codex.config_path().to_string(),
            config_content: serde_json::to_string_pretty(&config)?,
            env_vars: Some(vec![
                ("OPENAI_API_KEY".to_string(), self.api_key.clone()),
                ("OPENAI_BASE_URL".to_string(), self.api_url.clone()),
            ]),
        })
    }

    fn generate_gemini_config(&self) -> anyhow::Result<CliToolConfigOutput> {
        let config = serde_json::json!({
            "api_key": self.api_key,
            "api_url": self.api_url,
            "model": self.model,
        });

        Ok(CliToolConfigOutput {
            tool_type: self.tool_type.clone(),
            config_path: CliToolType::GeminiCli.config_path().to_string(),
            config_content: serde_json::to_string_pretty(&config)?,
            env_vars: None,
        })
    }

    fn generate_opencode_config(&self) -> anyhow::Result<CliToolConfigOutput> {
        let config = serde_json::json!({
            "provider": {
                "name": "custom",
                "api_key": self.api_key,
                "base_url": self.api_url,
                "model": self.model,
            }
        });

        Ok(CliToolConfigOutput {
            tool_type: self.tool_type.clone(),
            config_path: CliToolType::OpenCode.config_path().to_string(),
            config_content: serde_json::to_string_pretty(&config)?,
            env_vars: None,
        })
    }

    fn generate_openclaw_config(&self) -> anyhow::Result<CliToolConfigOutput> {
        let config = serde_json::json!({
            "api_key": self.api_key,
            "base_url": self.api_url,
            "model": self.model,
        });

        Ok(CliToolConfigOutput {
            tool_type: self.tool_type.clone(),
            config_path: CliToolType::OpenClaw.config_path().to_string(),
            config_content: serde_json::to_string_pretty(&config)?,
            env_vars: None,
        })
    }
}

impl TryFrom<&str> for CliToolType {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "claude_code" => Ok(CliToolType::ClaudeCode),
            "codex" => Ok(CliToolType::Codex),
            "gemini_cli" => Ok(CliToolType::GeminiCli),
            "opencode" => Ok(CliToolType::OpenCode),
            "openclaw" => Ok(CliToolType::OpenClaw),
            _ => Err(anyhow::anyhow!("Unknown CLI tool type: {}", value)),
        }
    }
}
