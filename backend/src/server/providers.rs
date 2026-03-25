//! Provider API 路由

use axum::{
    extract::{Path, State},
    routing::{get, post, put, delete},
    Json, Router,
};
use std::sync::Arc;

use crate::database::Database;
use crate::models::provider::*;
use crate::models::{ApiResponse, CreateProviderRequest, UpdateProviderRequest, PaginationParams, PaginatedResponse};

/// Provider 路由
pub fn provider_routes() -> Router<Arc<Database>> {
    Router::new()
        .route("/", get(list_providers).post(create_provider))
        .route("/:id", get(get_provider).put(update_provider).delete(delete_provider))
        .route("/:id/default", post(set_default_provider))
        .route("/types", get(list_provider_types))
}

/// 获取 Provider 列表
async fn list_providers(
    State(db): State<Arc<Database>>,
    axum::extract::Query(params): axum::extract::Query<PaginationParams>,
) -> Json<ApiResponse<PaginatedResponse<Provider>>> {
    let conn = db.conn();

    // 查询总数
    let total: i64 = conn
        .query_row("SELECT COUNT(*) FROM providers", [], |row| row.get(0))
        .unwrap_or(0);

    // 查询分页数据
    let offset = (params.page - 1) * params.per_page;
    let mut stmt = conn.prepare(
        "SELECT id, name, provider_type, base_url, api_key, models, config, category, is_default, created_at, updated_at
         FROM providers
         ORDER BY created_at DESC
         LIMIT ? OFFSET ?"
    ).unwrap();

    let providers: Vec<Provider> = stmt
        .query_map([params.per_page, offset], |row| {
            Ok(Provider {
                id: row.get(0)?,
                name: row.get(1)?,
                provider_type: row.get(2)?,
                base_url: row.get(3)?,
                api_key: row.get(4)?,
                models: row.get(5)?,
                config: row.get(6)?,
                category: row.get(7)?,
                is_default: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect();

    let response = PaginatedResponse::new(providers, total, params.page, params.per_page);
    Json(ApiResponse::success(response))
}

/// 获取单个 Provider
async fn get_provider(
    State(db): State<Arc<Database>>,
    Path(id): Path<String>,
) -> Json<ApiResponse<Provider>> {
    let conn = db.conn();

    let result = conn.query_row(
        "SELECT id, name, provider_type, base_url, api_key, models, config, category, is_default, created_at, updated_at
         FROM providers WHERE id = ?",
        [&id],
        |row| {
            Ok(Provider {
                id: row.get(0)?,
                name: row.get(1)?,
                provider_type: row.get(2)?,
                base_url: row.get(3)?,
                api_key: row.get(4)?,
                models: row.get(5)?,
                config: row.get(6)?,
                category: row.get(7)?,
                is_default: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        }
    );

    match result {
        Ok(provider) => Json(ApiResponse::success(provider)),
        Err(_) => Json(ApiResponse::error("Provider not found")),
    }
}

/// 创建 Provider
async fn create_provider(
    State(db): State<Arc<Database>>,
    Json(req): Json<CreateProviderRequest>,
) -> Json<ApiResponse<Provider>> {
    let provider = match Provider::new(
        req.name,
        req.provider_type,
        req.base_url,
        req.api_key,
        req.models,
    ) {
        Ok(p) => p,
        Err(e) => return Json(ApiResponse::error(format!("Failed to create provider: {}", e))),
    };

    let conn = db.conn();
    let result = conn.execute(
        "INSERT INTO providers (id, name, provider_type, base_url, api_key, models, config, category, is_default, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        rusqlite::params![
            provider.id,
            provider.name,
            provider.provider_type,
            provider.base_url,
            provider.api_key,
            provider.models,
            provider.config,
            provider.category,
            provider.is_default,
            provider.created_at,
            provider.updated_at,
        ],
    );

    match result {
        Ok(_) => Json(ApiResponse::success(provider)),
        Err(e) => Json(ApiResponse::error(format!("Failed to create provider: {}", e))),
    }
}

/// 更新 Provider
async fn update_provider(
    State(db): State<Arc<Database>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateProviderRequest>,
) -> Json<ApiResponse<Provider>> {
    let conn = db.conn();

    // 先获取当前 Provider
    let current: Result<Provider, _> = conn.query_row(
        "SELECT id, name, provider_type, base_url, api_key, models, config, category, is_default, created_at, updated_at
         FROM providers WHERE id = ?",
        [&id],
        |row| {
            Ok(Provider {
                id: row.get(0)?,
                name: row.get(1)?,
                provider_type: row.get(2)?,
                base_url: row.get(3)?,
                api_key: row.get(4)?,
                models: row.get(5)?,
                config: row.get(6)?,
                category: row.get(7)?,
                is_default: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        }
    );

    let mut provider = match current {
        Ok(p) => p,
        Err(_) => return Json(ApiResponse::error("Provider not found")),
    };

    // 更新字段
    if let Some(name) = req.name {
        provider.name = name;
    }
    if let Some(base_url) = req.base_url {
        provider.base_url = base_url;
    }
    if let Some(api_key) = req.api_key {
        provider.api_key = api_key;
    }
    if let Some(models) = req.models {
        match serde_json::to_string(&models) {
            Ok(models_json) => provider.models = models_json,
            Err(e) => return Json(ApiResponse::error(format!("Failed to serialize models: {}", e))),
        }
    }
    if let Some(config) = req.config {
        match serde_json::to_string(&config) {
            Ok(config_json) => provider.config = Some(config_json),
            Err(e) => return Json(ApiResponse::error(format!("Failed to serialize config: {}", e))),
        }
    }
    if let Some(category) = req.category {
        provider.category = Some(category);
    }
    if let Some(is_default) = req.is_default {
        provider.is_default = is_default;
    }

    // 更新时间戳
    provider.updated_at = chrono::Utc::now().timestamp_millis();

    // 执行更新
    let result = conn.execute(
        "UPDATE providers SET name = ?1, base_url = ?2, api_key = ?3, models = ?4,
         config = ?5, category = ?6, is_default = ?7, updated_at = ?8 WHERE id = ?9",
        rusqlite::params![
            &provider.name,
            &provider.base_url,
            &provider.api_key,
            &provider.models,
            &provider.config,
            &provider.category,
            &provider.is_default,
            &provider.updated_at,
            &id,
        ],
    );

    match result {
        Ok(_) => Json(ApiResponse::success(provider)),
        Err(e) => Json(ApiResponse::error(format!("Failed to update provider: {}", e))),
    }
}

/// 删除 Provider
async fn delete_provider(
    State(db): State<Arc<Database>>,
    Path(id): Path<String>,
) -> Json<ApiResponse<()>> {
    let conn = db.conn();
    let result = conn.execute("DELETE FROM providers WHERE id = ?", [&id]);

    match result {
        Ok(rows) if rows > 0 => Json(ApiResponse::success(())),
        Ok(_) => Json(ApiResponse::error("Provider not found")),
        Err(e) => Json(ApiResponse::error(format!("Failed to delete provider: {}", e))),
    }
}

/// 设置默认 Provider
async fn set_default_provider(
    State(db): State<Arc<Database>>,
    Path(id): Path<String>,
) -> Json<ApiResponse<()>> {
    let conn = db.conn();

    // 先将所有 provider 设为非默认
    if let Err(e) = conn.execute("UPDATE providers SET is_default = FALSE", []) {
        return Json(ApiResponse::error(format!("Failed to update providers: {}", e)));
    }

    // 再设置指定的 provider 为默认
    match conn.execute("UPDATE providers SET is_default = TRUE WHERE id = ?", [&id]) {
        Ok(rows) if rows > 0 => Json(ApiResponse::success(())),
        Ok(_) => Json(ApiResponse::error("Provider not found")),
        Err(e) => Json(ApiResponse::error(format!("Failed to set default provider: {}", e))),
    }
}

/// 获取支持的 Provider 类型列表
async fn list_provider_types() -> Json<ApiResponse<Vec<serde_json::Value>>> {
    let types = vec![
        serde_json::json!({
            "id": "openai",
            "name": "OpenAI",
            "description": "OpenAI API (GPT-4, GPT-3.5, etc.)"
        }),
        serde_json::json!({
            "id": "anthropic",
            "name": "Anthropic",
            "description": "Anthropic API (Claude, etc.)"
        }),
        serde_json::json!({
            "id": "gemini",
            "name": "Google Gemini",
            "description": "Google Gemini API"
        }),
        serde_json::json!({
            "id": "azure",
            "name": "Azure OpenAI",
            "description": "Microsoft Azure OpenAI Service"
        }),
        serde_json::json!({
            "id": "ollama",
            "name": "Ollama",
            "description": "Local Ollama instance"
        }),
        serde_json::json!({
            "id": "custom",
            "name": "Custom",
            "description": "Custom OpenAI-compatible API"
        }),
    ];

    Json(ApiResponse::success(types))
}
