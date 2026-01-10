use crate::db::schema::{
    clipboard_entry,
    clipboard_selection,
};

pub struct DieselClipboardEntryRepository {
    executor: DieselSqliteExecutor,
}

impl DieselClipboardEntryRepository {
    pub fn new(executor: DieselSqliteExecutor) -> Self {
        Self { executor }
    }
}

#[async_trait::async_trait]
impl ClipboardEntryRepository for DieselClipboardEntryRepository {
    async fn insert_entry(
        &self,
        entry: NewClipboardEntry,
        selection: NewClipboardSelection,
    ) -> Result<()> {
        self.executor.with_conn(|conn| {
            conn.transaction(|conn| {
                diesel::insert_into(clipboard_entry::table)
                    .values(&entry)
                    .execute(conn)?;

                diesel::insert_into(clipboard_selection::table)
                    .values(&selection)
                    .execute(conn)?;

                Ok(())
            })
        })
    }
}
