//! Event Forwarding - Forward backend events to frontend
//! 事件转发 - 将后端事件转发到前端

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

/// Clipboard events emitted to frontend
/// 发送到前端的剪贴板事件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClipboardEvent {
    /// New clipboard content captured
    NewContent { entry_id: String, preview: String },
    /// Clipboard content deleted
    Deleted { entry_id: String },
}

/// Encryption events emitted to frontend
/// 发送到前端的加密事件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EncryptionEvent {
    /// Encryption initialized
    Initialized,
    /// Encryption session ready (auto-unlock completed)
    SessionReady,
    /// Encryption failed
    Failed { reason: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encryption_event_serializes_with_type_tag() {
        let ready = serde_json::to_value(EncryptionEvent::SessionReady).unwrap();
        assert_eq!(ready, serde_json::json!({ "type": "SessionReady" }));

        let failed = serde_json::to_value(EncryptionEvent::Failed {
            reason: "oops".to_string(),
        })
        .unwrap();
        assert_eq!(
            failed,
            serde_json::json!({ "type": "Failed", "reason": "oops" })
        );
    }
}

/// Forward clipboard event to frontend
/// 将剪贴板事件转发到前端
pub fn forward_clipboard_event(
    app: &AppHandle,
    event: ClipboardEvent,
) -> Result<(), Box<dyn std::error::Error>> {
    app.emit("clipboard://event", event)?;
    Ok(())
}

/// Forward encryption event to frontend
/// 将加密事件转发到前端
pub fn forward_encryption_event(
    app: &AppHandle,
    event: EncryptionEvent,
) -> Result<(), Box<dyn std::error::Error>> {
    app.emit("encryption://event", event)?;
    Ok(())
}
