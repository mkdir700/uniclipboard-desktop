use crate::db::models::blob::NewBlobRow;
use crate::db::ports::DbExecutor;
use crate::db::ports::Mapper;
use crate::db::schema::blob;
use anyhow::Result;
use diesel::RunQueryDsl;
use uc_core::ports::BlobRepositoryPort;
use uc_core::Blob;
use uc_core::ContentHash;

pub struct DieselBlobRepository<E, M>
where
    E: DbExecutor,
    M: Mapper<Blob, NewBlobRow>,
{
    executor: E,
    mapper: M,
}

impl<E, M> DieselBlobRepository<E, M>
where
    E: DbExecutor,
    M: Mapper<Blob, NewBlobRow>,
{
    pub fn new(executor: E, mapper: M) -> Self {
        Self { executor, mapper }
    }
}

#[async_trait::async_trait]
impl<E, M> BlobRepositoryPort for DieselBlobRepository<E, M>
where
    E: DbExecutor,
    M: Mapper<Blob, NewBlobRow>,
{
    async fn insert_blob(&self, blob_row: &Blob) -> Result<()> {
        let new_blob_row = self.mapper.to_row(blob_row);
        self.executor.run(|conn| {
            diesel::insert_into(blob::table)
                .values(&new_blob_row)
                .execute(conn)?;
            Ok(())
        })
    }

    async fn find_by_hash(&self, content_hash: &ContentHash) -> Result<Option<Blob>> {
        todo!()
    }
}
