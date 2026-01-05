use anyhow::Result;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use log::info;

/// Embed all diesel migrations at compile time
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// Type alias for SQLite connection pool
pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;

/// Create database connection pool and run migrations
///
/// This function should be called **once at application startup**.
///
/// Responsibilities:
/// - Build r2d2 connection pool
/// - Automatically run all pending Diesel migrations
///
/// # Panics
/// - If database connection fails
/// - If migrations fail
pub fn init_db_pool(database_url: &str) -> Result<DbPool> {
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);

    let pool = Pool::builder()
        .build(manager)
        .expect("Failed to create database pool");

    run_migrations(&pool)?;

    Ok(pool)
}

/// Run embedded Diesel migrations
fn run_migrations(pool: &DbPool) -> Result<()> {
    let mut conn = pool.get()?;

    info!("Running database migrations...");
    conn.run_pending_migrations(MIGRATIONS)
        .map_err(|e| anyhow::anyhow!("Migration failed: {}", e))?;
    info!("Database migrations completed");

    Ok(())
}
