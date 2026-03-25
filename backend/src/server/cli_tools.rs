//! CLI 工具配置 API 路由

use axum::{
    extract::{Path, State},
    routing::{get, post, put, delete},
    Json, Router,
};
use std::sync::Arc;
use std::path::PathBuf;

use crate::database::Database;
use crate::models::cli_tool::*;
use crate::models::{ApiResponse, PaginationParams, PaginatedResponse};

/// CLI 工具路由
pub fn cli_tool_routes() -> Router<Arc<Database>> {
    Router::new()
        .route("/", get(list_cli_tools).post(create_cli_tool))
        .route("/:id", get(get_cli_tool).put(update_cli_tool).delete(delete_cli_tool))
        .route("/:id/toggle", post(toggle_cli_tool))
        .route("/:id/config", get(generate_cli_tool_config))
        .route("/:id/apply", post(apply_cli_tool_config))
        .route("/:id/backup", post(backup_cli_tool_config))
        .route("/:id/restore", post(restore_cli_tool_backup))
        .route("/types", get(list_cli_tool_types))
        .route("/skip-onboarding", post(skip_claude_onboarding))
}

/// 获取 CLI 工具配置列表
async fn list_cli_tools(
    State(db): State<Arc<Database>>,
    axum::extract::Query(params): axum::extract::Query<PaginationParams>,
) -> Json<ApiResponse<PaginatedResponse<CliTool>>> {
    let conn = db.conn();

    let total: i64 = conn
        .query_row("SELECT COUNT(*) FROM cli_tools", [], |row| row.get(0))
        .unwrap_or(0);

    let offset = (params.page - 1) * params.per_page;
    let mut stmt = conn.prepare(
        "SELECT id, name, description, tool_type, provider_id, api_key, api_url, model, enabled, created_at, updated_at
         FROM cli_tools
         ORDER BY created_at DESC
         LIMIT ? OFFSET ?"
    ).unwrap();

    let tools: Vec<CliTool> = stmt
        .query_map([params.per_page, offset], |row| {
            Ok(CliTool {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                tool_type: row.get(3)?,
                provider_id: row.get(4)?,
                api_key: row.get(5)?,
                api_url: row.get(6)?,
                model: row.get(7)?,
                enabled: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    let response = PaginatedResponse::new(tools, total, params.page, params.per_page);
    Json(ApiResponse::success(response))
}

/// 获取单个 CLI 工具配置
async fn get_cli_tool(
    State(db): State<Arc<Database>>,
    Path(id): Path<String>,
) -> Json<ApiResponse<CliTool>> {
    let conn = db.conn();

    let result = conn.query_row(
        "SELECT id, name, description, tool_type, provider_id, api_key, api_url, model, enabled, created_at, updated_at
         FROM cli_tools WHERE id = ?",
        [&id],
        |row| {
            Ok(CliTool {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                tool_type: row.get(3)?,
                provider_id: row.get(4)?,
                api_key: row.get(5)?,
                api_url: row.get(6)?,
                model: row.get(7)?,
                enabled: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        }
    );

    match result {
        Ok(tool) => Json(ApiResponse::success(tool)),
        Err(_) => Json(ApiResponse::error("CLI tool config not found")),
    }
}

/// 创建 CLI 工具配置
async fn create_cli_tool(
    State(db): State<Arc<Database>>,
    Json(req): Json<CreateCliToolRequest>,
) -> Json<ApiResponse<CliTool>> {
    let tool = match CliTool::new(
        req.name,
        req.description,
        req.tool_type,
        req.provider_id,
        req.api_key,
        req.api_url,
        req.model,
    ) {
        Ok(t) => t,
        Err(e) => return Json(ApiResponse::error(format!("Failed to create CLI tool: {}", e))),
    };

    let conn = db.conn();

    // 如果启用，先禁用同一工具类型的其他配置
    if tool.enabled {
        if let Err(e) = conn.execute(
            "UPDATE cli_tools SET enabled = FALSE WHERE tool_type = ?",
            [&tool.tool_type],
        ) {
            return Json(ApiResponse::error(format!("Failed to update other configs: {}", e)));
        }
    }

    let result = conn.execute(
        "INSERT INTO cli_tools (id, name, description, tool_type, provider_id, api_key, api_url, model, enabled, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        rusqlite::params![
            tool.id,
            tool.name,
            tool.description,
            tool.tool_type,
            tool.provider_id,
            tool.api_key,
            tool.api_url,
            tool.model,
            tool.enabled,
            tool.created_at,
            tool.updated_at,
        ],
    );

    match result {
        Ok(_) => Json(ApiResponse::success(tool)),
        Err(e) => Json(ApiResponse::error(format!("Failed to create CLI tool: {}", e))),
    }
}

/// 更新 CLI 工具配置
async fn update_cli_tool(
    State(db): State<Arc<Database>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateCliToolRequest>,
) -> Json<ApiResponse<CliTool>> {
    let conn = db.conn();

    // 先获取当前配置
    let current: Result<CliTool, _> = conn.query_row(
        "SELECT id, name, description, tool_type, provider_id, api_key, api_url, model, enabled, created_at, updated_at
         FROM cli_tools WHERE id = ?",
        [&id],
        |row| {
            Ok(CliTool {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                tool_type: row.get(3)?,
                provider_id: row.get(4)?,
                api_key: row.get(5)?,
                api_url: row.get(6)?,
                model: row.get(7)?,
                enabled: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        }
    );

    let mut tool = match current {
        Ok(t) => t,
        Err(_) => return Json(ApiResponse::error("CLI tool config not found")),
    };

    // 更新字段
    if let Some(name) = req.name {
        tool.name = name;
    }
    if let Some(description) = req.description {
        tool.description = Some(description);
    }
    if let Some(provider_id) = req.provider_id {
        tool.provider_id = provider_id;
    }
    if let Some(api_key) = req.api_key {
        tool.api_key = api_key;
    }
    if let Some(api_url) = req.api_url {
        tool.api_url = api_url;
    }
    if let Some(model) = req.model {
        tool.model = model;
    }
    if let Some(enabled) = req.enabled {
        tool.enabled = enabled;
        // 如果启用，禁用同类型的其他配置
        if enabled {
            if let Err(e) = conn.execute(
                "UPDATE cli_tools SET enabled = FALSE WHERE tool_type = ? AND id != ?",
                [&tool.tool_type, &id],
            ) {
                return Json(ApiResponse::error(format!("Failed to update other configs: {}", e)));
            }
        }
    }

    // 更新时间戳
    tool.updated_at = chrono::Utc::now().timestamp_millis();

    // 执行更新
    let result = conn.execute(
        "UPDATE cli_tools SET name = ?1, description = ?2, provider_id = ?3, api_key = ?4,
         api_url = ?5, model = ?6, enabled = ?7, updated_at = ?8 WHERE id = ?9",
        rusqlite::params![
            &tool.name,
            &tool.description,
            &tool.provider_id,
            &tool.api_key,
            &tool.api_url,
            &tool.model,
            &tool.enabled,
            &tool.updated_at,
            &id,
        ],
    );

    match result {
        Ok(_) => Json(ApiResponse::success(tool)),
        Err(e) => Json(ApiResponse::error(format!("Failed to update CLI tool: {}", e))),
    }
}

/// 删除 CLI 工具配置
async fn delete_cli_tool(
    State(db): State<Arc<Database>>,
    Path(id): Path<String>,
) -> Json<ApiResponse<()>> {
    let conn = db.conn();
    let result = conn.execute("DELETE FROM cli_tools WHERE id = ?", [&id]);

    match result {
        Ok(rows) if rows > 0 => Json(ApiResponse::success(())),
        Ok(_) => Json(ApiResponse::error("CLI tool config not found")),
        Err(e) => Json(ApiResponse::error(format!("Failed to delete CLI tool: {}", e))),
    }
}

/// 切换 CLI 工具启用状态
async fn toggle_cli_tool(
    State(db): State<Arc<Database>>,
    Path(id): Path<String>,
) -> Json<ApiResponse<()>> {
    let conn = db.conn();

    // 获取当前配置的类型
    let tool_type: Result<String, _> = conn.query_row(
        "SELECT tool_type FROM cli_tools WHERE id = ?",
        [&id],
        |row| row.get(0),
    );

    let tool_type = match tool_type {
        Ok(t) => t,
        Err(_) => return Json(ApiResponse::error("CLI tool config not found")),
    };

    // 禁用该类型的所有配置
    if let Err(e) = conn.execute(
        "UPDATE cli_tools SET enabled = FALSE WHERE tool_type = ?",
        [&tool_type],
    ) {
        return Json(ApiResponse::error(format!("Failed to update configs: {}", e)));
    }

    // 启用指定配置
    match conn.execute(
        "UPDATE cli_tools SET enabled = TRUE, updated_at = ? WHERE id = ?",
        rusqlite::params![chrono::Utc::now().timestamp_millis(), id],
    ) {
        Ok(rows) if rows > 0 => Json(ApiResponse::success(())),
        Ok(_) => Json(ApiResponse::error("CLI tool config not found")),
        Err(e) => Json(ApiResponse::error(format!("Failed to toggle CLI tool: {}", e))),
    }
}

/// 生成 CLI 工具配置文件
async fn generate_cli_tool_config(
    State(db): State<Arc<Database>>,
    Path(id): Path<String>,
) -> Json<ApiResponse<CliToolConfigOutput>> {
    let conn = db.conn();

    let result = conn.query_row(
        "SELECT id, name, description, tool_type, provider_id, api_key, api_url, model, enabled, created_at, updated_at
         FROM cli_tools WHERE id = ?",
        [&id],
        |row| {
            Ok(CliTool {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                tool_type: row.get(3)?,
                provider_id: row.get(4)?,
                api_key: row.get(5)?,
                api_url: row.get(6)?,
                model: row.get(7)?,
                enabled: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        }
    );

    match result {
        Ok(tool) => {
            match tool.generate_config() {
                Ok(config) => Json(ApiResponse::success(config)),
                Err(e) => Json(ApiResponse::error(format!("Failed to generate config: {}", e))),
            }
        }
        Err(_) => Json(ApiResponse::error("CLI tool config not found")),
    }
}

/// 获取支持的 CLI 工具类型列表
async fn list_cli_tool_types() -> Json<ApiResponse<Vec<serde_json::Value>>> {
    let types = vec![
        serde_json::json!({
            "id": "claude_code",
            "name": "Claude Code",
            "description": "Claude Code CLI - Anthropic's AI coding assistant",
            "config_path": "~/.claude.json"
        }),
        serde_json::json!({
            "id": "codex",
            "name": "Codex",
            "description": "OpenAI Codex CLI",
            "config_path": "~/.codex/config.json"
        }),
        serde_json::json!({
            "id": "gemini_cli",
            "name": "Gemini CLI",
            "description": "Google Gemini CLI",
            "config_path": "~/.gemini/config.json"
        }),
        serde_json::json!({
            "id": "opencode",
            "name": "OpenCode",
            "description": "OpenCode CLI",
            "config_path": "~/.opencode/config.json"
        }),
        serde_json::json!({
            "id": "openclaw",
            "name": "OpenClaw",
            "description": "OpenClaw CLI",
            "config_path": "~/.openclaw/config.json"
        }),
    ];

    Json(ApiResponse::success(types))
}

/// 备份 CLI 工具当前配置
async fn backup_cli_tool_config(
    State(db): State<Arc<Database>>,
    Path(id): Path<String>,
) -> Json<ApiResponse<serde_json::Value>> {
    let conn = db.conn();

    // 查询配置获取 tool_type
    let tool_result = conn.query_row(
        "SELECT tool_type FROM cli_tools WHERE id = ?",
        [&id],
        |row| -> Result<String, rusqlite::Error> { row.get(0) }
    );

    // 根据 tool_type 获取配置文件路径
    let config_path_str = match tool_result {
        Ok(tool_type) => {
            match tool_type.as_str() {
                "claude_code" => "~/.claude/settings.json",
                "codex" => "~/.codex/config.json",
                "gemini_cli" => "~/.gemini/config.json",
                "opencode" => "~/.opencode/config.json",
                "openclaw" => "~/.openclaw/config.json",
                _ => "~/.claude/settings.json",
            }
        }
        Err(_) => "~/.claude/settings.json",
    };

    let expanded_path = shellexpand::tilde(config_path_str);
    let config_path = PathBuf::from(expanded_path.as_ref());

    // 检查配置文件是否存在
    if !config_path.exists() {
        return Json(ApiResponse::error(format!(
            "Config file not found: {}",
            config_path.display()
        )));
    }

    let backup_path = config_path.with_extension("json.backup");

    // 执行备份
    match std::fs::copy(&config_path, &backup_path) {
        Ok(_) => Json(ApiResponse::success(serde_json::json!({
            "message": "Configuration backed up successfully",
            "config_path": config_path.to_string_lossy(),
            "backup_path": backup_path.to_string_lossy(),
        }))),
        Err(e) => Json(ApiResponse::error(format!(
            "Failed to backup config: {}",
            e
        ))),
    }
}

/// 恢复 CLI 工具配置的备份
async fn restore_cli_tool_backup(
    State(db): State<Arc<Database>>,
    Path(id): Path<String>,
) -> Json<ApiResponse<serde_json::Value>> {
    let conn = db.conn();

    // 查询配置获取 tool_type
    let tool_result = conn.query_row(
        "SELECT tool_type FROM cli_tools WHERE id = ?",
        [&id],
        |row| -> Result<String, rusqlite::Error> { row.get(0) }
    );

    // 根据 tool_type 获取配置文件路径
    let config_path_str = match tool_result {
        Ok(tool_type) => {
            match tool_type.as_str() {
                "claude_code" => "~/.claude/settings.json",
                "codex" => "~/.codex/config.json",
                "gemini_cli" => "~/.gemini/config.json",
                "opencode" => "~/.opencode/config.json",
                "openclaw" => "~/.openclaw/config.json",
                _ => "~/.claude/settings.json",
            }
        }
        Err(_) => "~/.claude/settings.json",
    };

    let expanded_path = shellexpand::tilde(config_path_str);
    let config_path = PathBuf::from(expanded_path.as_ref());
    let backup_path = config_path.with_extension("json.backup");

    // 检查备份文件是否存在
    if !backup_path.exists() {
        return Json(ApiResponse::error(format!(
            "Backup file not found: {}",
            backup_path.display()
        )));
    }

    // 恢复备份
    match std::fs::copy(&backup_path, &config_path) {
        Ok(_) => {
            // 可选：删除备份文件
            let _ = std::fs::remove_file(&backup_path);

            Json(ApiResponse::success(serde_json::json!({
                "message": "Configuration restored successfully",
                "config_path": config_path.to_string_lossy(),
                "backup_path": backup_path.to_string_lossy(),
            })))
        }
        Err(e) => Json(ApiResponse::error(format!(
            "Failed to restore backup: {}",
            e
        ))),
    }
}

/// 应用 CLI 工具配置到本地文件系统
async fn apply_cli_tool_config(
    State(db): State<Arc<Database>>,
    Path(id): Path<String>,
) -> Json<ApiResponse<serde_json::Value>> {
    let conn = db.conn();

    // 获取配置
    let result = conn.query_row(
        "SELECT id, name, description, tool_type, provider_id, api_key, api_url, model, enabled, created_at, updated_at
         FROM cli_tools WHERE id = ?",
        [&id],
        |row| {
            Ok(CliTool {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                tool_type: row.get(3)?,
                provider_id: row.get(4)?,
                api_key: row.get(5)?,
                api_url: row.get(6)?,
                model: row.get(7)?,
                enabled: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        }
    );

    let tool = match result {
        Ok(t) => t,
        Err(_) => return Json(ApiResponse::error("CLI tool config not found")),
    };

    // 生成配置
    let config = match tool.generate_config() {
        Ok(c) => c,
        Err(e) => return Json(ApiResponse::error(format!("Failed to generate config: {}", e))),
    };

    // 展开路径中的 ~ 为用户主目录
    let expanded_path = shellexpand::tilde(&config.config_path);
    let config_path = PathBuf::from(expanded_path.as_ref());

    // 确保父目录存在
    if let Some(parent) = config_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            return Json(ApiResponse::error(format!(
                "Failed to create directory {}: {}",
                parent.display(),
                e
            )));
        }
    }

    // 备份原配置文件（如果存在）
    if config_path.exists() {
        let backup_path = config_path.with_extension("json.backup");
        if let Err(e) = std::fs::copy(&config_path, &backup_path) {
            return Json(ApiResponse::error(format!(
                "Failed to backup existing config: {}",
                e
            )));
        }
    }

    // 写入配置文件
    if let Err(e) = std::fs::write(&config_path, &config.config_content) {
        return Json(ApiResponse::error(format!(
            "Failed to write config to {}: {}",
            config_path.display(),
            e
        )));
    }

    // 写入环境变量到 shell 配置文件
    if let Some(env_vars) = &config.env_vars {
        if let Err(e) = write_env_vars_to_shell_config(env_vars) {
            tracing::warn!("Failed to write env vars to shell config: {}", e);
        }
    }

    // 更新数据库中的启用状态
    if let Err(e) = conn.execute(
        "UPDATE cli_tools SET enabled = TRUE, updated_at = ? WHERE id = ?",
        rusqlite::params![chrono::Utc::now().timestamp_millis(), &id],
    ) {
        tracing::warn!("Failed to update enabled status: {}", e);
    }

    // 禁用同类型的其他配置
    if let Err(e) = conn.execute(
        "UPDATE cli_tools SET enabled = FALSE WHERE tool_type = ? AND id != ?",
        [&tool.tool_type, &id],
    ) {
        tracing::warn!("Failed to disable other configs: {}", e);
    }

    // 获取 shell 配置文件路径用于响应
    let shell_config_path = get_shell_config_path();

    Json(ApiResponse::success(serde_json::json!({
        "message": "Configuration applied successfully",
        "config_path": config_path.to_string_lossy(),
        "backup_path": config_path.with_extension("json.backup").to_string_lossy(),
        "env_vars": config.env_vars,
        "shell_config_path": shell_config_path,
        "note": "Please run 'source ~/.bashrc' (or ~/.zshrc) to apply environment variables",
    })))
}

/// 获取 shell 配置文件路径
fn get_shell_config_path() -> String {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
    let home = dirs::home_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "~".to_string());

    if shell.contains("zsh") {
        format!("{}/.zshrc", home)
    } else {
        format!("{}/.bashrc", home)
    }
}

/// 将环境变量写入 shell 配置文件
fn write_env_vars_to_shell_config(env_vars: &[(String, String)]) -> anyhow::Result<()> {
    use std::io::Write;

    // 检测当前 shell
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Failed to get home directory"))?;

    // 根据 shell 类型选择配置文件
    let shell_config_path = if shell.contains("zsh") {
        home.join(".zshrc")
    } else if shell.contains("bash") {
        home.join(".bashrc")
    } else {
        // 默认使用 .bashrc
        home.join(".bashrc")
    };

    // 读取现有配置内容
    let existing_content = std::fs::read_to_string(&shell_config_path).unwrap_or_default();

    // 准备要添加的环境变量内容
    let mut env_vars_content = String::new();
    env_vars_content.push_str("\n# AI Gateway Hub - Environment Variables\n");
    env_vars_content.push_str("# Added automatically by AI Gateway Hub\n");

    for (key, value) in env_vars {
        // 检查该环境变量是否已存在（避免重复添加）
        let pattern = format!("export {}=", key);
        if !existing_content.contains(&pattern) {
            env_vars_content.push_str(&format!("export {}={}\n", key, value));
        } else {
            // 如果已存在，更新它
            tracing::info!("Env var {} already exists in shell config, skipping", key);
        }
    }

    // 追加到配置文件
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&shell_config_path)?;

    file.write_all(env_vars_content.as_bytes())?;

    tracing::info!(
        "Environment variables written to shell config: {}",
        shell_config_path.display()
    );

    Ok(())
}

/// 跳过 Claude Code 登录引导
async fn skip_claude_onboarding() -> Json<ApiResponse<serde_json::Value>> {
    use std::collections::HashMap;

    // 固定使用 ~/.claude.json 路径（Docker 挂载到宿主机）
    let config_path = PathBuf::from("/root/.claude.json");

    // 读取现有配置（如果存在）
    let mut config: HashMap<String, serde_json::Value> = match std::fs::read_to_string(&config_path) {
        Ok(content) => {
            serde_json::from_str(&content).unwrap_or_default()
        }
        Err(_) => HashMap::new(),
    };

    // 设置 hasCompletedOnboarding 为 true
    config.insert("hasCompletedOnboarding".to_string(), serde_json::json!(true));

    // 写入配置文件
    let config_content = match serde_json::to_string_pretty(&config) {
        Ok(content) => content,
        Err(e) => return Json(ApiResponse::error(format!("Failed to serialize config: {}", e))),
    };

    match std::fs::write(&config_path, config_content) {
        Ok(_) => {
            tracing::info!("Successfully wrote hasCompletedOnboarding to {:?}", config_path);
            Json(ApiResponse::success(serde_json::json!({
                "message": "Claude Code onboarding skipped successfully",
                "config_path": config_path.to_string_lossy(),
                "os": std::env::consts::OS,
            })))
        }
        Err(e) => {
            tracing::error!("Failed to write config: {}", e);
            Json(ApiResponse::error(format!(
                "Failed to write config to {}: {}",
                config_path.display(),
                e
            )))
        }
    }
}

/// 获取 Claude Code 配置文件路径（根据操作系统）
fn get_claude_config_path() -> PathBuf {
    let home_dir = dirs::home_dir().expect("Failed to get home directory");

    match std::env::consts::OS {
        "windows" => {
            // Windows: %APPDATA%\Claude\settings.json
            let app_data = std::env::var("APPDATA")
                .map(PathBuf::from)
                .unwrap_or_else(|_| home_dir.join("AppData").join("Roaming"));
            app_data.join("Claude").join("settings.json")
        }
        "macos" => {
            // macOS: ~/Library/Application Support/Claude/settings.json
            home_dir
                .join("Library")
                .join("Application Support")
                .join("Claude")
                .join("settings.json")
        }
        _ => {
            // Linux 和其他系统: ~/.claude.json
            home_dir.join(".claude.json")
        }
    }
}
