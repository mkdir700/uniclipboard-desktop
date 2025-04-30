use std::sync::Arc;

use crate::infrastructure::storage::file_storage::FileStorageManager;
use crate::infrastructure::storage::record_manager::ClipboardRecordManager;

use crate::infrastructure::event::publish_clipboard_new_content;
use crate::domain::transfer_message::ClipboardTransferMessage;
use crate::application::device_service::get_device_manager;
use crate::message::Payload;
use anyhow::Result;
use log::info;
use reqwest_dav::re_exports::reqwest;

pub struct ClipboardContentReceiver {
    file_storage: Arc<FileStorageManager>,
    record_manager: Arc<ClipboardRecordManager>,
}

impl ClipboardContentReceiver {
    pub fn new(
        file_storage: Arc<FileStorageManager>,
        record_manager: Arc<ClipboardRecordManager>,
    ) -> Self {
        Self {
            file_storage,
            record_manager,
        }
    }

    pub async fn receive(&self, message: ClipboardTransferMessage) -> Result<Payload> {
        let manager = get_device_manager();
        let device = manager.get(&message.sender_id)?;

        match device {
            Some(device) => {
                let ip = device.ip.ok_or(anyhow::anyhow!("Device IP not found"))?;
                let port = device
                    .server_port
                    .ok_or(anyhow::anyhow!("Device server port not found"))?;
                let url = format!("http://{}:{}/api/download/{}", ip, port, message.record_id);

                // 发送HTTP请求下载文件
                info!("Downloading file from: {}", url);
                let response = reqwest::get(&url).await?;

                if !response.status().is_success() {
                    return Err(anyhow::anyhow!(
                        "Failed to download file, status: {}",
                        response.status()
                    ));
                }

                // 读取响应体内容
                let bytes = response.bytes().await?;
                info!("Downloaded file with size: {} bytes", bytes.len());

                // 根据内容类型创建对应的Payload
                let payload = Payload::from((&message, bytes));

                // 存储下载的内容
                let file_path = self.file_storage.store(&payload).await?;
                let metadata = (&payload, &file_path).into();
                // 新增记录
                let record_id = self
                    .record_manager
                    .add_or_update_record_with_metadata(&metadata)
                    .await?;

                info!("Downloaded content stored at: {:?}", file_path);

                // 发布剪贴板新内容事件
                publish_clipboard_new_content(record_id);

                Ok(payload)
            }
            None => {
                return Err(anyhow::anyhow!("Device not found"));
            }
        }
    }
}
