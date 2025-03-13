use anyhow::{anyhow, Result};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use once_cell::sync::Lazy;
use std::env;

use crate::config::get_config_dir;
use crate::migrations::run_migrations;

// 定义连接池类型
pub type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

// 定义一个用于管理数据库连接池的结构体
pub struct DbManager {
    pub pool: DbPool,
}

impl DbManager {
    // 创建一个新的 DbManager 实例
    pub fn new() -> Result<Self, diesel::r2d2::PoolError> {
        // 从环境变量中获取数据库 URL, 如果环境变量中没有设置 DATABASE_URL，则使用默认的配置目录
        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
            let config_dir = get_config_dir().unwrap();
            config_dir
                .join("uniclipboard.db")
                .to_str()
                .unwrap()
                .to_string()
        });

        // 创建连接管理器
        let manager = ConnectionManager::<SqliteConnection>::new(database_url);

        // 创建连接池
        let pool = r2d2::Pool::builder().build(manager)?;

        Ok(DbManager { pool })
    }

    pub fn init(&self) -> Result<()> {
        self.run_migrations()
    }

    // 运行数据库迁移
    pub fn run_migrations(&self) -> Result<()> {
        let mut conn = self
            .get_connection()
            .map_err(|e| anyhow!("Failed to get connection: {}", e))?;
        run_migrations(&mut conn).map_err(|e| anyhow!("Failed to run migrations: {}", e))
    }

    // 获取数据库连接
    pub fn get_connection(
        &self,
    ) -> Result<r2d2::PooledConnection<ConnectionManager<SqliteConnection>>, diesel::r2d2::PoolError>
    {
        self.pool.get()
    }
}

// 创建一个全局的数据库连接池
pub static DB_POOL: Lazy<DbManager> =
    Lazy::new(|| DbManager::new().expect("Failed to create DB manager"));
