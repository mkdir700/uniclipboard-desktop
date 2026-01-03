//! Storage service - High-level storage abstraction
//!
//! This service provides a unified interface for all storage operations,
//! integrating database access and file storage.

use crate::application::clipboard_service::ClipboardItemResponse;
use crate::domain::clipboard_metadata::ClipboardMetadata;
use crate::domain::device::Device;
use crate::error::{AppError, Result};
use crate::infrastructure::storage::db::dao;
use crate::infrastructure::storage::db::models::clipboard_record::{
    DbClipboardRecord, Filter, OrderBy,
};
use crate::infrastructure::storage::db::models::device::DbDevice;
use crate::infrastructure::storage::db::pool::DbPool;
use crate::infrastructure::storage::file_storage::FileStorageManager;
use crate::infrastructure::storage::record_manager::ClipboardStats;
use crate::message::Payload;
use bytes::Bytes;
use chrono::Utc;
use log::error;
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

/// High-level storage service that integrates database and file storage operations.
///
/// # Responsibilities
/// - Device CRUD operations
/// - Clipboard record CRUD operations with deduplication
/// - File storage management
/// - Automatic cleanup of old records
///
/// # Example
/// ```rust,no_run
/// use crate::services::storage::StorageService;
/// use std::sync::Arc;
///
/// let storage = StorageService::new(
///     Arc::new(db_pool),
///     Arc::new(file_storage),
///     1000, // max_records
/// );
///
/// // Get all devices
/// let devices = storage.get_devices().await?;
/// ```
pub struct StorageService {
    db: Arc<DbPool>,
    file_storage: Arc<FileStorageManager>,
    max_records: usize,
}

impl StorageService {
    /// Create a new StorageService instance.
    ///
    /// # Arguments
    /// * `db` - Database connection pool
    /// * `file_storage` - File storage manager
    /// * `max_records` - Maximum number of clipboard records to keep
    pub fn new(db: Arc<DbPool>, file_storage: Arc<FileStorageManager>, max_records: usize) -> Self {
        Self {
            db,
            file_storage,
            max_records,
        }
    }

    // ========== Device Operations ==========

    /// Get all devices from the database.
    pub async fn get_devices(&self) -> Result<Vec<Device>> {
        let mut conn = self.db.get()?;
        let db_devices = dao::device::get_all_devices(&mut conn)?;
        Ok(db_devices.into_iter().map(|d| d.into()).collect())
    }

    /// Get a device by its ID.
    pub async fn get_device_by_id(&self, id: &str) -> Result<Option<Device>> {
        let mut conn = self.db.get()?;
        let db_device = dao::device::get_device(&mut conn, id)?;
        Ok(db_device.map(|d| d.into()))
    }

    /// Get the local (self) device.
    pub async fn get_self_device(&self) -> Result<Option<Device>> {
        let mut conn = self.db.get()?;
        let db_device = dao::device::get_self_device(&mut conn)?;
        Ok(db_device.map(|d| d.into()))
    }

    /// Insert or update a device.
    pub async fn upsert_device(&self, device: &Device) -> Result<()> {
        let mut conn = self.db.get()?;
        let db_device: DbDevice = device.into();

        // Check if device exists, then insert or update
        if dao::device::is_exist(&mut conn, &db_device.id)? {
            dao::device::update_device(&mut conn, &db_device)?;
        } else {
            dao::device::insert_device(&mut conn, &db_device)?;
        }
        Ok(())
    }

    /// Insert or update multiple devices in batch.
    pub async fn upsert_devices(&self, devices: &[Device]) -> Result<()> {
        let mut conn = self.db.get()?;
        let db_devices: Vec<DbDevice> = devices.iter().map(|d| d.into()).collect();
        dao::device::batch_insert_devices(&mut conn, &db_devices)?;
        Ok(())
    }

    /// Delete a device by its ID.
    pub async fn delete_device(&self, id: &str) -> Result<()> {
        let mut conn = self.db.get()?;
        dao::device::delete_device(&mut conn, id)?;
        Ok(())
    }

    /// Update the status of a device.
    pub async fn update_device_status(&self, id: &str, status: crate::domain::device::DeviceStatus) -> Result<()> {
        let mut conn = self.db.get()?;
        dao::device::update_device_status(&mut conn, id, status as i32)?;
        Ok(())
    }

    /// Update the alias of a device.
    pub async fn update_device_alias(&self, id: &str, alias: &str) -> Result<()> {
        let mut conn = self.db.get()?;
        dao::device::update_device_alias(&mut conn, id, alias)?;
        Ok(())
    }

    /// Update the peer_id of a device.
    pub async fn update_device_peer_id(&self, id: &str, peer_id: &str) -> Result<()> {
        let mut conn = self.db.get()?;
        dao::device::update_device_peer_id(&mut conn, id, peer_id)?;
        Ok(())
    }

    /// Update the last_seen timestamp of a device.
    pub async fn update_device_last_seen(&self, id: &str, last_seen: i32) -> Result<()> {
        let mut conn = self.db.get()?;
        dao::device::update_device_last_seen(&mut conn, id, last_seen)?;
        Ok(())
    }

    // ========== Clipboard Record Operations ==========

    /// Get clipboard items with optional filtering, sorting, and pagination.
    pub async fn get_clipboard_items(
        &self,
        filter: Option<Filter>,
        order_by: Option<OrderBy>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<ClipboardItemResponse>> {
        let mut conn = self.db.get()?;
        let records = dao::clipboard_record::query_clipboard_records(
            &mut conn,
            order_by,
            limit,
            offset,
            filter,
        )?;
        Ok(records
            .into_iter()
            .map(|r| ClipboardItemResponse::from((r, false)))
            .collect())
    }

    /// Get a single clipboard item by ID.
    pub async fn get_clipboard_item_by_id(
        &self,
        id: &str,
        full_content: bool,
    ) -> Result<Option<ClipboardItemResponse>> {
        let mut conn = self.db.get()?;
        let record = dao::clipboard_record::get_clipboard_record_by_id(&mut conn, id)?;
        Ok(record.map(|r| ClipboardItemResponse::from((r, full_content))))
    }

    /// Get a single clipboard record (DbClipboardRecord) by ID.
    ///
    /// This is used when you need the raw database record, not the response format.
    pub async fn get_clipboard_record_by_id(&self, id: &str) -> Result<Option<DbClipboardRecord>> {
        let mut conn = self.db.get()?;
        Ok(dao::clipboard_record::get_clipboard_record_by_id(&mut conn, id)?)
    }

    /// Save a clipboard item with automatic deduplication.
    ///
    /// If a record with the same content_hash already exists, it will be updated
    /// instead of creating a new record. Old records are automatically cleaned up
    /// when the max_records limit is exceeded.
    pub async fn save_clipboard_item(&self, metadata: &ClipboardMetadata) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().timestamp() as i32;
        let content_hash = metadata.get_content_hash().to_string();

        let mut conn = self.db.get()?;

        // Check if record with same content_hash exists (deduplication)
        let existing =
            dao::clipboard_record::query_clipboard_records_by_content_hash(&mut conn, &content_hash)?;

        if existing.is_empty() {
            // Create new record
            let record = DbClipboardRecord::new(
                id.clone(),
                metadata.get_device_id().to_string(),
                Some(metadata.get_storage_path().to_string()),
                None,
                metadata.get_content_type().to_string(),
                Some(content_hash.clone()),
                Some(metadata.get_size() as i32),
                false,
                now,
                now,
                now,
                metadata.try_into()?,
            )?;

            dao::clipboard_record::insert_clipboard_record(&mut conn, &record)?;

            // Cleanup old records asynchronously
            self.cleanup_old_records();

            Ok(id)
        } else {
            // Update existing record
            let existing_record = &existing[0];
            dao::clipboard_record::update_clipboard_record(
                &mut conn,
                &existing_record.id,
                &existing_record.get_update_record(),
            )?;
            Ok(existing_record.id.clone())
        }
    }

    /// Delete a clipboard item by ID.
    ///
    /// Also deletes the associated file if it exists.
    pub async fn delete_clipboard_item(&self, id: &str) -> Result<()> {
        let mut conn = self.db.get()?;

        // Get the record first to check for associated file
        if let Some(record) = dao::clipboard_record::get_clipboard_record_by_id(&mut conn, id)? {
            // Delete associated file if exists
            if let Some(path) = &record.local_file_path {
                if let Err(e) = self.file_storage.delete(Path::new(path)).await {
                    log::warn!("Failed to delete file, but continuing: {}", e);
                }
            }

            // Delete the record
            dao::clipboard_record::delete_clipboard_record(&mut conn, id)?;
        }

        Ok(())
    }

    /// Clear all clipboard items.
    ///
    /// Also deletes all associated files.
    pub async fn clear_clipboard_items(&self) -> Result<usize> {
        let mut conn = self.db.get()?;

        // Get all records to delete associated files
        let records = dao::clipboard_record::query_clipboard_records(&mut conn, None, None, None, None)?;

        // Delete all associated files
        for record in &records {
            if let Some(path) = &record.local_file_path {
                if let Err(e) = self.file_storage.delete(Path::new(path)).await {
                    log::warn!("Failed to delete file: {}, continuing", e);
                }
            }
        }

        // Clear all records
        Ok(dao::clipboard_record::clear_all_records(&mut conn)?)
    }

    /// Toggle the favorite status of a clipboard item.
    pub async fn toggle_clipboard_item_favorite(&self, id: &str, is_favorited: bool) -> Result<()> {
        let mut conn = self.db.get()?;

        if let Some(mut record) = dao::clipboard_record::get_clipboard_record_by_id(&mut conn, id)? {
            record.is_favorited = is_favorited;
            dao::clipboard_record::update_clipboard_record(
                &mut conn,
                id,
                &record.get_update_record(),
            )?;
        }

        Ok(())
    }

    /// Update the active_time of a clipboard item.
    pub async fn update_clipboard_item_active_time(&self, id: &str, active_time: Option<i32>) -> Result<()> {
        let mut conn = self.db.get()?;

        if let Some(mut record) = dao::clipboard_record::get_clipboard_record_by_id(&mut conn, id)? {
            record.active_time = active_time.unwrap_or(Utc::now().timestamp() as i32);
            dao::clipboard_record::update_clipboard_record(
                &mut conn,
                id,
                &record.get_update_record(),
            )?;
        }

        Ok(())
    }

    /// Get clipboard statistics.
    pub async fn get_clipboard_stats(&self) -> Result<ClipboardStats> {
        let mut conn = self.db.get()?;
        let total_items = dao::clipboard_record::get_total_items(&mut conn)?;
        let total_size = dao::clipboard_record::get_total_size(&mut conn)?;
        Ok(ClipboardStats {
            total_items: total_items as usize,
            total_size: total_size as usize,
        })
    }

    /// Find clipboard records by content hash (internal helper).
    pub async fn find_clipboard_items_by_content_hash(
        &self,
        content_hash: &str,
    ) -> Result<Vec<DbClipboardRecord>> {
        let mut conn = self.db.get()?;
        Ok(dao::clipboard_record::query_clipboard_records_by_content_hash(&mut conn, content_hash)?)
    }

    // ========== File Storage Operations ==========

    /// Store clipboard content to the file system.
    pub async fn store_clipboard_content(&self, payload: &Payload) -> Result<std::path::PathBuf> {
        self.file_storage
            .store(payload)
            .await
            .map_err(|e| AppError::io(format!("Failed to store content: {}", e)))
    }

    /// Read file content.
    pub async fn read_file(&self, path: &Path) -> Result<Bytes> {
        self.file_storage
            .read(path)
            .await
            .map_err(|e| AppError::io(format!("Failed to read file: {}", e)))
    }

    /// Delete a file.
    pub async fn delete_file(&self, path: &Path) -> Result<()> {
        self.file_storage
            .delete(path)
            .await
            .map_err(|e| AppError::io(format!("Failed to delete file: {}", e)))
    }

    // ========== Helper Methods ==========

    /// Cleanup old records to maintain max_records limit.
    ///
    /// This runs asynchronously to avoid blocking the main operation.
    fn cleanup_old_records(&self) {
        let max_records = self.max_records;
        tokio::spawn(async move {
            if let Err(e) = Self::do_cleanup_old_records(max_records).await {
                error!("Failed to cleanup old records: {:?}", e);
            }
        });
    }

    async fn do_cleanup_old_records(max_records: usize) -> Result<()> {
        let pool = &crate::infrastructure::storage::db::pool::DB_POOL.pool;
        let mut conn = pool.get()?;
        let count = dao::clipboard_record::get_record_count(&mut conn)?;

        if count <= max_records as i64 {
            return Ok(());
        }

        let to_delete = count - max_records as i64;

        // Get records to delete (oldest first)
        let records = dao::clipboard_record::query_clipboard_records(
            &mut conn,
            Some(OrderBy::CreatedAtAsc),
            Some(to_delete),
            None,
            None,
        )?;

        // Delete records (files will be cleaned up separately if needed)
        for record in records {
            dao::clipboard_record::delete_clipboard_record(&mut conn, &record.id)?;
        }

        log::info!("Cleaned up {} old clipboard records", to_delete);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require a database connection and file storage setup.
    // They should be run as integration tests with proper fixtures.
}
