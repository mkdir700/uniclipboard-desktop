use crate::application::device_service::{get_device_manager, GLOBAL_DEVICE_MANAGER};
use crate::config::Setting;
use crate::domain::transfer_message::ClipboardTransferMessage;
use crate::domain::device::Device;
use crate::domain::network::{ConnectionRequestMessage, ConnectionResponseMessage};
use crate::infrastructure::connection::connection_manager::ConnectionManager;
use crate::infrastructure::connection::DeviceId;
use crate::infrastructure::event::event_bus::{
    publish_connection_request, publish_connection_response,
};
use crate::message::{DeviceSyncInfo, DevicesSyncMessage, RegisterDeviceMessage, WebSocketMessage};
use log::{debug, error, info};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::oneshot;
use anyhow::Result;

pub struct MessageHandler {
    connection_manager: Arc<ConnectionManager>,
}

impl MessageHandler {
    pub fn new(connection_manager: Arc<ConnectionManager>) -> Self {
        Self { connection_manager }
    }

    /// 处理剪贴板同步消息
    /// 将消息集中到一个 channel 中, 然后由上层获取
    pub async fn handle_clipboard_sync(
        &self,
        data: ClipboardTransferMessage,
        message_source: MessageSource,
    ) {
        info!("Received clipboard sync message from {:?}", message_source);
        self.connection_manager.send_clipboard_sync(data).await;
    }

    /// 处理设备列表同步消息
    /// 合并设备列表, 并广播给其他设备
    pub async fn handle_device_list_sync(
        &self,
        mut data: DevicesSyncMessage,
        message_source: MessageSource,
    ) {
        info!(
            "Received device list sync message from {:?}",
            message_source
        );

        let devices = data
            .devices
            .iter()
            .map(|d| {
                let mut device = Device::new(
                    d.id.clone(),
                    d.ip.clone(),
                    d.server_port.clone(),
                    d.server_port.clone(),
                );
                device.status = d.status;
                device.updated_at = d.updated_at;
                device
            })
            .collect();

        if let Err(e) = GLOBAL_DEVICE_MANAGER.merge(&devices) {
            error!("Failed to merge devices: {}", e);
        }

        let device_id = Setting::get_instance().get_device_id().clone();

        if data.replay_device_ids.contains(&device_id) {
            debug!(
                "Device {} is already in replay_device_ids, skip...",
                device_id
            );
            return;
        }

        data.replay_device_ids.push(device_id.clone());
        let excludes = data.replay_device_ids.clone();

        let devices = {
            if let Ok(devices) = get_device_manager().get_all_devices() {
                devices
            } else {
                error!("Failed to get all devices");
                return;
            }
        };

        info!(
            "Broadcasting device list sync to others, excludes: {:?}",
            excludes
        );

        let device_sync_infos: Vec<DeviceSyncInfo> =
            devices.iter().map(|d| DeviceSyncInfo::from(d)).collect();

        let _ = self
            .connection_manager
            .broadcast(
                &WebSocketMessage::DeviceListSync(DevicesSyncMessage::new(
                    device_sync_infos,
                    data.replay_device_ids,
                )),
                &Some(excludes),
            )
            .await;
    }

    /// 处理设备注册消息
    /// 建立 ip端口与设备id的映射关系
    pub async fn handle_register(
        &self,
        register_device_message: RegisterDeviceMessage,
        addr: SocketAddr,
    ) {
        let device_id = register_device_message.id.clone();
        let ip_port = format!("{}:{}", addr.ip(), addr.port());

        // 设置设备在线
        if let Err(e) = GLOBAL_DEVICE_MANAGER.set_online(&device_id) {
            error!("Failed to set device {} online: {}", device_id, e);
        }

        self.connection_manager
            .update_device_ip_port(device_id, ip_port)
            .await;
    }

    /// 处理设备注销消息
    /// 删除设备 IP 和端口映射
    pub async fn handle_unregister(&self, device_id: DeviceId) {
        info!("Received device unregister message from {:?}", device_id);
        todo!()
    }

    /// 处理连接请求消息
    ///
    /// 当收到其他设备的连接请求时，记录并通知前端
    pub async fn handle_connection_request(
        &self,
        request: ConnectionRequestMessage,
        addr: SocketAddr,
    ) {
        info!(
            "Received connection request from device {} at {}",
            request.requester_device_id,
            addr
        );

        // 1. 将请求存储到待处理列表
        let (response_tx, _response_rx) = oneshot::channel();
        if let Err(e) = self
            .connection_manager
            .pending_connections
            .add_incoming_request(request.clone(), response_tx)
            .await
        {
            error!("Failed to store incoming connection request: {}", e);
            return;
        }

        // 2. 通过 event_bus 发布事件，通知前端
        publish_connection_request(
            request.requester_device_id.clone(),
            request.requester_ip.clone(),
            request.requester_alias.clone(),
            request.requester_platform.clone(),
        );

        info!(
            "Connection request from {} stored, event published, waiting for user response",
            request.requester_device_id
        );

        // 3. 等待用户响应（通过 respond_to_connection_request API 命令）
    }

    /// 响应入站连接请求
    ///
    /// 当用户在前端确认或拒绝连接请求时，调用此方法发送响应
    pub async fn respond_to_incoming_request(
        &self,
        requester_device_id: &str,
        accept: bool,
    ) -> Result<()> {
        // 获取响应通道
        let response_tx = self
            .connection_manager
            .pending_connections
            .respond_to_incoming_request(requester_device_id, accept)
            .await?;

        // 发送响应到等待的协程
        let _ = response_tx.send(accept);

        Ok(())
    }

    /// 处理连接响应消息
    ///
    /// 当对方响应连接请求时，完成设备注册和同步
    pub async fn handle_connection_response(
        &self,
        response: ConnectionResponseMessage,
        message_source: MessageSource,
    ) {
        info!(
            "Received connection response from device {}: accepted={}",
            response.responder_device_id,
            response.accepted
        );

        // 获取发起连接时的源地址
        let source_addr = match &message_source {
            MessageSource::IpPort(addr) => format!("{}:{}", addr.ip(), addr.port()),
            MessageSource::DeviceId(_device_id) => {
                // 如果是设备 ID，我们暂时忽略这个情况
                // 实际上连接响应应该总是来自 IpPort
                error!("Connection response from DeviceId not supported");
                return;
            }
        };

        // 将响应传递给待处理连接管理器
        if let Err(e) = self
            .connection_manager
            .pending_connections
            .handle_outgoing_response(&source_addr, response.clone())
            .await
        {
            error!("Failed to handle outgoing response: {}", e);
            return;
        }

        // 通过 event_bus 发布事件，通知前端
        publish_connection_response(
            response.accepted,
            response.responder_device_id.clone(),
            response.responder_ip.clone(),
            response.responder_alias.clone(),
        );

        // 如果连接被接受，需要将设备添加到设备列表
        if response.accepted {
            // 设置设备在线
            if let Err(e) = GLOBAL_DEVICE_MANAGER
                .set_online(&response.responder_device_id)
            {
                error!(
                    "Failed to set device {} online: {}",
                    response.responder_device_id, e
                );
            }

            info!(
                "Connection accepted by device {}",
                response.responder_device_id
            );
        } else {
            info!(
                "Connection rejected by device {}",
                response.responder_device_id
            );
        }
    }
}

#[derive(Debug, Clone)]
pub enum MessageSource {
    IpPort(SocketAddr),
    DeviceId(DeviceId),
}
