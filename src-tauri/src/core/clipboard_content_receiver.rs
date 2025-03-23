use std::sync::Arc;

use crate::infrastructure::storage::record_manager::ClipboardRecordManager;
use crate::{
    core::transfer::ClipboardMetadata, infrastructure::storage::file_storage::FileStorageManager,
};

use super::transfer::ClipboardTransferMessage;
use super::event_bus::publish_clipboard_new_content;
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

                // // 获取Content-Type头，确定内容类型
                // let content_type = response
                //     .headers()
                //     .get("content-type")
                //     .and_then(|h| h.to_str().ok())
                //     .unwrap_or("application/octet-stream");

                // // 获取Content-Length头，确定内容大小
                // let content_length = response
                //     .headers()
                //     .get("content-length")
                //     .and_then(|h| h.to_str().ok())
                //     .and_then(|s| s.parse::<usize>().ok())
                //     .unwrap_or(0);

                // 读取响应体内容
                let bytes = response.bytes().await?;
                info!("Downloaded file with size: {} bytes", bytes.len());

                // 根据内容类型创建对应的Payload
                let payload = message.to_payload(bytes);

                // 存储下载的内容
                let file_path = self.file_storage.store(&payload).await?;
                let metadata = ClipboardMetadata::from_payload(&payload, &file_path);
                // 新增记录
                let record_id = self.record_manager
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
