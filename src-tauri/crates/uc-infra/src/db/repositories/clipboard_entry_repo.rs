use crate::db::models::ClipboardEntryRow;
use crate::db::models::NewClipboardEntryRow;
use crate::db::models::NewClipboardSelectionRow;
use crate::db::ports::DbExecutor;
use crate::db::ports::{InsertMapper, RowMapper};
use crate::db::schema::{clipboard_entry, clipboard_selection};
use anyhow::Result;
use diesel::query_dsl::methods::FilterDsl;
use diesel::query_dsl::methods::LimitDsl;
use diesel::query_dsl::methods::OffsetDsl;
use diesel::query_dsl::methods::OrderDsl;
use diesel::Connection;
use diesel::ExpressionMethods;
use diesel::OptionalExtension;
use diesel::RunQueryDsl;
use tracing::debug_span;
use uc_core::clipboard::{ClipboardEntry, ClipboardSelectionDecision};
use uc_core::ids::EntryId;
use uc_core::ports::ClipboardEntryRepositoryPort;

pub struct DieselClipboardEntryRepository<E, ME, MS, RE> {
    executor: E,
    entry_mapper: ME,
    selection_mapper: MS,
    row_entry_mapper: RE,
}

impl<E, ME, MS, RE> DieselClipboardEntryRepository<E, ME, MS, RE> {
    pub fn new(executor: E, entry_mapper: ME, selection_mapper: MS, row_entry_mapper: RE) -> Self {
        Self {
            executor,
            entry_mapper,
            selection_mapper,
            row_entry_mapper,
        }
    }
}

#[async_trait::async_trait]
impl<E, ME, MS, RE> ClipboardEntryRepositoryPort for DieselClipboardEntryRepository<E, ME, MS, RE>
where
    E: DbExecutor,
    ME: InsertMapper<ClipboardEntry, NewClipboardEntryRow>,
    MS: InsertMapper<ClipboardSelectionDecision, NewClipboardSelectionRow>,
    RE: RowMapper<ClipboardEntryRow, ClipboardEntry>,
{
    async fn save_entry_and_selection(
        &self,
        entry: &ClipboardEntry,
        selection: &ClipboardSelectionDecision,
    ) -> Result<()> {
        let span = debug_span!(
            "infra.sqlite.insert_clipboard_entry",
            table = "clipboard_entry",
            entry_id = %entry.entry_id,
        );
        let _enter = span.enter();

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

    async fn get_entry(&self, entry_id: &EntryId) -> Result<Option<ClipboardEntry>> {
        let span = debug_span!(
            "infra.sqlite.query_clipboard_entry",
            table = "clipboard_entry",
            entry_id = %entry_id,
        );
        let _enter = span.enter();

        let entry_id_str = entry_id.to_string();
        self.executor.run(|conn| {
            let entry_row = clipboard_entry::table
                .filter(clipboard_entry::entry_id.eq(&entry_id_str))
                .first::<ClipboardEntryRow>(conn)
                .optional()?;

            match entry_row {
                Some(row) => {
                    let entry = self.row_entry_mapper.to_domain(&row)?;
                    Ok(Some(entry))
                }
                None => Ok(None),
            }
        })
    }

    /// Lists clipboard entries ordered by creation time (newest first) with pagination.
    ///
    /// # Parameters
    ///
    /// - `limit`: Maximum number of entries to return.
    /// - `offset`: Number of entries to skip before collecting results (zero-based).
    ///
    /// # Returns
    ///
    /// A `Vec<ClipboardEntry>` containing entries ordered by `created_at_ms` descending.
    ///
    /// # Examples
    ///
    /// ```
    /// # async fn example(repo: &impl ClipboardEntryRepositoryPort) {
    /// let entries = repo.list_entries(10, 0).await.unwrap();
    /// assert!(entries.len() <= 10);
    /// # }
    /// ```
    async fn list_entries(&self, limit: usize, offset: usize) -> Result<Vec<ClipboardEntry>> {
        let span = debug_span!(
            "infra.sqlite.query_clipboard_entries",
            table = "clipboard_entry",
            limit = limit,
            offset = offset,
        );
        let _enter = span.enter();

        self.executor.run(|conn| {
            let entry_rows = clipboard_entry::table
                .order(clipboard_entry::created_at_ms.desc())
                .limit(limit as i64)
                .offset(offset as i64)
                .load::<ClipboardEntryRow>(conn)?;

            entry_rows
                .into_iter()
                .map(|row| self.row_entry_mapper.to_domain(&row))
                .collect()
        })
    }

    /// Deletes the clipboard entry with the given `EntryId` from the database.
    ///
    /// Attempts to remove the entry row whose `entry_id` matches `entry_id`. The operation returns `Ok(())` on success; if no row matches the provided `entry_id` the call still succeeds and returns `Ok(())`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // Remove an entry by id
    /// # async fn run(repo: &impl std::marker::Send, id: uc_core::domain::EntryId) {
    /// // repo.delete_entry(&id).await.unwrap();
    /// # }
    /// ```
    async fn delete_entry(&self, entry_id: &EntryId) -> Result<()> {
        let span = debug_span!(
            "infra.sqlite.delete_clipboard_entry",
            table = "clipboard_entry",
            entry_id = %entry_id,
        );
        let _enter = span.enter();

        let entry_id_str = entry_id.to_string();
        self.executor.run(|conn| {
            diesel::delete(clipboard_entry::table)
                .filter(clipboard_entry::entry_id.eq(&entry_id_str))
                .execute(conn)?;
            Ok(())
        })
    }
}