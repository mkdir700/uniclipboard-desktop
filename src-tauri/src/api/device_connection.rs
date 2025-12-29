//! 设备连接相关的 Tauri API 命令
//!
//! 提供手动连接设备、获取网络接口等功能

use std::sync::{Arc, Mutex};

use tauri::{Runtime, State};
use log::{error, info};

use crate::config::Setting;
use crate::domain::network::*;
use crate::infrastructure::uniclipboard::UniClipboard;
use crate::infrastructure::web::handlers::message_handler::MessageHandler;
use crate::utils::helpers;

/// 获取所有本地网络接口
///
/// 返回所有非回环的 IPv4 地址，用于显示本机可用的网络接口
#[tauri::command]
pub fn get_local_network_interfaces() -> Result<Vec<NetworkInterface>, String> {
    Ok(helpers::get_local_network_interfaces())
}

/// 手动连接到指定设备
///
/// 发起连接请求到指定 IP 和端口的设备
/// 成功连接后发送 ConnectionRequestMessage，等待对方确认
#[tauri::command]
pub async fn connect_to_device_manual(
    request: ManualConnectionRequest,
    uniclipboard_app: State<'_, Arc<Mutex<Option<Arc<UniClipboard>>>>>,
) -> Result<ManualConnectionResponse, String> {
    info!(
        "Manual connection request to {}:{}",
        request.ip, request.port
    );

    // 验证 IP 地址
    if !helpers::is_valid_ip(&request.ip) {
        return Ok(ManualConnectionResponse {
            success: false,
            device_id: None,
            message: "请输入有效的 IPv4 地址".to_string(),
        });
    }

    // 验证端口
    if !helpers::is_valid_port(request.port) {
        return Ok(ManualConnectionResponse {
            success: false,
            device_id: None,
            message: "端口号必须在 1024-65535 之间".to_string(),
        });
    }

    // 获取 UniClipboard 实例
    let uniclipboard = uniclipboard_app
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?
        .as_ref()
        .cloned()
        .ok_or("UniClipboard not initialized")?;

    // 获取 connection_manager
    let connection_manager = uniclipboard.get_connection_manager();

    // 执行手动连接
    match connection_manager
        .unified
        .connect_with_peer_device(&request.ip, request.port)
        .await
    {
        Ok(_) => {
            info!(
                "Successfully connected to device {}:{} via manual connection",
                request.ip, request.port
            );
            Ok(ManualConnectionResponse {
                success: true,
                device_id: Some(format!("{}:{}", request.ip, request.port)),
                message: "连接成功".to_string(),
            })
        }
        Err(e) => {
            error!("Manual connection failed: {}", e);
            Ok(ManualConnectionResponse {
                success: false,
                device_id: None,
                message: format!("连接失败: {}", e),
            })
        }
    }
}

/// 响应连接请求（接受或拒绝）
///
/// 当收到其他设备的连接请求时，用户可以接受或拒绝
#[tauri::command]
pub async fn respond_to_connection_request(
    decision: ConnectionRequestDecision,
    uniclipboard_app: State<'_, Arc<Mutex<Option<Arc<UniClipboard>>>>>,
) -> Result<ManualConnectionResponse, String> {
    info!(
        "Connection request decision for device {}: {}",
        decision.requester_device_id,
        if decision.accept { "accept" } else { "reject" }
    );

    // 获取 UniClipboard 实例
    let uniclipboard = uniclipboard_app
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?
        .as_ref()
        .cloned()
        .ok_or("UniClipboard not initialized")?;

    // 获取 connection_manager
    let connection_manager = uniclipboard.get_connection_manager();

    // 创建 MessageHandler
    let message_handler = MessageHandler::new(connection_manager.clone());

    // 响应入站请求
    match message_handler
        .respond_to_incoming_request(&decision.requester_device_id, decision.accept)
        .await
    {
        Ok(_) => {
            info!(
                "Successfully responded to connection request from {}",
                decision.requester_device_id
            );

            // 如果接受，发送 ConnectionResponseMessage
            if decision.accept {
                // 获取待处理请求信息
                let pending_requests = connection_manager
                    .pending_connections
                    .get_incoming_requests()
                    .await;

                // 找到对应的请求（应该已经移除了，需要从其他地方获取）
                // 这里需要发送响应消息到请求方
                let user_setting = Setting::get_instance();
                let response = ConnectionResponseMessage {
                    accepted: true,
                    responder_device_id: user_setting.get_device_id().clone(),
                    responder_ip: None,     // TODO: 获取本地 IP
                    responder_alias: None,  // TODO: 从设置中获取
                };

                // TODO: 发送响应到请求方
                // 需要通过 incoming connections 发送
                info!("Would send connection response: {:?}", response);
            }

            Ok(ManualConnectionResponse {
                success: true,
                device_id: None,
                message: if decision.accept {
                    "已接受连接".to_string()
                } else {
                    "已拒绝连接".to_string()
                },
            })
        }
        Err(e) => {
            error!("Failed to respond to connection request: {}", e);
            Ok(ManualConnectionResponse {
                success: false,
                device_id: None,
                message: format!("响应失败: {}", e),
            })
        }
    }
}

/// 取消待处理的连接请求
#[tauri::command]
pub async fn cancel_connection_request(
    uniclipboard_app: State<'_, Arc<Mutex<Option<Arc<UniClipboard>>>>>,
) -> Result<(), String> {
    info!("Cancel connection request");

    // 获取 UniClipboard 实例
    let uniclipboard = uniclipboard_app
        .lock()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?
        .as_ref()
        .cloned()
        .ok_or("UniClipboard not initialized")?;

    // 获取 connection_manager
    let connection_manager = uniclipboard.get_connection_manager();

    // 清空所有待处理连接
    connection_manager.pending_connections.clear_all().await;

    Ok(())
}
