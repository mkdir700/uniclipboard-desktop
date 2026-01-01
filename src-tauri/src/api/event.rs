use serde::Serialize;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Runtime, State};

use crate::infrastructure::event::event_bus::{
    subscribe_clipboard_new_content, subscribe_connection_request, subscribe_connection_response,
    ListenerId,
};

/// 剪贴板新内容事件数据
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardNewContent {
    /// 新内容的记录ID
    pub record_id: String,
    /// 事件发生的时间戳（毫秒）
    pub timestamp: u64,
}

/// 连接请求事件数据
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionRequestEventData {
    /// 请求方设备 ID
    pub requester_device_id: String,
    /// 请求方 IP 地址
    pub requester_ip: String,
    /// 请求方设备别名（可选）
    pub requester_alias: Option<String>,
    /// 请求方平台（可选）
    pub requester_platform: Option<String>,
}

/// 连接响应事件数据
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionResponseEventData {
    /// 是否接受连接
    pub accepted: bool,
    /// 响应方设备 ID
    pub responder_device_id: String,
    /// 响应方 IP 地址（可选）
    pub responder_ip: Option<String>,
    /// 响应方设备别名（可选）
    pub responder_alias: Option<String>,
}

/// 事件监听器状态
pub struct EventListenerState {
    /// 剪贴板新内容事件监听器ID
    clipboard_new_content_listener_id: Option<ListenerId>,
    /// 连接请求事件监听器ID
    connection_request_listener_id: Option<ListenerId>,
    /// 连接响应事件监听器ID
    connection_response_listener_id: Option<ListenerId>,
}

impl Default for EventListenerState {
    fn default() -> Self {
        Self {
            clipboard_new_content_listener_id: None,
            connection_request_listener_id: None,
            connection_response_listener_id: None,
        }
    }
}

/// 监听剪贴板新内容事件
///
/// 当有新的剪贴板内容时，会向前端发送 "clipboard-new-content" 事件
#[tauri::command]
pub fn listen_clipboard_new_content<R: Runtime>(
    app_handle: AppHandle<R>,
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
        crate::infrastructure::event::event_bus::EVENT_BUS.unsubscribe(listener_id);
    }
}

/// 监听连接请求事件
///
/// 当收到其他设备的连接请求时，会向前端发送 "connection-request" 事件
#[tauri::command]
pub fn listen_connection_request<R: Runtime>(
    app_handle: AppHandle<R>,
    event_listener_state: State<'_, Arc<Mutex<EventListenerState>>>,
) {
    // 如果已经在监听，则不再重复监听
    let mut state = event_listener_state.lock().unwrap();
    if state.connection_request_listener_id.is_some() {
        return;
    }

    // 订阅连接请求事件
    let app_handle_clone = app_handle.clone();
    let listener_id = subscribe_connection_request(move |event| {
        let event_data: ConnectionRequestEventData = ConnectionRequestEventData {
            requester_device_id: event.requester_device_id.clone(),
            requester_ip: event.requester_ip.clone(),
            requester_alias: event.requester_alias.clone(),
            requester_platform: event.requester_platform.clone(),
        };

        log::info!(
            "Received connection-request event from {}",
            event.requester_device_id
        );

        // 向前端发送事件
        if let Err(e) = app_handle_clone.emit("connection-request", event_data) {
            log::error!("Failed to emit connection-request event: {:?}", e);
        }
    });

    // 保存监听器ID，以便后续可以取消监听
    state.connection_request_listener_id = Some(listener_id);
}

/// 停止监听连接请求事件
#[tauri::command]
pub fn stop_listen_connection_request(
    event_listener_state: State<'_, Arc<Mutex<EventListenerState>>>,
) {
    let mut state = event_listener_state.lock().unwrap();
    if let Some(listener_id) = state.connection_request_listener_id.take() {
        crate::infrastructure::event::event_bus::EVENT_BUS.unsubscribe(listener_id);
    }
}

/// 监听连接响应事件
///
/// 当收到连接响应时，会向前端发送 "connection-response" 事件
#[tauri::command]
pub fn listen_connection_response<R: Runtime>(
    app_handle: AppHandle<R>,
    event_listener_state: State<'_, Arc<Mutex<EventListenerState>>>,
) {
    // 如果已经在监听，则不再重复监听
    let mut state = event_listener_state.lock().unwrap();
    if state.connection_response_listener_id.is_some() {
        return;
    }

    // 订阅连接响应事件
    let app_handle_clone = app_handle.clone();
    let listener_id = subscribe_connection_response(move |event| {
        let event_data: ConnectionResponseEventData = ConnectionResponseEventData {
            accepted: event.accepted,
            responder_device_id: event.responder_device_id.clone(),
            responder_ip: event.responder_ip.clone(),
            responder_alias: event.responder_alias.clone(),
        };

        log::info!(
            "Received connection-response event from {}, accepted: {}",
            event.responder_device_id,
            event.accepted
        );

        // 向前端发送事件
        if let Err(e) = app_handle_clone.emit("connection-response", event_data) {
            log::error!("Failed to emit connection-response event: {:?}", e);
        }
    });

    // 保存监听器ID，以便后续可以取消监听
    state.connection_response_listener_id = Some(listener_id);
}

/// 停止监听连接响应事件
#[tauri::command]
pub fn stop_listen_connection_response(
    event_listener_state: State<'_, Arc<Mutex<EventListenerState>>>,
) {
    let mut state = event_listener_state.lock().unwrap();
    if let Some(listener_id) = state.connection_response_listener_id.take() {
        crate::infrastructure::event::event_bus::EVENT_BUS.unsubscribe(listener_id);
    }
}
