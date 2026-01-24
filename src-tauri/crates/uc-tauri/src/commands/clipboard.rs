//! Clipboard-related Tauri commands
//! 剪贴板相关的 Tauri 命令

use crate::bootstrap::AppRuntime;
use crate::models::{
    ClipboardEntriesResponse, ClipboardEntryDetail, ClipboardEntryProjection,
    ClipboardEntryResource,
};
use std::sync::Arc;
use std::time::Duration;
use tauri::State;
use tracing::{info_span, Instrument};
use uc_core::security::state::EncryptionState;
use uc_core::{ids::EntryId, ClipboardChangeOrigin};

/// Get clipboard history entries (preview only)
/// 获取剪贴板历史条目（仅预览）
#[tauri::command]
pub async fn get_clipboard_entries(
    runtime: State<'_, Arc<AppRuntime>>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<ClipboardEntriesResponse, String> {
    let resolved_limit = limit.unwrap_or(50);
    let resolved_offset = offset.unwrap_or(0);
    let device_id = runtime.deps.device_identity.current_device_id();

    let span = info_span!(
        "command.clipboard.get_entries",
        device_id = %device_id,
        limit = resolved_limit,
        offset = resolved_offset,
    );

    async move {
        // Check encryption session readiness to avoid decryption failures during startup
        let encryption_state = runtime
            .deps
            .encryption_state
            .load_state()
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to check encryption state");
                format!("Failed to check encryption state: {}", e)
            })?;

        let session_ready = runtime.deps.encryption_session.is_ready().await;
        if should_return_not_ready(encryption_state, session_ready) {
            tracing::warn!(
                "Encryption initialized but session not ready yet, returning not-ready response. \
                 This typically happens during app startup before auto-unlock completes."
            );
            return Ok(ClipboardEntriesResponse::NotReady);
        }

        let uc = runtime.usecases().list_entry_projections();
        let dtos = uc
            .execute(resolved_limit, resolved_offset)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to get clipboard entry projections");
                e.to_string()
            })?;

        // Map DTOs to command layer models
        let projections: Vec<ClipboardEntryProjection> = dtos
            .into_iter()
            .map(|dto| ClipboardEntryProjection {
                id: dto.id,
                preview: dto.preview,
                has_detail: dto.has_detail,
                size_bytes: dto.size_bytes,
                captured_at: dto.captured_at,
                content_type: dto.content_type,
                thumbnail_url: dto.thumbnail_url,
                is_encrypted: dto.is_encrypted,
                is_favorited: dto.is_favorited,
                updated_at: dto.updated_at,
                active_time: dto.active_time,
            })
            .collect();

        tracing::info!(count = projections.len(), "Retrieved clipboard entries");
        Ok(ClipboardEntriesResponse::Ready {
            entries: projections,
        })
    }
    .instrument(span)
    .await
}

fn should_return_not_ready(state: EncryptionState, session_ready: bool) -> bool {
    matches!(state, EncryptionState::Initialized) && !session_ready
}

#[cfg(test)]
mod tests {
    use super::should_return_not_ready;
    use uc_core::security::state::EncryptionState;

    #[test]
    fn returns_not_ready_only_when_initialized_and_session_not_ready() {
        assert!(should_return_not_ready(EncryptionState::Initialized, false));
        assert!(!should_return_not_ready(EncryptionState::Initialized, true));
        assert!(!should_return_not_ready(
            EncryptionState::Uninitialized,
            false
        ));
    }
}

/// Deletes a clipboard entry identified by `entry_id`.
///
/// This command converts the provided `entry_id` to the domain `EntryId` type and invokes the runtime's
/// delete clipboard-entry use case; on success it returns without value, otherwise it returns a stringified error.
///
/// # Examples
///
/// ```no_run
/// # use std::sync::Arc;
/// # async fn example(runtime: tauri::State<'_, Arc<uc_tauri::bootstrap::AppRuntime>>) {
/// // Tauri provides `State<Arc<AppRuntime>>` when invoking commands from the frontend.
/// let result = uc_tauri::commands::clipboard::delete_clipboard_entry(runtime, "entry-id-123".to_string()).await;
/// match result {
///     Ok(()) => println!("Deleted"),
///     Err(e) => eprintln!("Delete failed: {}", e),
/// }
/// # }
/// ```
#[tauri::command]
pub async fn delete_clipboard_entry(
    runtime: State<'_, Arc<AppRuntime>>,
    entry_id: String,
) -> Result<(), String> {
    let device_id = runtime.deps.device_identity.current_device_id();

    let span = info_span!(
        "command.clipboard.delete_entry",
        device_id = %device_id,
        entry_id = %entry_id,
    );

    async move {
        let parsed_id = uc_core::ids::EntryId::from(entry_id.clone());
        let use_case = runtime.usecases().delete_clipboard_entry();
        use_case.execute(&parsed_id).await.map_err(|e| {
            tracing::error!(error = %e, entry_id = %entry_id, "Failed to delete entry");
            e.to_string()
        })?;

        tracing::info!(entry_id = %entry_id, "Deleted clipboard entry");
        Ok(())
    }
    .instrument(span)
    .await
}

/// Get full clipboard entry detail
/// 获取剪贴板条目完整详情
#[tauri::command]
pub async fn get_clipboard_entry_detail(
    runtime: State<'_, Arc<AppRuntime>>,
    entry_id: String,
) -> Result<ClipboardEntryDetail, String> {
    let span = info_span!(
        "command.clipboard.get_entry_detail",
        entry_id = %entry_id,
    );

    async move {
        let parsed_id = uc_core::ids::EntryId::from(entry_id.clone());
        let use_case = runtime.usecases().get_entry_detail();
        let result = use_case.execute(&parsed_id).await.map_err(|e| {
            tracing::error!(error = %e, entry_id = %entry_id, "Failed to get entry detail");
            e.to_string()
        })?;

        let detail = ClipboardEntryDetail {
            id: result.id,
            content: result.content,
            size_bytes: result.size_bytes,
            content_type: result.mime_type.unwrap_or_else(|| "unknown".to_string()),
            is_favorited: false,
            updated_at: result.created_at_ms,
            active_time: result.active_time_ms,
        };

        tracing::info!(entry_id = %entry_id, "Retrieved clipboard entry detail");
        Ok(detail)
    }
    .instrument(span)
    .await
}

/// Get clipboard entry resource metadata
/// 获取剪贴板条目资源元信息
#[tauri::command]
pub async fn get_clipboard_entry_resource(
    runtime: State<'_, Arc<AppRuntime>>,
    entry_id: String,
) -> Result<ClipboardEntryResource, String> {
    let span = info_span!(
        "command.clipboard.get_entry_resource",
        entry_id = %entry_id,
    );

    async move {
        let parsed_id = uc_core::ids::EntryId::from(entry_id.clone());
        let use_case = runtime.usecases().get_entry_resource();
        let result = use_case.execute(&parsed_id).await.map_err(|e| {
            tracing::error!(
                error = %e,
                entry_id = %entry_id,
                "Failed to get entry resource"
            );
            e.to_string()
        })?;

        let resource = ClipboardEntryResource {
            blob_id: result.blob_id.to_string(),
            mime_type: result.mime_type.unwrap_or_else(|| "unknown".to_string()),
            size_bytes: result.size_bytes,
            url: result.url,
        };

        tracing::info!(entry_id = %entry_id, "Retrieved clipboard entry resource");
        Ok(resource)
    }
    .instrument(span)
    .await
}

/// Restore clipboard entry to system clipboard.
/// 将历史剪贴板条目恢复到系统剪贴板。
#[tauri::command]
pub async fn restore_clipboard_entry(
    runtime: State<'_, Arc<AppRuntime>>,
    entry_id: String,
) -> Result<bool, String> {
    let span = info_span!(
        "command.clipboard.restore_entry",
        entry_id = %entry_id,
    );

    async move {
        let parsed_id = EntryId::from(entry_id.clone());

        let restore_uc = runtime.usecases().restore_clipboard_selection();
        let snapshot = restore_uc.build_snapshot(&parsed_id).await.map_err(|e| {
            tracing::error!(error = %e, entry_id = %entry_id, "Failed to build restore snapshot");
            e.to_string()
        })?;

        runtime
            .deps
            .clipboard_change_origin
            .set_next_origin(ClipboardChangeOrigin::LocalRestore, Duration::from_secs(2))
            .await;

        if let Err(err) = runtime.deps.system_clipboard.write_snapshot(snapshot) {
            tracing::error!(error = %err, entry_id = %entry_id, "Failed to write restore snapshot");
            return Err(err.to_string());
        }

        let touch_uc = runtime.usecases().touch_clipboard_entry();
        let touched = touch_uc.execute(&parsed_id).await.map_err(|e| {
            tracing::error!(error = %e, entry_id = %entry_id, "Failed to update entry active time");
            e.to_string()
        })?;

        if !touched {
            tracing::warn!(entry_id = %entry_id, "Entry not found when touching active time");
            return Err("Entry not found".to_string());
        }

        // TODO(sync): emit restore-originated event to remote peers when sync is implemented.

        if let Some(app) = runtime.app_handle().as_ref() {
            if let Err(err) = crate::events::forward_clipboard_event(
                app,
                crate::events::ClipboardEvent::NewContent {
                    entry_id: entry_id.clone(),
                    preview: "Clipboard restored".to_string(),
                },
            ) {
                tracing::warn!(error = %err, entry_id = %entry_id, "Failed to emit restore event");
            }
        } else {
            tracing::debug!("AppHandle not available, skipping restore event emission");
        }

        Ok(true)
    }
    .instrument(span)
    .await
}
