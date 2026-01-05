use anyhow::Result;
use async_trait::async_trait;
use clipboard_rs::common::RustImage;
use clipboard_rs::{Clipboard, ClipboardContext, RustImageData};
use std::sync::{Arc, Mutex};
use tokio::task::spawn_blocking;
use uc_core::clipboard::{ClipboardContent, ClipboardData, ClipboardItem, MimeType};
use uc_core::ports::ClipboardPort;

/// Windows clipboard implementation using clipboard-rs and clipboard-win
pub struct WindowsClipboard {
    inner: Arc<Mutex<ClipboardContext>>,
}

impl WindowsClipboard {
    pub fn new() -> Result<Self> {
        let context = ClipboardContext::new()
            .map_err(|e| anyhow::anyhow!("Failed to create clipboard context: {}", e))?;
        Ok(Self {
            inner: Arc::new(Mutex::new(context)),
        })
    }
}

#[async_trait]
impl ClipboardPort for WindowsClipboard {
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

            // Windows: 使用 clipboard_win 读取 Bitmap 格式
            #[cfg(target_os = "windows")]
            {
                if let Ok(image) = read_image_windows() {
                    return Ok(Some(("image/bmp", ClipboardData::Bytes { bytes: image })));
                }
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
                    "image/bmp" | "image/png" => {
                        if let ClipboardData::Bytes { bytes } = item.data {
                            #[cfg(target_os = "windows")]
                            {
                                return write_image_windows(&bytes)
                                    .map_err(|e| anyhow::anyhow!("Failed to write image: {}", e));
                            }
                            #[cfg(not(target_os = "windows"))]
                            {
                                let image = RustImageData::from_bytes(&bytes).map_err(|e| {
                                    anyhow::anyhow!("Failed to decode image: {}", e)
                                })?;
                                return guard
                                    .set_image(image)
                                    .map_err(|e| anyhow::anyhow!("Failed to write image: {}", e));
                            }
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

/// Windows-specific: Read image from clipboard as Bitmap format
#[cfg(target_os = "windows")]
fn read_image_windows() -> Result<Vec<u8>> {
    use clipboard_win::{formats, get_clipboard};
    let data = get_clipboard(formats::Bitmap)
        .map_err(|e| anyhow::anyhow!("Failed to get image from clipboard: {}", e))?;

    // Convert BMP to PNG for consistency with other platforms
    let image = image::load_from_memory_with_format(&data, image::ImageFormat::Bmp)
        .map_err(|e| anyhow::anyhow!("Failed to decode BMP: {}", e))?;
    let rgba_image = image.to_rgba8();

    Ok(rgba_image.to_vec())
}

/// Windows-specific: Write image to clipboard as Bitmap format
#[cfg(target_os = "windows")]
fn write_image_windows(bytes: &[u8]) -> Result<()> {
    use clipboard_win::{empty, formats, set_clipboard};
    use std::ptr::null_mut;
    use winapi::um::winuser::{CloseClipboard, GetOpenClipboardWindow, OpenClipboard};

    // Decode image bytes
    let img = image::load_from_memory(bytes)
        .map_err(|e| anyhow::anyhow!("Failed to decode image: {}", e))?;

    // Convert to BMP format with proper header
    let bmp_bytes = to_bitmap(&img);

    // Retry mechanism to open clipboard (max 5 attempts, 10ms delay)
    let mut retry_count = 0;
    while retry_count < 5 {
        unsafe {
            if OpenClipboard(null_mut()) != 0 {
                // Successfully opened clipboard
                break;
            }
            // If clipboard is opened by another process, wait and retry
            if GetOpenClipboardWindow() != null_mut() {
                std::thread::sleep(std::time::Duration::from_millis(10));
                retry_count += 1;
            } else {
                return Err(anyhow::anyhow!("Failed to open clipboard"));
            }
        }
    }

    if retry_count == 5 {
        return Err(anyhow::anyhow!(
            "Failed to open clipboard after multiple attempts"
        ));
    }

    // Clear clipboard and set new content
    let clear_result = empty();
    let set_result = set_clipboard(formats::Bitmap, &bmp_bytes);

    // Close clipboard
    unsafe {
        CloseClipboard();
    }

    clear_result.map_err(|e| anyhow::anyhow!("Failed to clear clipboard: {}", e))?;
    set_result.map_err(|e| anyhow::anyhow!("Failed to set clipboard: {}", e))?;

    Ok(())
}

/// Convert image to BMP format (Windows Bitmap)
/// Generates BMP file header + info header + pixel data
#[cfg(target_os = "windows")]
fn to_bitmap(img: &image::DynamicImage) -> Vec<u8> {
    use image::GenericImageView;

    // Flip image vertically because BMP scan lines are stored bottom to top
    let img = img.flipv();

    // Generate the 54-byte header
    let mut byte_vec = get_bmp_header(img.width(), img.height());

    // Add pixel data (BGRA format)
    for (_, _, pixel) in img.pixels() {
        let pixel_bytes = pixel.0;
        byte_vec.push(pixel_bytes[2]); // B
        byte_vec.push(pixel_bytes[1]); // G
        byte_vec.push(pixel_bytes[0]); // R
        byte_vec.push(pixel_bytes[3]); // A (unused in BMP spec but included)
    }

    byte_vec
}

/// Generate BMP file header and info header (54 bytes total)
#[cfg(target_os = "windows")]
fn get_bmp_header(width: u32, height: u32) -> Vec<u8> {
    let mut vec = vec![0; 54];

    // BM signature
    vec[0] = 66; // 'B'
    vec[1] = 77; // 'M'

    // File size
    let file_size = width * height * 4 + 54;
    set_bytes(&mut vec, &file_size.to_le_bytes(), 2..6);

    // Reserved (unused)
    set_bytes(&mut vec, &0_u32.to_le_bytes(), 6..10);

    // Offset to pixel data
    let offset = 54_u32;
    set_bytes(&mut vec, &offset.to_le_bytes(), 10..14);

    // Info header size
    let header_size = 40_u32;
    set_bytes(&mut vec, &header_size.to_le_bytes(), 14..18);

    // Width
    set_bytes(&mut vec, &width.to_le_bytes(), 18..22);

    // Height
    set_bytes(&mut vec, &height.to_le_bytes(), 22..26);

    // Planes (must be 1)
    let planes = 1_u16;
    set_bytes(&mut vec, &planes.to_le_bytes(), 26..28);

    // Bits per pixel (32 bits for BGRA)
    let bits_per_pixel = 32_u16;
    set_bytes(&mut vec, &bits_per_pixel.to_le_bytes(), 28..30);

    // Compression (0 = no compression)
    set_bytes(&mut vec, &0_u32.to_le_bytes(), 30..34);

    // Compressed size (0 when no compression)
    set_bytes(&mut vec, &0_u32.to_le_bytes(), 34..38);

    // Horizontal resolution (0 is allowed)
    set_bytes(&mut vec, &0_u32.to_le_bytes(), 38..42);

    // Vertical resolution (0 is allowed)
    set_bytes(&mut vec, &0_u32.to_le_bytes(), 42..46);

    // Colors used (0 is allowed)
    set_bytes(&mut vec, &0_u32.to_le_bytes(), 46..50);

    // Important colors (0 is allowed)
    set_bytes(&mut vec, &0_u32.to_le_bytes(), 50..54);

    vec
}

/// Helper to set bytes in a slice at a specific range
#[cfg(target_os = "windows")]
fn set_bytes(to: &mut [u8], from: &[u8], range: Range<usize>) {
    for (from_idx, i) in range.enumerate() {
        to[i] = from[from_idx];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[cfg(target_os = "windows")]
    async fn test_read_write_text() {
        let clipboard = WindowsClipboard::new().unwrap();

        let content = ClipboardContent {
            v: 1,
            ts_ms: chrono::Utc::now().timestamp_millis(),
            items: vec![ClipboardItem {
                mime: MimeType::text_plain(),
                data: ClipboardData::Text {
                    text: "Hello, Windows!".to_string(),
                },
                meta: Default::default(),
            }],
            meta: Default::default(),
        };

        clipboard.write(content.clone()).await.unwrap();

        let read = clipboard.read().await.unwrap();
        assert_eq!(read.items.len(), 1);
        assert_eq!(read.items[0].mime.0, "text/plain");
        if let ClipboardData::Text { text } = &read.items[0].data {
            assert_eq!(text, "Hello, Windows!");
        } else {
            panic!("Expected text data");
        }
    }
}
