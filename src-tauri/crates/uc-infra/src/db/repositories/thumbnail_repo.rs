use crate::db::mappers::thumbnail_mapper::ThumbnailRowMapper;
use crate::db::models::clipboard_representation_thumbnail::ClipboardRepresentationThumbnailRow;
use crate::db::ports::{DbExecutor, InsertMapper, RowMapper};
use crate::db::schema::clipboard_representation_thumbnail;
use anyhow::Result;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use uc_core::clipboard::ThumbnailMetadata;
use uc_core::ids::RepresentationId;
use uc_core::ports::clipboard::ThumbnailRepositoryPort;

pub struct DieselThumbnailRepository<E>
where
    E: DbExecutor,
{
    executor: E,
}

impl<E> DieselThumbnailRepository<E>
where
    E: DbExecutor,
{
    pub fn new(executor: E) -> Self {
        Self { executor }
    }
}

#[async_trait::async_trait]
impl<E> ThumbnailRepositoryPort for DieselThumbnailRepository<E>
where
    E: DbExecutor,
{
    async fn get_by_representation_id(
        &self,
        representation_id: &RepresentationId,
    ) -> Result<Option<ThumbnailMetadata>> {
        let rep_id_str = representation_id.to_string();
        let row: Option<ClipboardRepresentationThumbnailRow> = self.executor.run(|conn| {
            let result: Result<Option<ClipboardRepresentationThumbnailRow>, diesel::result::Error> =
                clipboard_representation_thumbnail::table
                    .filter(clipboard_representation_thumbnail::representation_id.eq(&rep_id_str))
                    .first::<ClipboardRepresentationThumbnailRow>(conn)
                    .optional();
            result.map_err(|e| anyhow::anyhow!("Database error: {}", e))
        })?;

        match row {
            Some(row) => {
                let mapper = ThumbnailRowMapper;
                let metadata = mapper.to_domain(&row)?;
                Ok(Some(metadata))
            }
            None => Ok(None),
        }
    }

    async fn insert_thumbnail(&self, metadata: &ThumbnailMetadata) -> Result<()> {
        let mapper = ThumbnailRowMapper;
        let new_row = mapper.to_row(metadata)?;
        self.executor.run(|conn| {
            diesel::insert_into(clipboard_representation_thumbnail::table)
                .values(&new_row)
                .execute(conn)?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use uc_core::clipboard::{ThumbnailMetadata, TimestampMs};
    use uc_core::ids::RepresentationId;
    use uc_core::ports::ThumbnailRepositoryPort;
    use uc_core::BlobId;
    use uc_core::MimeType;

    use crate::db::executor::DieselSqliteExecutor;
    use crate::db::pool::init_db_pool;

    use super::DieselThumbnailRepository;

    #[tokio::test]
    async fn test_thumbnail_repo_insert_and_get() {
        let pool = init_db_pool(":memory:").unwrap();
        let executor = DieselSqliteExecutor::new(pool);
        let repo = DieselThumbnailRepository::new(executor);

        let metadata = ThumbnailMetadata::new(
            RepresentationId::new(),
            BlobId::new(),
            MimeType("image/webp".to_string()),
            640,
            480,
            1234,
            Some(TimestampMs::from_epoch_millis(1)),
        );

        repo.insert_thumbnail(&metadata).await.unwrap();
        let fetched = repo
            .get_by_representation_id(&metadata.representation_id)
            .await
            .unwrap();
        let fetched = fetched.expect("expected thumbnail metadata");
        assert_eq!(fetched.representation_id, metadata.representation_id);
        assert_eq!(fetched.thumbnail_blob_id, metadata.thumbnail_blob_id);
        assert_eq!(fetched.thumbnail_mime_type, metadata.thumbnail_mime_type);
        assert_eq!(fetched.width, metadata.width);
        assert_eq!(fetched.height, metadata.height);
        assert_eq!(fetched.size_bytes, metadata.size_bytes);
        assert_eq!(fetched.created_at_ms, metadata.created_at_ms);
    }
}
