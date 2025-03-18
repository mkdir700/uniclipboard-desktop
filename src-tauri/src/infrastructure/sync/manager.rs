use std::{sync::Arc, time::Duration};

use crate::config::Setting;
use crate::interface::{RemoteClipboardSync as RemoteClipboardSyncTrait, RemoteSyncManagerTrait};
use crate::core::transfer::ClipboardTransferMessage;
use anyhow::Result;
use tokio::sync::RwLock;
use async_trait::async_trait;

pub struct RemoteSyncManager {
    sync_handler: Arc<RwLock<Option<Arc<dyn RemoteClipboardSyncTrait>>>>,
    user_setting: Setting,
}

impl RemoteSyncManager {
    pub fn new() -> Self {
        RemoteSyncManager {
            sync_handler: Arc::new(RwLock::new(None)),
            user_setting: Setting::get_instance(),
        }
    }

    /// 创建一个新的 RemoteSyncManager 实例，使用指定的配置
    pub fn with_user_setting(user_setting: Setting) -> Self {
        RemoteSyncManager {
            sync_handler: Arc::new(RwLock::new(None)),
            user_setting,
        }
    }
}

#[async_trait]
impl RemoteSyncManagerTrait for RemoteSyncManager {
    async fn set_sync_handler(&self, handler: Arc<dyn RemoteClipboardSyncTrait>) {
        let mut sync_handler = self.sync_handler.write().await;
        *sync_handler = Some(handler);
    }

    async fn push(&self, message: ClipboardTransferMessage) -> Result<()> {
        let sync_handler = self.sync_handler.read().await;
        if let Some(handler) = sync_handler.as_ref() {
            handler.push(message).await
        } else {
            Err(anyhow::anyhow!("No sync handler set"))
        }
    }

    async fn pull(&self, timeout: Option<Duration>) -> Result<ClipboardTransferMessage> {
        let sync_handler = self.sync_handler.read().await;
        if let Some(handler) = sync_handler.as_ref() {
            handler.pull(timeout).await
        } else {
            Err(anyhow::anyhow!("No sync handler set"))
        }
    }

    #[allow(dead_code)]
    async fn sync(&self) -> Result<()> {
        let sync_handler = self.sync_handler.read().await;
        if let Some(handler) = sync_handler.as_ref() {
            handler.sync().await
        } else {
            Err(anyhow::anyhow!("No sync handler set"))
        }
    }

    async fn start(&self) -> Result<()> {
        let sync_handler = self.sync_handler.read().await;
        if let Some(handler) = sync_handler.as_ref() {
            handler.start().await
        } else {
            Err(anyhow::anyhow!("No sync handler set"))
        }
    }

    #[allow(dead_code)]
    async fn stop(&self) -> Result<()> {
        let sync_handler = self.sync_handler.read().await;
        if let Some(handler) = sync_handler.as_ref() {
            handler.stop().await
        } else {
            Err(anyhow::anyhow!("No sync handler set"))
        }
    }

    async fn pause(&self) -> Result<()> {
        let sync_handler = self.sync_handler.read().await;
        if let Some(handler) = sync_handler.as_ref() {
            handler.pause().await
        } else {
            Err(anyhow::anyhow!("No sync handler set"))
        }
    }

    async fn resume(&self) -> Result<()> {
        let sync_handler = self.sync_handler.read().await;
        if let Some(handler) = sync_handler.as_ref() {
            handler.resume().await
        } else {
            Err(anyhow::anyhow!("No sync handler set"))
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::remote_sync::traits::MockRemoteClipboardSync;
//     use crate::message::{ClipboardSyncMessage, Payload};

//     use super::*;
//     use chrono::Utc;
//     use mockall::predicate::*;
//     use std::sync::Arc;
//     use bytes::Bytes;

//     #[tokio::test]
//     async fn test_set_sync_handler() {
//         let manager = RemoteSyncManager::new();
//         let mock_handler = Arc::new(MockRemoteClipboardSync::new());
//         manager.set_sync_handler(mock_handler.clone()).await;
//         assert!(manager.sync_handler.read().await.is_some());
//     }

//     #[tokio::test]
//     async fn test_push() {
//         let manager = RemoteSyncManager::new();
//         let mut mock_handler = MockRemoteClipboardSync::new();
//         let payload = Payload::new_text(
//             Bytes::from("test_data".to_string()),
//             "text/plain".to_string(),
//             Utc::now(),
//         );
//         let message = ClipboardSyncMessage::from_payload(payload);
    
//         mock_handler
//             .expect_push()
//             .withf(|m: &ClipboardSyncMessage| {
//                 // 检查消息的关键属性
//                 m.payload.is_some()
//             })
//             .times(1)
//             .returning(|_| Ok(()));
    
//         manager.set_sync_handler(Arc::new(mock_handler)).await;
//         assert!(manager.push(message).await.is_ok());
//     }

//     #[tokio::test]
//     async fn test_pull() {
//         let manager = RemoteSyncManager::new();
//         let mut mock_handler = MockRemoteClipboardSync::new();
//         let payload = Payload::new_text(
//             Bytes::from("test_data".to_string()),
//             "text/plain".to_string(),
//             Utc::now(),
//         );
//         let message = ClipboardSyncMessage::from_payload(payload.clone());

//         mock_handler
//             .expect_pull()
//             .with(eq(None))
//             .times(1)
//             .returning(move |_| Ok(message.clone()));

//         manager.set_sync_handler(Arc::new(mock_handler)).await;
//         let received_payload = manager.pull(None).await.unwrap();
//         assert_eq!(payload, received_payload.payload.unwrap());
//     }

//     #[tokio::test]
//     async fn test_start() {
//         let manager = RemoteSyncManager::new();
//         let mut mock_handler = MockRemoteClipboardSync::new();
        
//         mock_handler
//             .expect_start()
//             .times(1)
//             .returning(|| Ok(()));

//         manager.set_sync_handler(Arc::new(mock_handler)).await;
//         assert!(manager.start().await.is_ok());
//     }
    
//     #[tokio::test]
//     async fn test_stop() {
//         let manager = RemoteSyncManager::new();
//         let mut mock_handler = MockRemoteClipboardSync::new();
        
//         mock_handler
//             .expect_stop()
//             .times(1)
//             .returning(|| Ok(()));

//         manager.set_sync_handler(Arc::new(mock_handler)).await;
//         assert!(manager.stop().await.is_ok());
//     }
    
//     #[tokio::test]
//     async fn test_pause() {
//         let manager = RemoteSyncManager::new();
//         let mut mock_handler = MockRemoteClipboardSync::new();
        
//         mock_handler
//             .expect_pause()
//             .times(1)
//             .returning(|| Ok(()));

//         manager.set_sync_handler(Arc::new(mock_handler)).await;
//         assert!(manager.pause().await.is_ok());
//     }
    
//     #[tokio::test]
//     async fn test_resume() {
//         let manager = RemoteSyncManager::new();
//         let mut mock_handler = MockRemoteClipboardSync::new();
        
//         mock_handler
//             .expect_resume()
//             .times(1)
//             .returning(|| Ok(()));

//         manager.set_sync_handler(Arc::new(mock_handler)).await;
//         assert!(manager.resume().await.is_ok());
//     }
// }