//! Clipboard-related Tauri commands
//! 剪贴板相关的 Tauri 命令

use crate::bootstrap::AppRuntime;
use crate::models::ClipboardEntryProjection;
use std::sync::Arc;
use tauri::State;

/// Get clipboard history entries
/// 获取剪贴板历史条目
#[tauri::command]
pub async fn get_clipboard_entries(
    runtime: State<'_, Arc<AppRuntime>>,
    limit: Option<usize>,
) -> Result<Vec<ClipboardEntryProjection>, String> {
    let resolved_limit = limit.unwrap_or(50);
    let device_id = runtime.deps.device_identity.current_device_id();

    log::info!(
        "Getting clipboard entries: device_id={}, limit={}",
        device_id,
        resolved_limit
    );

    // Use UseCases accessor pattern (consistent with other commands)
    let uc = runtime.usecases().list_clipboard_entries();

    // Query entries through use case
    let entries = uc.execute(resolved_limit, 0).await.map_err(|e| {
        log::error!("Failed to get clipboard entries: {}", e);
        e.to_string()
    })?;

    // Convert domain models to DTOs
    let mut projections = Vec::with_capacity(entries.len());

    for entry in entries {
        let captured_at = entry.created_at_ms;

        // Try to get actual text content from selection and representation
        // Return full content, not truncated - frontend will handle display truncation
        let (preview, size_bytes) = if let Ok(Some(selection)) = runtime
            .deps
            .selection_repo
            .get_selection(&entry.entry_id)
            .await
        {
            // Get the preview representation
            if let Ok(rep) = runtime
                .deps
                .clipboard_event_reader
                .get_representation(
                    &entry.event_id,
                    selection.selection.preview_rep_id.as_ref(),
                )
                .await
            {
                // Try to convert bytes to UTF-8 string
                if let Ok(text) = std::str::from_utf8(&rep.bytes) {
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        (trimmed.to_string(), rep.bytes.len() as i64)
                    } else {
                        (
                            entry.title.unwrap_or_else(|| {
                                format!("Entry ({} bytes)", entry.total_size)
                            }),
                            entry.total_size,
                        )
                    }
                } else {
                    (
                        entry
                            .title
                            .unwrap_or_else(|| format!("Entry ({} bytes)", entry.total_size)),
                        entry.total_size,
                    )
                }
            } else {
                (
                    entry
                        .title
                        .unwrap_or_else(|| format!("Entry ({} bytes)", entry.total_size)),
                    entry.total_size,
                )
            }
        } else {
            (
                entry
                    .title
                    .unwrap_or_else(|| format!("Entry ({} bytes)", entry.total_size)),
                entry.total_size,
            )
        };

        projections.push(ClipboardEntryProjection {
            id: entry.entry_id.to_string(),
            preview,
            size_bytes,
            captured_at,
            content_type: "clipboard".to_string(),
            is_encrypted: false, // TODO: Determine from actual entry state
            is_favorited: false, // TODO: Implement favorites feature
            updated_at: captured_at, // Same as captured_at initially
            active_time: captured_at, // Same as captured_at initially
        });
    }

    log::info!("Retrieved {} clipboard entries", projections.len());
    Ok(projections)
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

    log::info!(
        "Deleting clipboard entry: device_id={}, entry_id={}",
        device_id,
        entry_id
    );

    // Parse entry_id (From trait always succeeds)
    let parsed_id = uc_core::ids::EntryId::from(entry_id.clone());

    // Execute use case
    let use_case = runtime.usecases().delete_clipboard_entry();
    use_case.execute(&parsed_id).await.map_err(|e| {
        log::error!("Failed to delete entry {}: {}", entry_id, e);
        e.to_string()
    })?;

    log::info!("Deleted clipboard entry: {}", entry_id);
    Ok(())
}
