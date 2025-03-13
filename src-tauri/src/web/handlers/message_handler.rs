use crate::config::CONFIG;
use crate::connection::{ConnectionManager, DeviceId};
use crate::device::{get_device_manager, Device, GLOBAL_DEVICE_MANAGER};
use crate::message::{
    ClipboardSyncMessage, DeviceSyncInfo, DevicesSyncMessage, RegisterDeviceMessage,
    WebSocketMessage,
};
use log::{debug, error, info};
use std::net::SocketAddr;
use std::sync::Arc;

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
        data: ClipboardSyncMessage,
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

        let device_id = {
            let config = CONFIG
                .read()
                .map_err(|e| anyhow::anyhow!("Failed to read config: {}", e))
                .unwrap();
            config.device_id.clone()
        };

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
}

#[derive(Debug, Clone)]
pub enum MessageSource {
    IpPort(SocketAddr),
    DeviceId(DeviceId),
}
