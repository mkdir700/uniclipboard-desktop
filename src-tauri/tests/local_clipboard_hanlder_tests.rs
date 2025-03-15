use arboard::Clipboard;
use bytes::Bytes;
use chrono::Utc;
use clipboard_rs::{common::RustImage, RustImageData};
use image::ImageReader;
use serial_test::serial;
use std::fs;
use std::path::PathBuf;
use uniclipboard::{LocalClipboard, LocalClipboardTrait, Payload};

// #[tokio::test]
// #[cfg_attr(not(feature = "integration_tests"), ignore)]
// #[serial]
// async fn test_read_image_from_local_clipboard() {
//     use std::io::Write;
//     use std::fs::File;

//     let handler = LocalClipboard::new();

//     let payload = handler.read().await.unwrap();
//     // 查看 payload 的类型
//     match payload {
//         Payload::Image(image_payload) => {
//             println!("size: {} MB", image_payload.size as f32 / 1024.0 / 1024.0);
//             let mut file = File::create("test_image.png").unwrap();
//             file.write_all(&image_payload.content).unwrap();
//         }
//         _ => {
//             panic!("读取到的 payload 不是图像");
//         }
//     }
// }

#[tokio::test]
#[cfg_attr(not(feature = "integration_tests"), ignore)]
#[serial]
async fn test_write_image_to_local_clipboard() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 读取测试图片
    let image_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test_resources")
        .join("google.png");

    let img = ImageReader::open(&image_path)?.decode()?;
    let (width, height) = (img.width(), img.height());
    let image_bytes = fs::read(&image_path)?;

    let payload = Payload::new_image(
        Bytes::from(image_bytes.clone()),
        "test_device".to_string(),
        Utc::now(),
        width as usize,
        height as usize,
        "png".to_string(),
        image_bytes.len(),
    );
    let local_handler = LocalClipboard::new().unwrap();
    local_handler.write(payload.clone()).await?;

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    Ok(())
}

#[tokio::test]
#[cfg_attr(not(feature = "integration_tests"), ignore)]
#[serial]
async fn test_read_write_clipboard_text() {
    let test_text = "Hello, world!";
    let mut clipboard = Clipboard::new().unwrap();
    clipboard.set_text("random text").unwrap();

    let local_handler = LocalClipboard::new().unwrap();
    let payload = local_handler.read().await.unwrap();
    if let Payload::Text(text_payload) = payload {
        assert_ne!(text_payload.text(), test_text);
    } else {
        panic!("读取到的 payload 不是文本");
    }

    let write_payload = Payload::new_text(
        Bytes::from(test_text.to_string()),
        "test_device".to_string(),
        Utc::now(),
    );
    local_handler.write(write_payload).await.unwrap();
    let payload = local_handler.read().await.unwrap();
    if let Payload::Text(text_payload) = payload {
        assert_eq!(text_payload.text(), test_text);
    } else {
        panic!("读取到的 payload 不是文本");
    }
}

#[tokio::test]
#[cfg_attr(not(feature = "integration_tests"), ignore)]
#[serial]
async fn test_read_write_clipboard_image() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 读取测试图片
    let image_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test_resources")
        .join("2048.jpeg");

    let img = ImageReader::open(&image_path)?.decode()?;
    let (width, height) = (img.width(), img.height());
    let image_bytes = fs::read(&image_path)?;

    // 创建 Payload
    let payload = Payload::new_image(
        Bytes::from(image_bytes.clone()),
        "test_device".to_string(),
        Utc::now(),
        width as usize,
        height as usize,
        "jpeg".to_string(),
        image_bytes.len(),
    );
    let local_handler = LocalClipboard::new().unwrap();
    local_handler.write(payload).await?;
    Ok(())
}

// 尝试读取两张图片，先写入一张，再写入另一张，然后再读取
#[tokio::test]
#[cfg_attr(not(feature = "integration_tests"), ignore)]
#[serial]
async fn test_write_two_images_and_read() -> Result<(), Box<dyn std::error::Error>> {
    let local_handler = LocalClipboard::new().unwrap();
    // 1. 读取测试图片
    let image_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test_resources")
        .join("2048.jpeg");

    let img = ImageReader::open(&image_path)?.decode()?;
    let (width, height) = (img.width(), img.height());
    let image_bytes = fs::read(&image_path)?;

    // 创建 Payload
    let payload = Payload::new_image(
        Bytes::from(image_bytes.clone()),
        "test_device".to_string(),
        Utc::now(),
        width as usize,
        height as usize,
        "jpeg".to_string(),
        image_bytes.len(),
    );
    local_handler.write(payload).await?;

    let img2_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test_resources")
        .join("google.png");
    let image2 = match RustImageData::from_path(img2_path.to_str().unwrap()) {
        Ok(image) => image,
        Err(e) => {
            println!("Error: {}", e);
            panic!("Error: {}", e);
        }
    };
    let (width2, height2) = image2.get_size();
    let png_buffer = image2.to_png().unwrap();
    let png_bytes = png_buffer.get_bytes().to_vec();
    let payload2 = Payload::new_image(
        Bytes::from(png_bytes.clone()),
        "test_device".to_string(),
        Utc::now(),
        width2 as usize,
        height2 as usize,
        "png".to_string(),
        png_bytes.len(),
    );
    local_handler.write(payload2.clone()).await?;

    let read_payload = local_handler.read().await?;
    println!("read_payload: {}", read_payload);
    println!("payload2: {}", payload2);
    assert!(payload2.is_duplicate(&read_payload));
    Ok(())
}

// use image::{ImageBuffer, Rgba};

// #[test]
// #[cfg_attr(not(feature = "integration_tests"), ignore)]
// fn test_read_image_directly_from_arboard() {
//     let mut clipboard = Clipboard::new().expect("Failed to create clipboard");

//     // 尝试从剪贴板读取图片
//     match clipboard.get_image() {
//         Ok(image_data) => {
//             println!("Successfully read image from clipboard");
//             println!(
//                 "Image dimensions: {}x{}",
//                 image_data.width, image_data.height
//             );

//             // 将图片数据转换为 ImageBuffer
//             let img = ImageBuffer::from_raw(
//                 image_data.width as u32,
//                 image_data.height as u32,
//                 image_data.bytes.to_vec(),
//             )
//             .expect("Failed to create image buffer");

//             // 保存图片以便查看
//             img.save("test_arboard_image.png")
//                 .expect("Failed to save image");

//             // 检查图片是否有透明度
//             let has_transparency = img.pixels().any(|p: &Rgba<u8>| p[3] < 255);
//             println!("Image has transparency: {}", has_transparency);

//             assert!(true, "Image read successfully");
//         }
//         Err(e) => {
//             panic!("Failed to read image from clipboard: {:?}", e);
//         }
//     }
// }

#[cfg(windows)]
use clipboard_win::{formats, get_clipboard, Clipboard as WinClipboard};

#[test]
#[cfg(windows)]
#[cfg_attr(not(feature = "integration_tests"), ignore)]
fn test_read_image_using_clipboard_win() {
    // 打开剪贴板

    use image::Rgba;
    let _clipboard = WinClipboard::new().expect("Failed to open clipboard");

    // 尝试从剪贴板读取位图
    match get_clipboard(formats::Bitmap) {
        Ok(data) => {
            println!("Successfully read image from clipboard");

            // 直接从剪贴板数据创建图像
            let img =
                image::load_from_memory(&data).expect("Failed to load image from clipboard data");

            println!("Image dimensions: {}x{}", img.width(), img.height());

            // 保存图片以便查看
            img.save("test_clipboard_win_image.png")
                .expect("Failed to save image");

            // 检查图片是否有透明度
            let rgba_image = img.to_rgba8();
            let has_transparency = rgba_image.pixels().any(|p: &Rgba<u8>| p[3] < 255);
            println!("Image has transparency: {}", has_transparency);

            assert!(true, "Image read successfully");
        }
        Err(e) => {
            panic!("Failed to read image from clipboard: {:?}", e);
        }
    }
}
