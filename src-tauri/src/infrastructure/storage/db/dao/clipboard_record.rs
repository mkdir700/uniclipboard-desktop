use crate::infrastructure::storage::db::models::clipboard_record::{
    DbClipboardRecord, NewClipboardRecord, OrderBy, UpdateClipboardRecord,
};
use crate::infrastructure::storage::db::schema::clipboard_records;
use crate::utils::helpers::get_current_time;
use anyhow::{Context, Result};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use log::debug;

/// 插入一条剪贴板记录
pub fn insert_clipboard_record(
    conn: &mut SqliteConnection,
    record: &DbClipboardRecord,
) -> Result<()> {
    let new_record = NewClipboardRecord {
        id: record.id.clone(),
        device_id: record.device_id.clone(),
        local_file_path: record.local_file_path.clone(),
        remote_record_id: record.remote_record_id.clone(),
        content_type: record.content_type.clone(),
        content_hash: record.content_hash.clone(),
        is_favorited: record.is_favorited,
        created_at: record.created_at,
        updated_at: record.updated_at,
    };

    diesel::insert_into(clipboard_records::table)
        .values(&new_record)
        .execute(conn)
        .context("Failed to insert clipboard record")?;
    Ok(())
}

/// 更新一条剪贴板记录
pub fn update_clipboard_record(
    conn: &mut SqliteConnection,
    record: &DbClipboardRecord,
) -> Result<()> {
    let update = UpdateClipboardRecord {
        is_favorited: record.is_favorited,
        updated_at: get_current_time(),
    };

    diesel::update(clipboard_records::table.find(&record.id))
        .set(&update)
        .execute(conn)
        .context("Failed to update clipboard record")?;
    Ok(())
}

/// 删除指定ID的剪贴板记录
pub fn delete_clipboard_record(conn: &mut SqliteConnection, id: &str) -> Result<()> {
    diesel::delete(clipboard_records::table.find(id))
        .execute(conn)
        .context("Failed to delete clipboard record")?;
    Ok(())
}

/// 清空所有剪贴板记录
pub fn clear_all_records(conn: &mut SqliteConnection) -> Result<usize> {
    let count = diesel::delete(clipboard_records::table)
        .execute(conn)
        .context("Failed to clear all clipboard records")?;
    Ok(count)
}

/// 获取记录总数
pub fn get_record_count(conn: &mut SqliteConnection) -> Result<i64> {
    let count = clipboard_records::table
        .count()
        .get_result(conn)
        .context("Failed to get record count")?;
    Ok(count)
}

/// 查询剪贴板记录
pub fn query_clipboard_records(
    conn: &mut SqliteConnection,
    is_favorited: Option<bool>,
    order_by: Option<OrderBy>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<DbClipboardRecord>> {
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);

    let mut query = clipboard_records::table.into_boxed();

    // 根据order_by参数设置排序方式
    match order_by.unwrap_or_default() {
        OrderBy::CreatedAtAsc => {
            query = query.order(clipboard_records::created_at.asc());
        }
        OrderBy::CreatedAtDesc => {
            query = query.order(clipboard_records::created_at.desc());
        }
        OrderBy::UpdatedAtAsc => {
            query = query.order(clipboard_records::updated_at.asc());
        }
        OrderBy::UpdatedAtDesc => {
            query = query.order(clipboard_records::updated_at.desc());
        }
        OrderBy::ContentTypeAsc => {
            query = query.order(clipboard_records::content_type.asc());
        }
        OrderBy::ContentTypeDesc => {
            query = query.order(clipboard_records::content_type.desc());
        }
        OrderBy::IsFavoritedAsc => {
            query = query.order(clipboard_records::is_favorited.asc());
        }
        OrderBy::IsFavoritedDesc => {
            query = query.order(clipboard_records::is_favorited.desc());
        }
    }

    if let Some(is_favorited) = is_favorited {
        query = query.filter(clipboard_records::is_favorited.eq(is_favorited));
    }

    let records = query
        .limit(limit)
        .offset(offset)
        .load::<DbClipboardRecord>(conn)
        .context("Failed to query clipboard records")?;

    Ok(records)
}

/// 查询指定ID的剪贴板记录
pub fn get_clipboard_record_by_id(
    conn: &mut SqliteConnection,
    id: &str,
) -> Result<Option<DbClipboardRecord>> {
    let record = clipboard_records::table
        .find(id)
        .first::<DbClipboardRecord>(conn)
        .optional()
        .context("Failed to get clipboard record by id")?;
    Ok(record)
}

/// 查询指定内容hash的剪贴板记录
pub fn query_clipboard_records_by_content_hash(
    conn: &mut SqliteConnection,
    content_hash: &str,
) -> Result<Vec<DbClipboardRecord>> {
    let records = clipboard_records::table
        .filter(clipboard_records::content_hash.eq(content_hash))
        .load::<DbClipboardRecord>(conn)
        .context("Failed to query clipboard records by content hash")?;
    Ok(records)
}
