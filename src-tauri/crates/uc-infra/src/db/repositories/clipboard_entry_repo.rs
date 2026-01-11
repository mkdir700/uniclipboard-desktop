use crate::db::models::NewClipboardEntryRow;
use crate::db::models::NewClipboardSelectionRow;
use crate::db::ports::DbExecutor;
use crate::db::ports::InsertMapper;
use crate::db::schema::{clipboard_entry, clipboard_selection};
use anyhow::Result;
use diesel::Connection;
use diesel::RunQueryDsl;
use uc_core::{
    clipboard::{ClipboardEntry, ClipboardSelectionDecision},
    ports::ClipboardEntryWriterPort,
};

pub struct DieselClipboardEntryRepository<E, ME, MS> {
    executor: E,
    entry_mapper: ME,
    selection_mapper: MS,
}

impl<E, ME, MS> DieselClipboardEntryRepository<E, ME, MS> {
    pub fn new(executor: E, entry_mapper: ME, selection_mapper: MS) -> Self {
        Self {
            executor,
            entry_mapper,
            selection_mapper,
        }
    }
}

#[async_trait::async_trait]
impl<E, ME, MS> ClipboardEntryWriterPort for DieselClipboardEntryRepository<E, ME, MS>
where
    E: DbExecutor,
    ME: InsertMapper<ClipboardEntry, NewClipboardEntryRow>,
    MS: InsertMapper<ClipboardSelectionDecision, NewClipboardSelectionRow>,
{
    async fn insert_entry(
        &self,
        entry: &ClipboardEntry,
        selection: &ClipboardSelectionDecision,
    ) -> Result<()> {
        self.executor.run(|conn| {
            let new_entry_row = self.entry_mapper.to_row(entry)?;
            let new_selection_row = self.selection_mapper.to_row(selection)?;

            conn.transaction(|conn| {
                diesel::insert_into(clipboard_entry::table)
                    .values(&new_entry_row)
                    .execute(conn)?;

                diesel::insert_into(clipboard_selection::table)
                    .values(&new_selection_row)
                    .execute(conn)?;

                Ok(())
            })
        })
    }
}
