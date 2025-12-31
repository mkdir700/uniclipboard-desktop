//! 设备连接相关的 Tauri API 命令
//!
//! 提供手动连接设备、获取网络接口等功能

use log::{error, info};
use tauri::State;
use tokio::sync::oneshot;

use crate::domain::network::*;
use crate::infrastructure::runtime::{AppRuntimeHandle, ConnectionCommand};
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
    app_handle: State<'_, AppRuntimeHandle>,
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

    let (tx, rx) = oneshot::channel();

    // 发送命令到 AppRuntime
    if let Err(e) = app_handle
        .connection_tx
        .send(ConnectionCommand::ManualConnect {
            ip: request.ip.clone(),
            port: request.port,
            respond_to: tx,
        })
        .await
    {
        error!("Failed to send connection command: {}", e);
        return Err("Internal service error".to_string());
    }

    // 等待结果
    match rx.await {
        Ok(Ok(device_id)) => {
            info!(
                "Successfully connected to device {} via manual connection",
                device_id
            );
            Ok(ManualConnectionResponse {
                success: true,
                device_id: Some(device_id),
                message: "连接成功".to_string(),
            })
        }
        Ok(Err(e)) => {
            error!("Manual connection failed: {}", e);
            Ok(ManualConnectionResponse {
                success: false,
                device_id: None,
                message: format!("连接失败: {}", e),
            })
        }
        Err(_) => Err("Connection service did not respond".to_string()),
    }
}

/// 响应连接请求（接受或拒绝）
///
/// 当收到其他设备的连接请求时，用户可以接受或拒绝
#[tauri::command]
pub async fn respond_to_connection_request(
    decision: ConnectionRequestDecision,
    app_handle: State<'_, AppRuntimeHandle>,
) -> Result<ManualConnectionResponse, String> {
    info!(
        "Connection request decision for device {}: {}",
        decision.requester_device_id,
        if decision.accept { "accept" } else { "reject" }
    );

    let (tx, rx) = oneshot::channel();

    if let Err(e) = app_handle
        .connection_tx
        .send(ConnectionCommand::RespondConnection {
            requester_device_id: decision.requester_device_id.clone(),
            accept: decision.accept,
            respond_to: tx,
        })
        .await
    {
        error!("Failed to send respond connection command: {}", e);
        return Err("Internal service error".to_string());
    }

    match rx.await {
        Ok(Ok(_)) => Ok(ManualConnectionResponse {
            success: true,
            device_id: None,
            message: if decision.accept {
                "已接受连接".to_string()
            } else {
                "已拒绝连接".to_string()
            },
        }),
        Ok(Err(e)) => {
            error!("Failed to respond to connection request: {}", e);
            Ok(ManualConnectionResponse {
                success: false,
                device_id: None,
                message: format!("响应失败: {}", e),
            })
        }
        Err(_) => Err("Service did not respond".to_string()),
    }
}

/// 取消待处理的连接请求
#[tauri::command]
pub async fn cancel_connection_request(
    app_handle: State<'_, AppRuntimeHandle>,
) -> Result<(), String> {
    info!("Cancel connection request");

    let (tx, rx) = oneshot::channel();

    if let Err(e) = app_handle
        .connection_tx
        .send(ConnectionCommand::CancelConnectionRequest { respond_to: tx })
        .await
    {
        return Err(format!("Failed to send command: {}", e));
    }

    match rx.await {
        Ok(result) => result,
        Err(_) => Err("Service did not respond".to_string()),
    }
}
