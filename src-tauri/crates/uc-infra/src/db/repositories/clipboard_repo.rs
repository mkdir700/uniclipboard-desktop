use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use diesel::prelude::*;

use crate::db::models::{
    ClipboardItemRow, ClipboardRecordRow, NewClipboardItemRow, NewClipboardRecordRow,
};
use crate::db::schema::t_clipboard_item::dsl as dsl_item;
use crate::db::schema::t_clipboard_record::dsl as dsl_record;

use crate::db::ports::{ClipboardDbPort, DbExecutor};

/// Extension trait for converting i64 timestamps to DateTime<Utc>
trait TimestampExt {
    fn to_datetime(self) -> DateTime<Utc>;
}

impl TimestampExt for i64 {
    /// Converts an integer representing milliseconds since the Unix epoch into a `DateTime<Utc>`,
    /// falling back to `DateTime::UNIX_EPOCH` if the milliseconds value is out of range or invalid.
    ///
    /// # Returns
    ///
    /// A `DateTime<Utc>` corresponding to `self` milliseconds since the Unix epoch, or `DateTime::UNIX_EPOCH` if conversion fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use chrono::{DateTime, Utc};
    ///
    /// let dt = 0i64.to_datetime();
    /// assert_eq!(dt, DateTime::UNIX_EPOCH);
    ///
    /// let dt = 1_000i64.to_datetime(); // 1 second after epoch
    /// assert_eq!(dt.timestamp_millis(), 1_000);
    /// ```
    fn to_datetime(self) -> DateTime<Utc> {
        DateTime::from_timestamp_millis(self).unwrap_or_else(|| DateTime::UNIX_EPOCH)
    }
}

pub struct DieselClipboardRepository<E>
where
    E: DbExecutor,
{
    db: E,
}

impl<E> DieselClipboardRepository<E>
where
    E: DbExecutor,
{
    pub fn new(db: E) -> Result<Self> {
        Ok(Self { db })
    }
}

#[async_trait]
impl<E> ClipboardDbPort for DieselClipboardRepository<E>
where
    E: DbExecutor,
{
    async fn insert_record(&self, row: NewClipboardRecordRow) -> Result<()> {
        self.db
            .run(Box::new(move |conn| {
                diesel::insert_into(dsl_record::t_clipboard_record)
                    .values(&row)
                    .on_conflict(dsl_record::record_hash)
                    .do_update()
                    .set(dsl_record::deleted_at.eq(row.deleted_at))
                    .execute(conn)?;
                Ok(())
            }))
            .await
    }

    async fn insert_item(&self, row: NewClipboardItemRow) -> Result<()> {
        self.db
            .run(Box::new(move |conn| {
                diesel::insert_into(dsl_item::t_clipboard_item)
                    .values(&row)
                    .execute(conn)?;
                Ok(())
            }))
            .await
    }

    async fn find_record_by_hash(&self, hash: String) -> Result<Option<ClipboardRecordRow>> {
        self.db
            .run(Box::new(move |conn| {
                let record = dsl_record::t_clipboard_record
                    .filter(dsl_record::record_hash.eq(hash))
                    .filter(dsl_record::deleted_at.is_null())
                    .first::<ClipboardRecordRow>(conn)
                    .optional()?;

                Ok(record)
            }))
            .await
    }
    async fn find_items_by_record(&self, record_id: String) -> Result<Vec<ClipboardItemRow>> {
        self.db
            .run(Box::new(move |conn| {
                let items = dsl_item::t_clipboard_item
                    .filter(dsl_item::record_id.eq(record_id))
                    .order(dsl_item::index_in_record.asc())
                    .load::<ClipboardItemRow>(conn)?;

                Ok(items)
            }))
            .await
    }

    async fn record_exists(&self, hash: String) -> anyhow::Result<bool> {
        self.db
            .run(Box::new(move |conn| {
                let count: i64 = dsl_record::t_clipboard_record
                    .filter(dsl_record::record_hash.eq(hash))
                    .count()
                    .get_result(conn)?;

                Ok(count > 0)
            }))
            .await
    }

    async fn soft_delete_record(&self, hash: String) -> Result<()> {
        self.db
            .run(Box::new(move |conn| {
                let now = chrono::Utc::now().timestamp_millis();
                diesel::update(
                    dsl_record::t_clipboard_record.filter(dsl_record::record_hash.eq(hash)),
                )
                .set(dsl_record::deleted_at.eq(now))
                .execute(conn)?;
                Ok(())
            }))
            .await
    }

    async fn list_recent_records(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<ClipboardRecordRow>> {
        self.db
            .run(Box::new(move |conn| {
                let records = dsl_record::t_clipboard_record
                    .filter(dsl_record::deleted_at.is_null())
                    .order(dsl_record::created_at.desc())
                    .limit(limit as i64)
                    .offset(offset as i64)
                    .load::<ClipboardRecordRow>(conn)?;

                Ok(records)
            }))
            .await
    }
}
