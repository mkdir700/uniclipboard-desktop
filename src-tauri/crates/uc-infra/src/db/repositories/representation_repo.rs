//! Clipboard Representation Repository
//!
//! Implements [`ClipboardRepresentationRepositoryPort`] for querying and updating
//! clipboard snapshot representations stored in SQLite.
//!
//! # Usage
//!
//! ```rust
//! use uc_infra::db::repositories::DieselClipboardRepresentationRepository;
//!
//! let repo = DieselClipboardRepresentationRepository::new(executor);
//!
//! // Query a representation
//! let rep = repo.get_representation(&event_id, &rep_id).await?;
//!
//! // Update blob_id after materialization
//! repo.update_blob_id(&rep_id, &blob_id).await?;
//! ```

use crate::db::models::snapshot_representation::SnapshotRepresentationRow;
use crate::db::mappers::snapshot_representation_mapper::RepresentationRowMapper;
use crate::db::ports::{DbExecutor, RowMapper};
use crate::db::schema::clipboard_snapshot_representation;
use anyhow::Result;
use diesel::{BoolExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, ExpressionMethods};
use uc_core::clipboard::PersistedClipboardRepresentation;
use uc_core::ids::{EventId, RepresentationId};
use uc_core::ports::clipboard::ClipboardRepresentationRepositoryPort;
use uc_core::BlobId;

pub struct DieselClipboardRepresentationRepository<E>
where
    E: DbExecutor,
{
    executor: E,
}

impl<E> DieselClipboardRepresentationRepository<E>
where
    E: DbExecutor,
{
    pub fn new(executor: E) -> Self {
        Self { executor }
    }
}

#[async_trait::async_trait]
impl<E> ClipboardRepresentationRepositoryPort for DieselClipboardRepresentationRepository<E>
where
    E: DbExecutor,
{
    async fn get_representation(
        &self,
        event_id: &EventId,
        representation_id: &RepresentationId,
    ) -> Result<Option<PersistedClipboardRepresentation>> {
        let event_id_str = event_id.to_string();
        let rep_id_str = representation_id.to_string();

        let row: Option<SnapshotRepresentationRow> = self.executor.run(|conn| {
            let result: Result<Option<SnapshotRepresentationRow>, diesel::result::Error> =
                clipboard_snapshot_representation::table
                    .filter(
                        clipboard_snapshot_representation::event_id
                            .eq(&event_id_str)
                            .and(clipboard_snapshot_representation::id.eq(&rep_id_str)),
                    )
                    .first::<SnapshotRepresentationRow>(conn)
                    .optional();
            result.map_err(|e| anyhow::anyhow!("Database error: {}", e))
        })?;

        match row {
            Some(r) => {
                let mapper = RepresentationRowMapper;
                let rep = mapper.to_domain(&r)?;
                Ok(Some(rep))
            }
            None => Ok(None),
        }
    }

    async fn update_blob_id(
        &self,
        representation_id: &RepresentationId,
        blob_id: &BlobId,
    ) -> Result<()> {
        let rep_id_str = representation_id.to_string();
        let blob_id_str = blob_id.to_string();

        self.executor.run(|conn| {
            diesel::update(
                clipboard_snapshot_representation::table
                    .filter(clipboard_snapshot_representation::id.eq(&rep_id_str)),
            )
            .set(clipboard_snapshot_representation::blob_id.eq(&blob_id_str))
            .execute(conn)?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::models::snapshot_representation::NewSnapshotRepresentationRow;
    use crate::db::schema::clipboard_snapshot_representation;
    use diesel::prelude::*;
    use uc_core::{clipboard::PersistedClipboardRepresentation, ids::{RepresentationId, EventId, FormatId}, MimeType};

    // Note: This requires a test database setup.
    // For now, we provide the test structure that can be run with proper test DB.
    // Actual execution requires test container or in-memory SQLite setup.

    #[tokio::test]
    async fn test_get_representation_found() {
        // TODO: Set up test database connection
        // This test requires DbExecutor implementation for testing

        // let executor = TestDbExecutor::new();
        // let repo = DieselClipboardRepresentationRepository::new(executor);

        // // Insert test data
        // executor.run(|conn| {
        //     diesel::insert_into(clipboard_snapshot_representation::table)
        //         .values(&NewSnapshotRepresentationRow {
        //             id: "test-rep-1".to_string(),
        //             event_id: "test-event-1".to_string(),
        //             format_id: "public.text".to_string(),
        //             mime_type: Some("text/plain".to_string()),
        //             size_bytes: 10,
        //             inline_data: Some(vec![1, 2, 3]),
        //             blob_id: None,
        //         })
        //         .execute(conn)
        //         .unwrap();
        // });

        // let result = repo
        //     .get_representation(
        //         &EventId::from("test-event-1".to_string()),
        //         &RepresentationId::from("test-rep-1".to_string()),
        //     )
        //     .await
        //     .unwrap();

        // assert!(result.is_some());
        // let rep = result.unwrap();
        // assert_eq!(rep.format_id.to_string(), "public.text");
    }

    #[tokio::test]
    async fn test_get_representation_not_found() {
        // TODO: Set up test database
        // Test that Ok(None) is returned for non-existent representation
    }

    #[tokio::test]
    async fn test_update_blob_id() {
        // TODO: Set up test database
        // Test that blob_id is correctly updated
    }
}
