//! Clipboard Representation Repository
//!
//! Implements [`ClipboardRepresentationRepositoryPort`] for querying and updating
//! clipboard snapshot representations stored in SQLite.
//!
//! # Usage
//!
//! Create a repository with a database executor:
//!
//! ```ignore
//! let repo = DieselClipboardRepresentationRepository::new(executor);
//! ```
//!
//! Query a representation by event and representation ID:
//!
//! ```ignore
//! let rep = repo.get_representation(&event_id, &rep_id).await?;
//! ```
//!
//! Update blob_id after materialization:
//!
//! ```ignore
//! repo.update_blob_id(&rep_id, &blob_id).await?;
//! ```

use crate::db::mappers::snapshot_representation_mapper::RepresentationRowMapper;
use crate::db::models::snapshot_representation::SnapshotRepresentationRow;
use crate::db::ports::{DbExecutor, RowMapper};
use crate::db::schema::clipboard_snapshot_representation;
use anyhow::Result;
use diesel::{BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use uc_core::clipboard::{PayloadAvailability, PersistedClipboardRepresentation};
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

    async fn get_representation_by_id(
        &self,
        representation_id: &RepresentationId,
    ) -> Result<Option<PersistedClipboardRepresentation>> {
        let rep_id_str = representation_id.to_string();

        let row: Option<SnapshotRepresentationRow> = self.executor.run(|conn| {
            let result: Result<Option<SnapshotRepresentationRow>, diesel::result::Error> =
                clipboard_snapshot_representation::table
                    .filter(clipboard_snapshot_representation::id.eq(&rep_id_str))
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

    async fn update_blob_id_if_none(
        &self,
        representation_id: &RepresentationId,
        blob_id: &BlobId,
    ) -> Result<bool> {
        let rep_id_str = representation_id.to_string();
        let blob_id_str = blob_id.to_string();

        let updated_rows = self.executor.run(|conn| {
            let result: diesel::result::QueryResult<usize> = diesel::update(
                clipboard_snapshot_representation::table.filter(
                    clipboard_snapshot_representation::id
                        .eq(&rep_id_str)
                        .and(clipboard_snapshot_representation::blob_id.is_null()),
                ),
            )
            .set(clipboard_snapshot_representation::blob_id.eq(&blob_id_str))
            .execute(conn);
            result.map_err(|e| anyhow::anyhow!("Database error: {}", e))
        })?;

        Ok(updated_rows > 0)
    }

    async fn update_processing_result(
        &self,
        rep_id: &RepresentationId,
        expected_states: &[PayloadAvailability],
        blob_id: Option<&BlobId>,
        new_state: PayloadAvailability,
        last_error: Option<&str>,
    ) -> Result<PersistedClipboardRepresentation> {
        let rep_id_str = rep_id.to_string();
        let expected_state_strs: Vec<String> = expected_states
            .iter()
            .map(|s| s.as_str().to_string())
            .collect();

        // First, verify the representation exists and get event_id
        let event_id_str: Option<String> = self.executor.run(|conn| {
            let result: Result<Option<String>, diesel::result::Error> =
                clipboard_snapshot_representation::table
                    .filter(clipboard_snapshot_representation::id.eq(&rep_id_str))
                    .select(clipboard_snapshot_representation::event_id)
                    .first::<String>(conn)
                    .optional();
            result.map_err(|e| anyhow::anyhow!("Database error: {}", e))
        })?;

        let _event_id_str = event_id_str
            .ok_or_else(|| anyhow::anyhow!("Representation not found: {}", rep_id_str))?;

        // Perform the CAS update with all fields set in one statement
        let updated_rows = self.executor.run(|conn| {
            let base_filter = clipboard_snapshot_representation::table.filter(
                clipboard_snapshot_representation::id.eq(&rep_id_str).and(
                    clipboard_snapshot_representation::payload_state.eq_any(&expected_state_strs),
                ),
            );

            // Build the update statement with all fields in one set() call
            let update_result = if let Some(blob_id) = blob_id {
                diesel::update(base_filter)
                    .set((
                        clipboard_snapshot_representation::payload_state.eq(new_state.as_str()),
                        clipboard_snapshot_representation::last_error.eq(last_error),
                        clipboard_snapshot_representation::blob_id.eq(blob_id.to_string()),
                    ))
                    .execute(conn)
            } else {
                diesel::update(base_filter)
                    .set((
                        clipboard_snapshot_representation::payload_state.eq(new_state.as_str()),
                        clipboard_snapshot_representation::last_error.eq(last_error),
                    ))
                    .execute(conn)
            };

            update_result.map_err(|e| anyhow::anyhow!("Database error: {}", e))
        })?;

        if updated_rows == 0 {
            return Err(anyhow::anyhow!(
                "CAS update failed: representation state changed. Expected one of {:?}, but current state differs",
                expected_states
            ));
        }

        // Fetch and return the updated representation
        let updated: Option<SnapshotRepresentationRow> = self.executor.run(|conn| {
            let result: Result<Option<SnapshotRepresentationRow>, diesel::result::Error> =
                clipboard_snapshot_representation::table
                    .filter(clipboard_snapshot_representation::id.eq(&rep_id_str))
                    .first::<SnapshotRepresentationRow>(conn)
                    .optional();
            result.map_err(|e| anyhow::anyhow!("Database error: {}", e))
        })?;

        let row = updated.ok_or_else(|| {
            anyhow::anyhow!("Representation disappeared after update: {}", rep_id_str)
        })?;

        let mapper = RepresentationRowMapper;
        mapper.to_domain(&row)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uc_core::clipboard::{PayloadAvailability, PersistedClipboardRepresentation};
    use uc_core::ids::{EventId, FormatId, RepresentationId};

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
        //             payload_state: "Inline".to_string(),
        //             last_error: None,
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

    #[tokio::test]
    async fn test_update_processing_result_cas() {
        // TODO: Set up test database
        // Test CAS semantics:
        // 1. Create representation with state=Staged
        // 2. Call update_processing_result with expected_states=[Staged, Processing]
        // 3. Should succeed and return updated representation with new state
        // 4. Call again with expected_states=[Staged] (but state is now BlobReady)
        // 5. Should fail with CAS error

        // Example test structure:
        // let executor = TestDbExecutor::new();
        // let repo = DieselClipboardRepresentationRepository::new(executor);
        // let rep_id = RepresentationId::new();
        // let blob_id = BlobId::new();
        //
        // // Insert Staged representation
        // ...
        //
        // // Should succeed - state is Staged
        // let result = repo.update_processing_result(
        //     &rep_id,
        //     &[PayloadAvailability::Staged, PayloadAvailability::Processing],
        //     Some(&blob_id),
        //     PayloadAvailability::BlobReady,
        //     None,
        // ).await.unwrap();
        //
        // assert_eq!(result.payload_state(), PayloadAvailability::BlobReady);
        // assert_eq!(result.blob_id, Some(blob_id));
        //
        // // Should fail - state is now BlobReady, not in expected states
        // let err = repo.update_processing_result(
        //     &rep_id,
        //     &[PayloadAvailability::Staged],
        //     None,
        //     PayloadAvailability::Lost,
        //     Some("test error"),
        // ).await.unwrap_err();
        //
        // assert!(err.to_string().contains("CAS update failed"));
    }
}
