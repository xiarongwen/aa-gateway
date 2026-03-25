//! 数据库模块 - SQLite 数据持久化

use anyhow::Result;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing::info;

mod schema;

pub use schema::*;

/// 数据库连接封装
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// 初始化数据库连接并创建表
    pub async fn init() -> Result<Self> {
        let db_path = Self::get_db_path()?;

        // 确保父目录存在
        if let Some(parent) = db_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let conn = Connection::open(&db_path)?;

        // 初始化表结构
        schema::init_schema(&conn)?;

        info!("数据库初始化完成: {:?}", db_path);

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// 获取数据库文件路径
    fn get_db_path() -> Result<PathBuf> {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("./data"))
            .join("ai-gateway");
        Ok(data_dir.join("ai-gateway.db"))
    }

    /// 获取数据库连接
    pub fn conn(&self) -> std::sync::MutexGuard<Connection> {
        self.conn.lock().expect("获取数据库锁失败")
    }
}

// 安全地在多线程间共享
unsafe impl Send for Database {}
unsafe impl Sync for Database {}
