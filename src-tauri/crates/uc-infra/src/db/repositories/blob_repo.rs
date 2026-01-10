use crate::db::schema::blob;

pub struct DieselBlobRepository {
    executor: DieselSqliteExecutor,
}

impl DieselBlobRepository {
    pub fn new(executor: DieselSqliteExecutor) -> Self {
        Self { executor }
    }
}

#[async_trait::async_trait]
impl BlobRepository for DieselBlobRepository {
    async fn insert_blob(&self, blob_row: NewBlob) -> Result<()> {
        self.executor.with_conn(|conn| {
            diesel::insert_into(blob::table)
                .values(&blob_row)
                .execute(conn)?;
            Ok(())
        })
    }
}
