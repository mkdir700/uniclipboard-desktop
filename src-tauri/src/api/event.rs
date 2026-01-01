use serde::Serialize;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Runtime, State};

use crate::infrastructure::event::event_bus::{subscribe_clipboard_new_content, ListenerId};

/// 剪贴板新内容事件数据
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardNewContent {
    /// 新内容的记录ID
    pub record_id: String,
    /// 事件发生的时间戳（毫秒）
    pub timestamp: u64,
}

/// P2P 配对请求事件数据
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct P2PPairingRequestEventData {
    /// Session ID
    pub session_id: String,
    /// Peer ID of the requester
    pub peer_id: String,
    /// Device name of the requester
    pub device_name: Option<String>,
}

/// P2P PIN 就绪事件数据
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct P2PPinReadyEventData {
    /// Session ID
    pub session_id: String,
    /// PIN to verify
    pub pin: String,
    /// Peer device name
    pub peer_device_name: String,
}

/// P2P 配对完成事件数据
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct P2PPairingCompleteEventData {
    /// Session ID
    pub session_id: String,
    /// Peer ID
    pub peer_id: String,
    /// Device name
    pub device_name: String,
}

/// P2P 配对失败事件数据
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct P2PPairingFailedEventData {
    /// Session ID
    pub session_id: String,
    /// Error message
    pub error: String,
}

/// Onboarding 密码设置成功事件数据
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OnboardingPasswordSetEvent {
    /// 事件时间戳（毫秒）
    pub timestamp: u64,
}

/// Onboarding 流程完成事件数据
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OnboardingCompletedEvent {
    /// 事件时间戳（毫秒）
    pub timestamp: u64,
}

/// 事件监听器状态
pub struct EventListenerState {
    /// 剪贴板新内容事件监听器ID
    clipboard_new_content_listener_id: Option<ListenerId>,
    /// P2P 配对请求事件监听器ID
    p2p_pairing_request_listener_id: Option<ListenerId>,
    /// P2P PIN 就绪事件监听器ID
    p2p_pin_ready_listener_id: Option<ListenerId>,
    /// P2P 配对完成事件监听器ID
    p2p_pairing_complete_listener_id: Option<ListenerId>,
    /// P2P 配对失败事件监听器ID
    p2p_pairing_failed_listener_id: Option<ListenerId>,
}

impl Default for EventListenerState {
    fn default() -> Self {
        Self {
            clipboard_new_content_listener_id: None,
            p2p_pairing_request_listener_id: None,
            p2p_pin_ready_listener_id: None,
            p2p_pairing_complete_listener_id: None,
            p2p_pairing_failed_listener_id: None,
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

/// 监听 P2P 配对请求事件
///
/// 当收到其他设备的P2P配对请求时，会向前端发送 "p2p-pairing-request" 事件
#[tauri::command]
pub fn listen_p2p_pairing_request<R: Runtime>(
    app_handle: AppHandle<R>,
    event_listener_state: State<'_, Arc<Mutex<EventListenerState>>>,
) {
    // 如果已经在监听，则不再重复监听
    let mut state = event_listener_state.lock().unwrap();
    if state.p2p_pairing_request_listener_id.is_some() {
        return;
    }

    // 标记为正在监听
    state.p2p_pairing_request_listener_id = Some(ListenerId(0));

    // 注意：P2P事件直接通过AppHandle发送，不通过内部事件总线
    // 这里只是注册监听器状态，实际的事件发送在 p2p_runtime.rs 中处理
}

/// 停止监听 P2P 配对请求事件
#[tauri::command]
pub fn stop_listen_p2p_pairing_request(
    event_listener_state: State<'_, Arc<Mutex<EventListenerState>>>,
) {
    let mut state = event_listener_state.lock().unwrap();
    state.p2p_pairing_request_listener_id = None;
}

/// 监听 P2P PIN 就绪事件
///
/// 当PIN准备好时，会向前端发送 "p2p-pin-ready" 事件
#[tauri::command]
pub fn listen_p2p_pin_ready<R: Runtime>(
    app_handle: AppHandle<R>,
    event_listener_state: State<'_, Arc<Mutex<EventListenerState>>>,
) {
    // 如果已经在监听，则不再重复监听
    let mut state = event_listener_state.lock().unwrap();
    if state.p2p_pin_ready_listener_id.is_some() {
        return;
    }

    // 标记为正在监听
    state.p2p_pin_ready_listener_id = Some(ListenerId(0));
}

/// 停止监听 P2P PIN 就绪事件
#[tauri::command]
pub fn stop_listen_p2p_pin_ready(event_listener_state: State<'_, Arc<Mutex<EventListenerState>>>) {
    let mut state = event_listener_state.lock().unwrap();
    state.p2p_pin_ready_listener_id = None;
}

/// 监听 P2P 配对完成事件
///
/// 当配对完成时，会向前端发送 "p2p-pairing-complete" 事件
#[tauri::command]
pub fn listen_p2p_pairing_complete<R: Runtime>(
    app_handle: AppHandle<R>,
    event_listener_state: State<'_, Arc<Mutex<EventListenerState>>>,
) {
    // 如果已经在监听，则不再重复监听
    let mut state = event_listener_state.lock().unwrap();
    if state.p2p_pairing_complete_listener_id.is_some() {
        return;
    }

    // 标记为正在监听
    state.p2p_pairing_complete_listener_id = Some(ListenerId(0));
}

/// 停止监听 P2P 配对完成事件
#[tauri::command]
pub fn stop_listen_p2p_pairing_complete(
    event_listener_state: State<'_, Arc<Mutex<EventListenerState>>>,
) {
    let mut state = event_listener_state.lock().unwrap();
    state.p2p_pairing_complete_listener_id = None;
}

/// 监听 P2P 配对失败事件
///
/// 当配对失败时，会向前端发送 "p2p-pairing-failed" 事件
#[tauri::command]
pub fn listen_p2p_pairing_failed<R: Runtime>(
    app_handle: AppHandle<R>,
    event_listener_state: State<'_, Arc<Mutex<EventListenerState>>>,
) {
    // 如果已经在监听，则不再重复监听
    let mut state = event_listener_state.lock().unwrap();
    if state.p2p_pairing_failed_listener_id.is_some() {
        return;
    }

    // 标记为正在监听
    state.p2p_pairing_failed_listener_id = Some(ListenerId(0));
}

/// 停止监听 P2P 配对失败事件
#[tauri::command]
pub fn stop_listen_p2p_pairing_failed(
    event_listener_state: State<'_, Arc<Mutex<EventListenerState>>>,
) {
    let mut state = event_listener_state.lock().unwrap();
    state.p2p_pairing_failed_listener_id = None;
}
