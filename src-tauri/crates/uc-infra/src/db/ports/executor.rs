use diesel::SqliteConnection;

#[async_trait::async_trait]
pub trait DbExecutor: Send + Sync {
    async fn run<T>(
        &self,
        f: Box<
            dyn FnOnce(&mut SqliteConnection) -> anyhow::Result<T>
                + Send
        >,
    ) -> anyhow::Result<T>;
}

