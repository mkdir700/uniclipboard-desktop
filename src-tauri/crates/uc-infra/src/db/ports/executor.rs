use diesel::SqliteConnection;

pub trait DbExecutor: Send + Sync {
    fn run<T>(
        &self,
        f: impl FnOnce(&mut SqliteConnection) -> anyhow::Result<T>,
    ) -> anyhow::Result<T>;
}
