use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Runtime, State};
use serde::Serialize;

use crate::core::{event_bus::{subscribe_clipboard_new_content, ListenerId}, uniclipboard::UniClipboard};

/// 剪贴板新内容事件数据
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardNewContent {
    /// 新内容的记录ID
    pub record_id: String,
    /// 事件发生的时间戳（毫秒）
    pub timestamp: u64,
}

/// 事件监听器状态
pub struct EventListenerState {
    /// 剪贴板新内容事件监听器ID
    clipboard_new_content_listener_id: Option<ListenerId>,
}

impl Default for EventListenerState {
    fn default() -> Self {
        Self {
            clipboard_new_content_listener_id: None,
        }
    }
}

/// 监听剪贴板新内容事件
/// 
/// 当有新的剪贴板内容时，会向前端发送 "clipboard-new-content" 事件
#[tauri::command]
pub fn listen_clipboard_new_content<R: Runtime>(
    app_handle: AppHandle<R>,
    uniclipboard_app: State<'_, Arc<Mutex<Option<Arc<UniClipboard>>>>>,
    event_listener_state: State<'_, Arc<Mutex<EventListenerState>>>,
) {
    // 如果已经在监听，则不再重复监听
    let mut state = event_listener_state.lock().unwrap();
    if state.clipboard_new_content_listener_id.is_some() {
        return;
    }

    // 订阅剪贴板新内容事件
    let app_handle_clone = app_handle.clone();
    let listener_id = subscribe_clipboard_new_content(move |event| {
        let event_data: ClipboardNewContent = ClipboardNewContent {
            record_id: event.record_id.clone(),
            timestamp: event.timestamp,
        };

        log::info!("Received clipboard-new-content event");

        // 向前端发送事件
        if let Err(e) = app_handle_clone.emit("clipboard-new-content", event_data) {
            log::error!("Failed to emit clipboard-new-content event: {:?}", e);
        }
    });

    // 保存监听器ID，以便后续可以取消监听
    state.clipboard_new_content_listener_id = Some(listener_id);
}

/// 停止监听剪贴板新内容事件
#[tauri::command]
pub fn stop_listen_clipboard_new_content(
    event_listener_state: State<'_, Arc<Mutex<EventListenerState>>>,
) {
    let mut state = event_listener_state.lock().unwrap();
    if let Some(listener_id) = state.clipboard_new_content_listener_id.take() {
        crate::core::event_bus::EVENT_BUS.unsubscribe(listener_id);
    }
}
