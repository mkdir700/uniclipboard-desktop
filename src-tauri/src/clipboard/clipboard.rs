use crate::config::Config;
use crate::message::Payload;
use anyhow::Result;
use bytes::Bytes;
use chrono::Utc;
use clipboard_rs::common::RustImage;
use clipboard_rs::{Clipboard, ClipboardContext};
use clipboard_rs::{ClipboardHandler, RustImageData};
use log::debug;
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;

pub struct RsClipboard(Arc<Mutex<dyn ClipboardContextTrait>>, Arc<Notify>, Config);

pub struct RsClipboardChangeHandler(Arc<Notify>);

pub struct ClipboardContextWrapper(ClipboardContext);

// 定义一个 trait 来抽象 ClipboardContext 的行为
pub trait ClipboardContextTrait: Send + Sync {
    fn get_text(&self) -> Result<String>;
    fn set_text(&self, text: String) -> Result<()>;
    fn get_image(&self) -> Result<RustImageData>;
    fn set_image(&self, image: RustImageData) -> Result<()>;
}

impl RsClipboardChangeHandler {
    pub fn new(notify: Arc<Notify>) -> Self {
        Self(notify)
    }
}

impl ClipboardHandler for RsClipboardChangeHandler {
    fn on_clipboard_change(&mut self) {
        debug!("Clipboard changed");
        self.0.notify_waiters();
    }
}

impl RsClipboard {
    fn clipboard(&self) -> Arc<Mutex<dyn ClipboardContextTrait>> {
        self.0.clone()
    }

    fn read_text(&self) -> Result<String> {
        let clipboard = self.clipboard();
        let guard = clipboard
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock clipboard: {}", e))?;
        guard
            .get_text()
            .map_err(|e| anyhow::anyhow!("Failed to read text: {}", e))
    }

    fn write_text(&self, text: &str) -> Result<()> {
        let clipboard = self.clipboard();
        let guard = clipboard
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock clipboard: {}", e))?;
        guard
            .set_text(text.to_string())
            .map_err(|e| anyhow::anyhow!("Failed to write text: {}", e))
    }

    fn read_image(&self) -> Result<RustImageData> {
        let clipboard = self.clipboard();
        let guard = clipboard
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock clipboard: {}", e))?;
        let image = guard
            .get_image()
            .map_err(|e| anyhow::anyhow!("Failed to read image: {}", e))?;
        Ok(image)
    }

    fn write_image(&self, image: RustImageData) -> Result<()> {
        let clipboard = self.clipboard();
        let guard = clipboard
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock clipboard: {}", e))?;
        guard
            .set_image(image)
            .map_err(|e| anyhow::anyhow!("Failed to set image: {}", e))
    }

    pub fn read(&self) -> Result<Payload> {
        if let Ok(image) = self.read_image() {
            let (width, height) = image.get_size();
            let png_bytes = {
                let png_buffer = image.to_png().unwrap();
                png_buffer.get_bytes().to_vec()
            };
            let size = png_bytes.len();
            let device_id = self.2.get_device_id();
            Ok(Payload::new_image(
                Bytes::from(png_bytes),
                device_id,
                Utc::now(),
                width as usize,
                height as usize,
                "png".to_string(),
                size,
            ))
        } else if let Ok(text) = self.read_text() {
            let device_id = self.2.get_device_id();
            Ok(Payload::new_text(
                Bytes::from(text),
                device_id,
                Utc::now(),
            ))
        } else {
            Err(anyhow::anyhow!("Clipboard is empty"))
        }
    }

    pub fn write(&self, payload: Payload) -> Result<()> {
        match payload {
            Payload::Image(image) => {
                let image_data = RustImageData::from_bytes(&image.content.to_vec())
                    .map_err(|e| anyhow::anyhow!("Failed to convert image: {}", e))?;
                self.write_image(image_data)
            }
            Payload::Text(text) => {
                let text = String::from_utf8(text.content.to_vec())?;
                self.write_text(&text)
            }
        }
    }

    pub async fn wait_clipboard_change(&self) -> Result<()> {
        self.1.notified().await;
        Ok(())
    }
}

impl ClipboardContextTrait for ClipboardContextWrapper {
    fn get_text(&self) -> Result<String> {
        self.0
            .get_text()
            .map_err(|e| anyhow::anyhow!("Failed to get text: {}", e))
    }

    fn set_text(&self, text: String) -> Result<()> {
        self.0
            .set_text(text)
            .map_err(|e| anyhow::anyhow!("Failed to set text: {}", e))
    }

    #[cfg(not(target_os = "windows"))]
    fn get_image(&self) -> Result<RustImageData> {
        self.0
            .get_image()
            .map_err(|e| anyhow::anyhow!("Failed to get image: {}", e))
    }

    #[cfg(not(target_os = "windows"))]
    fn set_image(&self, image: RustImageData) -> Result<()> {
        self.0
            .set_image(image)
            .map_err(|e| anyhow::anyhow!("Failed to set image: {}", e))
    }

    #[cfg(target_os = "windows")]
    fn get_image(&self) -> Result<RustImageData> {
        use clipboard_win::{formats, get_clipboard};
        let data = get_clipboard(formats::Bitmap)
            .map_err(|e| anyhow::anyhow!("Failed to get image: {}", e))?;
        RustImageData::from_bytes(&data)
            .map_err(|e| anyhow::anyhow!("Failed to convert image: {}", e))
    }

    #[cfg(target_os = "windows")]
    fn set_image(&self, image: RustImageData) -> Result<()> {
        use super::utils::PlatformImage;
        use clipboard_win::{empty, formats, set_clipboard};
        use std::ptr::null_mut;
        use winapi::um::winuser::{CloseClipboard, GetOpenClipboardWindow, OpenClipboard};

        let platform_image = PlatformImage::new(
            image
                .get_dynamic_image()
                .map_err(|e| anyhow::anyhow!("Failed to get dynamic image: {}", e))?,
        );
        let bmp_bytes = platform_image.to_bitmap();

        // 尝试打开剪贴板
        let mut retry_count = 0;
        while retry_count < 5 {
            unsafe {
                if OpenClipboard(null_mut()) != 0 {
                    // 剪贴板成功打开
                    break;
                }
                // 如果剪贴板已经被其他进程打开，等待一段时间后重试
                if GetOpenClipboardWindow() != null_mut() {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    retry_count += 1;
                } else {
                    // 如果剪贴板没有被其他进程打开但仍然失败，返回错误
                    return Err(anyhow::anyhow!("Failed to open clipboard"));
                }
            }
        }

        if retry_count == 5 {
            return Err(anyhow::anyhow!(
                "Failed to open clipboard after multiple attempts"
            ));
        }

        // 清空剪贴板
        let clear_result = empty();

        // 设置新的剪贴板内容
        let set_result = set_clipboard(formats::Bitmap, &bmp_bytes);

        // 关闭剪贴板
        unsafe {
            CloseClipboard();
        }

        // 检查操作结果
        clear_result.map_err(|e| anyhow::anyhow!("Failed to clear clipboard: {}", e))?;
        set_result.map_err(|e| anyhow::anyhow!("Failed to set clipboard: {}", e))?;

        Ok(())
    }
}

impl RsClipboard {
    #[cfg(not(test))]
    pub fn new(notify: Arc<Notify>) -> Result<Self> {
        Ok(Self(
            Arc::new(Mutex::new(ClipboardContextWrapper(
                ClipboardContext::new()
                    .map_err(|e| anyhow::anyhow!("Failed to create clipboard context: {}", e))?,
            ))),
            notify,
            Config::get_instance(),
        ))
    }

    /// 创建一个新的 RsClipboard 实例，使用指定的配置
    #[cfg(not(test))]
    pub fn with_config(notify: Arc<Notify>, config: Config) -> Result<Self> {
        Ok(Self(
            Arc::new(Mutex::new(ClipboardContextWrapper(
                ClipboardContext::new()
                    .map_err(|e| anyhow::anyhow!("Failed to create clipboard context: {}", e))?,
            ))),
            notify,
            config,
        ))
    }

    #[cfg(test)]
    pub fn new(notify: Arc<Notify>) -> Result<Self> {
        use self::mock::MockClipboardContext;
        Ok(Self(
            Arc::new(Mutex::new(MockClipboardContext::new())),
            notify,
            Config::get_instance(),
        ))
    }

    /// 创建一个新的 RsClipboard 实例，使用指定的配置（测试用）
    #[cfg(test)]
    pub fn with_config(notify: Arc<Notify>, config: Config) -> Result<Self> {
        use self::mock::MockClipboardContext;
        Ok(Self(
            Arc::new(Mutex::new(MockClipboardContext::new())),
            notify,
            config,
        ))
    }
}

// 在 mock 模块中创建 MockClipboardContext
#[cfg(test)]
mod mock {
    use super::*;
    use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba};

    pub struct MockClipboardContext {
        text: Mutex<String>,
        image: Mutex<Option<MockImageData>>,
    }

    #[derive(Clone)]
    struct MockImageData {
        width: u32,
        height: u32,
        data: Vec<u8>,
    }

    impl MockClipboardContext {
        pub fn new() -> Self {
            Self {
                text: Mutex::new(String::new()),
                image: Mutex::new(None),
            }
        }
    }

    impl ClipboardContextTrait for MockClipboardContext {
        fn get_text(&self) -> Result<String> {
            Ok(self.text.lock().unwrap().clone())
        }

        fn set_text(&self, text: String) -> Result<()> {
            *self.text.lock().unwrap() = text;
            Ok(())
        }

        fn get_image(&self) -> Result<RustImageData> {
            let image_data = self.image.lock().unwrap().clone();
            match image_data {
                Some(data) => {
                    let img = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(
                        data.width,
                        data.height,
                        data.data,
                    )
                    .ok_or_else(|| anyhow::anyhow!("Failed to create image buffer"))?;
                    let dynamic_image = DynamicImage::ImageRgba8(img);
                    Ok(RustImageData::from_dynamic_image(dynamic_image))
                }
                None => Err(anyhow::anyhow!("No image")),
            }
        }

        fn set_image(&self, image: RustImageData) -> Result<()> {
            let dynamic_image = image.get_dynamic_image().unwrap();
            let (width, height) = dynamic_image.dimensions();
            let rgba_image = dynamic_image.to_rgba8();
            let data = rgba_image.into_raw();

            *self.image.lock().unwrap() = Some(MockImageData {
                width,
                height,
                data,
            });
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;
    use std::path::PathBuf;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_read_write_text() {
        let clipboard = RsClipboard::new(Arc::new(Notify::new())).unwrap();
        clipboard.write_text("Hello, world!").unwrap();
        let text = clipboard.read_text().unwrap();
        assert_eq!(text, "Hello, world!");
    }

    #[tokio::test]
    async fn test_read_write_image() {
        let clipboard = RsClipboard::new(Arc::new(Notify::new())).unwrap();
        // 从 test_resources 目录中读取图片
        // 构建测试资源文件的路径
        let mut test_image_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_image_path.push("test_resources");
        test_image_path.push("moon.jpg");
        // 判断文件是否存在
        assert!(test_image_path.exists());
        println!("test_image_path: {}", test_image_path.to_str().unwrap());

        // let img = ImageReader::open(&test_image_path).unwrap().decode().unwrap();
        let image_bytes = fs::read(&test_image_path).unwrap();
        let image_data = RustImageData::from_bytes(&image_bytes).unwrap();
        let size = image_data.get_size();

        clipboard.write_image(image_data).unwrap();

        let read_image = clipboard.read_image().unwrap();
        assert_eq!(read_image.get_size(), size);
    }

    #[tokio::test]
    async fn test_wait_clipboard_change() {
        let notify = Arc::new(Notify::new());
        let clipboard = RsClipboard::new(notify.clone()).unwrap();

        // 创建一个任务来模拟剪贴板变化
        let trigger_change = tokio::spawn(async move {
            // 等待一小段时间，确保主任务已经开始等待
            sleep(Duration::from_millis(100)).await;
            notify.notify_one();
        });

        // 等待剪贴板变化
        let wait_result =
            tokio::time::timeout(Duration::from_secs(1), clipboard.wait_clipboard_change()).await;

        // 确保触发任务完成
        trigger_change.await.unwrap();

        // 检查是否成功等待到剪贴板变化
        assert!(
            wait_result.is_ok(),
            "Timed out waiting for clipboard change"
        );
        assert!(
            wait_result.unwrap().is_ok(),
            "Error while waiting for clipboard change"
        );
    }

    #[tokio::test]
    async fn test_write_read_payload_text() {
        let clipboard = RsClipboard::new(Arc::new(Notify::new())).unwrap();
        let text = "Hello, Payload!";
        let payload = Payload::new_text(
            Bytes::from(text.to_string()),
            "test_device".to_string(),
            Utc::now(),
        );
        clipboard.write(payload).unwrap();

        let read_payload = clipboard.read().unwrap();
        match read_payload {
            Payload::Text(text_payload) => {
                assert_eq!(text_payload.content, Bytes::from(text.to_string()));
            }
            _ => panic!("Expected text payload"),
        }
    }

    #[tokio::test]
    async fn test_write_read_payload_image() {
        let clipboard = RsClipboard::new(Arc::new(Notify::new())).unwrap();
        let mut test_image_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_image_path.push("test_resources");
        test_image_path.push("google.png");

        let image_bytes = fs::read(&test_image_path).unwrap();
        let image_data = RustImageData::from_bytes(&image_bytes).unwrap();
        let (width, height) = image_data.get_size();

        let payload = Payload::new_image(
            Bytes::from(image_bytes.clone()),
            "test_device".to_string(),
            Utc::now(),
            width as usize,
            height as usize,
            "png".to_string(),
            image_bytes.len(),
        );

        clipboard.write(payload).unwrap();

        let read_payload = clipboard.read().unwrap();
        match read_payload {
            Payload::Image(image_payload) => {
                assert_eq!(image_payload.width, width as usize);
                assert_eq!(image_payload.height, height as usize);
                assert_eq!(image_payload.format, "png");
            }
            _ => panic!("Expected image payload"),
        }
    }
}
