use diesel::r2d2::{ConnectionManager, Pool};
use diesel::SqliteConnection;

use crate::db::ports::DbExecutor;

pub struct DieselSqliteExecutor {
    pool: Pool<ConnectionManager<SqliteConnection>>,
}

impl DieselSqliteExecutor {
    pub fn new(pool: Pool<ConnectionManager<SqliteConnection>>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl DbExecutor for DieselSqliteExecutor {
    async fn run(
        &self,
        f: Box<
            dyn FnOnce(&mut SqliteConnection) -> anyhow::Result<()>
                + Send
        >,
    ) -> anyhow::Result<()> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get()?;
            f(&mut conn)
        })
        .await?
    }
}

