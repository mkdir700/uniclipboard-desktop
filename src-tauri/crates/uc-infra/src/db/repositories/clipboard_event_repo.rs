use crate::db::{
    models::{
        clipboard_event::NewClipboardEventRow,
        snapshot_representation::NewSnapshotRepresentationRow,
    },
    ports::{DbExecutor, InsertMapper},
    schema::{clipboard_event, clipboard_snapshot_representation},
};
use anyhow::Result;
use diesel::prelude::*;
use uc_core::{
    clipboard::{ClipboardEvent, PersistedClipboardRepresentation},
    ids::EventId,
    ports::ClipboardEventWriterPort,
};

pub struct DieselClipboardEventRepository<E, ME, MS> {
    executor: E,
    event_mapper: ME,
    snapshot_mapper: MS,
}

impl<E, ME, MS> DieselClipboardEventRepository<E, ME, MS> {
    pub fn new(executor: E, event_mapper: ME, snapshot_mapper: MS) -> Self {
        Self {
            executor,
            event_mapper,
            snapshot_mapper,
        }
    }
}

#[async_trait::async_trait]
impl<E, ME, MS> ClipboardEventWriterPort for DieselClipboardEventRepository<E, ME, MS>
where
    E: DbExecutor,
    ME: InsertMapper<ClipboardEvent, NewClipboardEventRow>,
    for<'a> MS: InsertMapper<
        (&'a PersistedClipboardRepresentation, &'a EventId),
        NewSnapshotRepresentationRow,
    >,
{
    async fn insert_event(
        &self,
        event: &ClipboardEvent,
        reps: &Vec<PersistedClipboardRepresentation>,
    ) -> Result<()> {
        let new_event: NewClipboardEventRow = self.event_mapper.to_row(event)?;
        let new_reps: Vec<NewSnapshotRepresentationRow> = reps
            .iter()
            .map(|rep| self.snapshot_mapper.to_row(&(rep, &event.event_id)))
            .collect::<Result<Vec<_>, _>>()?;

        self.executor.run(|conn| {
            conn.transaction(|conn| {
                diesel::insert_into(clipboard_event::table)
                    .values(&new_event)
                    .execute(conn)?;

                for rep in new_reps {
                    diesel::insert_into(clipboard_snapshot_representation::table)
                        .values((
                            clipboard_snapshot_representation::id.eq(rep.id),
                            clipboard_snapshot_representation::event_id.eq(&new_event.event_id),
                            clipboard_snapshot_representation::format_id.eq(rep.format_id),
                            clipboard_snapshot_representation::mime_type.eq(rep.mime_type),
                            clipboard_snapshot_representation::size_bytes.eq(rep.size_bytes),
                            clipboard_snapshot_representation::inline_data.eq(rep.inline_data),
                            clipboard_snapshot_representation::blob_id.eq(rep.blob_id),
                        ))
                        .execute(conn)?;
                }

                Ok(())
            })
        })
    }
}
