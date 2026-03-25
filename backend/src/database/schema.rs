//! 数据库表结构定义

use anyhow::Result;
use rusqlite::Connection;
use tracing::info;

/// 当前 Schema 版本
const SCHEMA_VERSION: i32 = 2;

/// 初始化数据库表结构
pub fn init_schema(conn: &Connection) -> Result<()> {
    // 创建 schema_version 表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY
        )",
        [],
    )?;

    // 获取当前版本
    let current_version: i32 = conn
        .query_row(
            "SELECT version FROM schema_version ORDER BY version DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if current_version < SCHEMA_VERSION {
        info!("执行数据库迁移: {} -> {}", current_version, SCHEMA_VERSION);
        run_migrations(conn, current_version)?;
    }

    Ok(())
}

/// 执行数据库迁移
fn run_migrations(conn: &Connection, from_version: i32) -> Result<()> {
    if from_version < 1 {
        // 初始化 v1 表结构
        create_v1_tables(conn)?;
    }

    if from_version < 2 {
        // v2: 添加 CLI 工具配置表
        create_cli_tools_table(conn)?;
    }

    // 更新版本号
    conn.execute(
        "INSERT OR REPLACE INTO schema_version (version) VALUES (?)",
        [SCHEMA_VERSION],
    )?;

    Ok(())
}

/// 创建 v1 表结构
fn create_v1_tables(conn: &Connection) -> Result<()> {
    // 供应商表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS providers (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            provider_type TEXT NOT NULL,
            base_url TEXT NOT NULL,
            api_key TEXT NOT NULL,
            models TEXT NOT NULL, -- JSON 数组
            config TEXT, -- JSON 对象，存储额外配置
            category TEXT,
            is_default BOOLEAN DEFAULT FALSE,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )",
        [],
    )?;

    // MCP 服务器表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS mcp_servers (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            server_type TEXT NOT NULL, -- 'stdio' | 'http' | 'sse'
            command TEXT,
            args TEXT, -- JSON 数组
            env TEXT, -- JSON 对象
            url TEXT,
            headers TEXT, -- JSON 对象
            enabled BOOLEAN DEFAULT TRUE,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )",
        [],
    )?;

    // 用量统计表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS usage_stats (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            provider_id TEXT NOT NULL,
            model TEXT,
            request_type TEXT NOT NULL,
            input_tokens INTEGER DEFAULT 0,
            output_tokens INTEGER DEFAULT 0,
            total_tokens INTEGER DEFAULT 0,
            cost REAL DEFAULT 0.0,
            latency_ms INTEGER,
            success BOOLEAN DEFAULT TRUE,
            error_message TEXT,
            created_at INTEGER NOT NULL,
            FOREIGN KEY (provider_id) REFERENCES providers(id)
        )",
        [],
    )?;

    // 代理配置表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS proxy_configs (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            enabled BOOLEAN DEFAULT FALSE,
            listen_address TEXT DEFAULT '0.0.0.0',
            listen_port INTEGER DEFAULT 8080,
            enable_auth BOOLEAN DEFAULT TRUE,
            enable_logging BOOLEAN DEFAULT TRUE,
            config TEXT -- JSON 对象
        )",
        [],
    )?;

    // 插入默认代理配置
    conn.execute(
        "INSERT OR IGNORE INTO proxy_configs (id, enabled, listen_address, listen_port)
         VALUES (1, FALSE, '0.0.0.0', 8080)",
        [],
    )?;

    // 创建索引
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_providers_type ON providers(provider_type)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_usage_provider ON usage_stats(provider_id)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_usage_created ON usage_stats(created_at)",
        [],
    )?;

    info!("✓ v1 表结构创建完成");

    Ok(())
}

/// 创建 CLI 工具配置表 (v2)
fn create_cli_tools_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS cli_tools (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            tool_type TEXT NOT NULL, -- 'claude_code' | 'codex' | 'gemini_cli' | 'opencode' | 'openclaw'
            provider_id TEXT NOT NULL,
            api_key TEXT NOT NULL,
            api_url TEXT NOT NULL,
            model TEXT NOT NULL,
            enabled BOOLEAN DEFAULT TRUE,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            FOREIGN KEY (provider_id) REFERENCES providers(id)
        )",
        [],
    )?;

    // 创建索引
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_cli_tools_type ON cli_tools(tool_type)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_cli_tools_provider ON cli_tools(provider_id)",
        [],
    )?;

    info!("✓ v2 CLI 工具表创建完成");

    Ok(())
}
