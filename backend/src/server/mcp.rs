//! MCP API 路由

use axum::{
    extract::{Path, State},
    routing::{get, post, put, delete},
    Json, Router,
};
use std::sync::Arc;

use crate::database::Database;
use crate::models::mcp::*;
use crate::models::{ApiResponse, CreateMcpServerRequest, UpdateMcpServerRequest, PaginationParams, PaginatedResponse};

/// MCP 路由
pub fn mcp_routes() -> Router<Arc<Database>> {
    Router::new()
        .route("/", get(list_mcp_servers).post(create_mcp_server))
        .route("/:id", get(get_mcp_server).put(update_mcp_server).delete(delete_mcp_server))
        .route("/:id/toggle", post(toggle_mcp_server))
}

/// 获取 MCP 服务器列表
async fn list_mcp_servers(
    State(db): State<Arc<Database>>,
    axum::extract::Query(params): axum::extract::Query<PaginationParams>,
) -> Json<ApiResponse<PaginatedResponse<McpServer>>> {
    let conn = db.conn();

    let total: i64 = conn
        .query_row("SELECT COUNT(*) FROM mcp_servers", [], |row| row.get(0))
        .unwrap_or(0);

    let offset = (params.page - 1) * params.per_page;
    let mut stmt = conn.prepare(
        "SELECT id, name, description, server_type, command, args, env, url, headers, enabled, created_at, updated_at
         FROM mcp_servers
         ORDER BY created_at DESC
         LIMIT ? OFFSET ?"
    ).unwrap();

    let servers: Vec<McpServer> = stmt
        .query_map([params.per_page, offset], |row| {
            Ok(McpServer {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                server_type: row.get(3)?,
                command: row.get(4)?,
                args: row.get(5)?,
                env: row.get(6)?,
                url: row.get(7)?,
                headers: row.get(8)?,
                enabled: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    let response = PaginatedResponse::new(servers, total, params.page, params.per_page);
    Json(ApiResponse::success(response))
}

/// 获取单个 MCP 服务器
async fn get_mcp_server(
    State(db): State<Arc<Database>>,
    Path(id): Path<String>,
) -> Json<ApiResponse<McpServer>> {
    let conn = db.conn();

    let result = conn.query_row(
        "SELECT id, name, description, server_type, command, args, env, url, headers, enabled, created_at, updated_at
         FROM mcp_servers WHERE id = ?",
        [&id],
        |row| {
            Ok(McpServer {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                server_type: row.get(3)?,
                command: row.get(4)?,
                args: row.get(5)?,
                env: row.get(6)?,
                url: row.get(7)?,
                headers: row.get(8)?,
                enabled: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        }
    );

    match result {
        Ok(server) => Json(ApiResponse::success(server)),
        Err(_) => Json(ApiResponse::error("MCP server not found")),
    }
}

/// 创建 MCP 服务器
async fn create_mcp_server(
    State(db): State<Arc<Database>>,
    Json(req): Json<CreateMcpServerRequest>,
) -> Json<ApiResponse<McpServer>> {
    let server = match req.server_type.as_str() {
        "stdio" => {
            let command = req.command.ok_or_else(|| "Command is required for stdio type").unwrap();
            let args = req.args.unwrap_or_default();
            match McpServer::new_stdio(req.name, req.description, command, args, req.env) {
                Ok(s) => s,
                Err(e) => return Json(ApiResponse::error(format!("Failed to create MCP server: {}", e))),
            }
        }
        "http" | "sse" => {
            let url = req.url.ok_or_else(|| "URL is required for http/sse type").unwrap();
            match McpServer::new_http(req.name, req.description, url, req.headers, req.server_type == "sse") {
                Ok(s) => s,
                Err(e) => return Json(ApiResponse::error(format!("Failed to create MCP server: {}", e))),
            }
        }
        _ => return Json(ApiResponse::error("Invalid server type")),
    };

    let conn = db.conn();
    let result = conn.execute(
        "INSERT INTO mcp_servers (id, name, description, server_type, command, args, env, url, headers, enabled, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        rusqlite::params![
            server.id,
            server.name,
            server.description,
            server.server_type,
            server.command,
            server.args,
            server.env,
            server.url,
            server.headers,
            server.enabled,
            server.created_at,
            server.updated_at,
        ],
    );

    match result {
        Ok(_) => Json(ApiResponse::success(server)),
        Err(e) => Json(ApiResponse::error(format!("Failed to create MCP server: {}", e))),
    }
}

/// 更新 MCP 服务器
async fn update_mcp_server(
    State(db): State<Arc<Database>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateMcpServerRequest>,
) -> Json<ApiResponse<McpServer>> {
    let conn = db.conn();

    // 先获取当前 MCP 服务器
    let current: Result<McpServer, _> = conn.query_row(
        "SELECT id, name, description, server_type, command, args, env, url, headers, enabled, created_at, updated_at
         FROM mcp_servers WHERE id = ?",
        [&id],
        |row| {
            Ok(McpServer {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                server_type: row.get(3)?,
                command: row.get(4)?,
                args: row.get(5)?,
                env: row.get(6)?,
                url: row.get(7)?,
                headers: row.get(8)?,
                enabled: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        }
    );

    let mut server = match current {
        Ok(s) => s,
        Err(_) => return Json(ApiResponse::error("MCP server not found")),
    };

    // 更新字段
    if let Some(name) = req.name {
        server.name = name;
    }
    if let Some(description) = req.description {
        server.description = Some(description);
    }
    if let Some(server_type) = req.server_type {
        server.server_type = server_type;
    }
    if let Some(command) = req.command {
        server.command = Some(command);
    }
    if let Some(args) = req.args {
        match serde_json::to_string(&args) {
            Ok(args_json) => server.args = Some(args_json),
            Err(e) => return Json(ApiResponse::error(format!("Failed to serialize args: {}", e))),
        }
    }
    if let Some(env) = req.env {
        match serde_json::to_string(&env) {
            Ok(env_json) => server.env = Some(env_json),
            Err(e) => return Json(ApiResponse::error(format!("Failed to serialize env: {}", e))),
        }
    }
    if let Some(url) = req.url {
        server.url = Some(url);
    }
    if let Some(headers) = req.headers {
        match serde_json::to_string(&headers) {
            Ok(headers_json) => server.headers = Some(headers_json),
            Err(e) => return Json(ApiResponse::error(format!("Failed to serialize headers: {}", e))),
        }
    }
    if let Some(enabled) = req.enabled {
        server.enabled = enabled;
    }

    // 更新时间戳
    server.updated_at = chrono::Utc::now().timestamp_millis();

    // 执行更新
    let result = conn.execute(
        "UPDATE mcp_servers SET name = ?1, description = ?2, server_type = ?3, command = ?4,
         args = ?5, env = ?6, url = ?7, headers = ?8, enabled = ?9, updated_at = ?10 WHERE id = ?11",
        rusqlite::params![
            &server.name,
            &server.description,
            &server.server_type,
            &server.command,
            &server.args,
            &server.env,
            &server.url,
            &server.headers,
            &server.enabled,
            &server.updated_at,
            &id,
        ],
    );

    match result {
        Ok(_) => Json(ApiResponse::success(server)),
        Err(e) => Json(ApiResponse::error(format!("Failed to update MCP server: {}", e))),
    }
}

/// 删除 MCP 服务器
async fn delete_mcp_server(
    State(db): State<Arc<Database>>,
    Path(id): Path<String>,
) -> Json<ApiResponse<()>> {
    let conn = db.conn();
    let result = conn.execute("DELETE FROM mcp_servers WHERE id = ?", [&id]);

    match result {
        Ok(rows) if rows > 0 => Json(ApiResponse::success(())),
        Ok(_) => Json(ApiResponse::error("MCP server not found")),
        Err(e) => Json(ApiResponse::error(format!("Failed to delete MCP server: {}", e))),
    }
}

/// 切换 MCP 服务器启用状态
async fn toggle_mcp_server(
    State(db): State<Arc<Database>>,
    Path(id): Path<String>,
) -> Json<ApiResponse<()>> {
    let conn = db.conn();

    let result = conn.execute(
        "UPDATE mcp_servers SET enabled = NOT enabled, updated_at = ? WHERE id = ?",
        rusqlite::params![chrono::Utc::now().timestamp_millis(), id],
    );

    match result {
        Ok(rows) if rows > 0 => Json(ApiResponse::success(())),
        Ok(_) => Json(ApiResponse::error("MCP server not found")),
        Err(e) => Json(ApiResponse::error(format!("Failed to toggle MCP server: {}", e))),
    }
}
