use diesel::sqlite::SqliteConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use anyhow::{Result, anyhow};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub fn run_migrations(connection: &mut SqliteConnection) -> Result<()> {
    connection
        .run_pending_migrations(MIGRATIONS)
        .map(|_| ()) // 忽略成功时返回的 Vec<MigrationVersion>
        .map_err(|e| anyhow!("Failed to run database migrations: {}", e))
}
