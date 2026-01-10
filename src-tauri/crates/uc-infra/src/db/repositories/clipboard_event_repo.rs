use anyhow::Result;
use diesel::prelude::*;
use crate::db::schema::{
    clipboard_event,
    clipboard_snapshot_representation,
};

pub struct DieselClipboardEventRepository {
    executor: DieselSqliteExecutor,
}

impl DieselClipboardEventRepository {
    pub fn new(executor: DieselSqliteExecutor) -> Self {
        Self { executor }
    }
}

#[async_trait::async_trait]
impl ClipboardEventRepository for DieselClipboardEventRepository {
    async fn insert_event(
        &self,
        event: NewClipboardEvent,
        reps: Vec<NewSnapshotRepresentation>,
    ) -> Result<()> {
        self.executor.with_conn(|conn| {
            conn.transaction(|conn| {
                diesel::insert_into(clipboard_event::table)
                    .values(&event)
                    .execute(conn)?;

                for rep in reps {
                    diesel::insert_into(clipboard_snapshot_representation::table)
                        .values((
                            clipboard_snapshot_representation::id.eq(rep.id),
                            clipboard_snapshot_representation::event_id.eq(&event.event_id),
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
