use anyhow::Result;
use async_trait::async_trait;
use clipboard_rs::common::RustImage;
use clipboard_rs::{Clipboard, ClipboardContext, RustImageData};
use std::sync::{Arc, Mutex};
use tokio::task::spawn_blocking;
use uc_core::clipboard::{ClipboardContent, ClipboardData, ClipboardItem, MimeType};
use uc_core::ports::LocalClipboardPort;

/// macOS clipboard implementation using clipboard-rs
pub struct MacOSClipboard {
    inner: Arc<Mutex<ClipboardContext>>,
}

impl MacOSClipboard {
    pub fn new() -> Result<Self> {
        let context = ClipboardContext::new()
            .map_err(|e| anyhow::anyhow!("Failed to create clipboard context: {}", e))?;
        Ok(Self {
            inner: Arc::new(Mutex::new(context)),
        })
    }
}

#[async_trait]
impl LocalClipboardPort for MacOSClipboard {
    async fn read(&self) -> Result<ClipboardContent> {
        let inner = self.inner.clone();
        let result = spawn_blocking(move || {
            let guard = inner
                .lock()
                .map_err(|e| anyhow::anyhow!("Failed to lock clipboard: {}", e))?;

            // 按优先级读取: files > image > text (与 legacy 一致)
            if let Ok(files) = guard.get_files() {
                if !files.is_empty() {
                    return Ok(Some((
                        "file/uri-list",
                        ClipboardData::Bytes {
                            bytes: files.join("\n").into_bytes(),
                        },
                    )));
                }
            }

            if let Ok(image) = guard.get_image() {
                let png_bytes = image
                    .to_png()
                    .map_err(|e| anyhow::anyhow!("Failed to convert to PNG: {}", e))?;
                return Ok(Some((
                    "image/png",
                    ClipboardData::Bytes {
                        bytes: png_bytes.get_bytes().to_vec(),
                    },
                )));
            }

            if let Ok(text) = guard.get_text() {
                return Ok(Some(("text/plain", ClipboardData::Text { text })));
            }

            Ok::<Option<(&str, ClipboardData)>, anyhow::Error>(None)
        })
        .await
        .map_err(|e| anyhow::anyhow!("Task join error: {}", e))??;

        match result {
            Some((mime, data)) => Ok(ClipboardContent {
                v: 1,
                ts_ms: chrono::Utc::now().timestamp_millis(),
                items: vec![ClipboardItem {
                    mime: MimeType(mime.to_string()),
                    data,
                    meta: Default::default(),
                }],
                meta: Default::default(),
            }),
            None => Err(anyhow::anyhow!("Clipboard is empty or unsupported type")),
        }
    }

    async fn write(&self, content: ClipboardContent) -> Result<()> {
        let inner = self.inner.clone();
        spawn_blocking(move || {
            let guard = inner
                .lock()
                .map_err(|e| anyhow::anyhow!("Failed to lock clipboard: {}", e))?;

            for item in content.items {
                match item.mime.0.as_str() {
                    "file/uri-list" => {
                        if let ClipboardData::Bytes { bytes } = item.data {
                            let text = String::from_utf8(bytes).map_err(|e| {
                                anyhow::anyhow!("Invalid file path encoding: {}", e)
                            })?;
                            let files: Vec<String> = text.lines().map(|s| s.to_string()).collect();
                            return guard
                                .set_files(files)
                                .map_err(|e| anyhow::anyhow!("Failed to write files: {}", e));
                        }
                    }
                    "image/png" => {
                        if let ClipboardData::Bytes { bytes } = item.data {
                            let image = RustImageData::from_bytes(&bytes)
                                .map_err(|e| anyhow::anyhow!("Failed to decode image: {}", e))?;
                            return guard
                                .set_image(image)
                                .map_err(|e| anyhow::anyhow!("Failed to write image: {}", e));
                        }
                    }
                    "text/plain" => {
                        if let ClipboardData::Text { text } = item.data {
                            return guard
                                .set_text(text)
                                .map_err(|e| anyhow::anyhow!("Failed to write text: {}", e));
                        }
                    }
                    _ => continue,
                }
            }

            Err(anyhow::anyhow!("No supported clipboard content found"))
        })
        .await
        .map_err(|e| anyhow::anyhow!("Task join error: {}", e))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_read_write_text() {
        let clipboard = MacOSClipboard::new().unwrap();

        let content = ClipboardContent {
            v: 1,
            ts_ms: chrono::Utc::now().timestamp_millis(),
            items: vec![ClipboardItem {
                mime: MimeType::text_plain(),
                data: ClipboardData::Text {
                    text: "Hello, macOS!".to_string(),
                },
                meta: Default::default(),
            }],
            meta: Default::default(),
        };

        clipboard.write(content).await.unwrap();

        let read = clipboard.read().await.unwrap();
        assert_eq!(read.items.len(), 1);
        assert_eq!(read.items[0].mime.0, "text/plain");
        if let ClipboardData::Text { text } = &read.items[0].data {
            assert_eq!(text, "Hello, macOS!");
        } else {
            panic!("Expected text data");
        }
    }
}
