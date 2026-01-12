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
