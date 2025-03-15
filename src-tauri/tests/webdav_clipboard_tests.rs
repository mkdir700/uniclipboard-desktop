// use anyhow::Result;
// use bytes::Bytes;
// use chrono::Utc;
// use dotenv::dotenv;
// use image::ImageReader;
// use serial_test::serial;
// use std::env;
// use std::fs;
// use std::path::PathBuf;
// use std::sync::Once;
// use std::time::Duration;
// use uniclipboard::remote_sync::traits::RemoteClipboardSync;
// use uniclipboard::remote_sync::webdav_handler::WebDavSync;
// use uniclipboard::{
//     config::{Config, CONFIG},
//     message::Payload,
//     network::WebDAVClient,
//     LocalClipboard,
//     LocalClipboardTrait,
// };

// static INIT: Once = Once::new();

// fn setup() {
//     INIT.call_once(|| {
//         dotenv().ok();
//         let mut test_config = Config::default();
//         test_config.device_id = "test-device".to_string();
//         test_config.webdav_url = Some(env::var("WEBDAV_URL").expect("WEBDAV_URL not set"));
//         test_config.username = Some(env::var("WEBDAV_USERNAME").expect("WEBDAV_USERNAME not set"));
//         test_config.password = Some(env::var("WEBDAV_PASSWORD").expect("WEBDAV_PASSWORD not set"));
//         *CONFIG.write().unwrap() = test_config;
//     });
// }

// async fn create_webdav_client() -> Result<WebDAVClient> {
//     setup();
//     let config = CONFIG.read().unwrap();
//     WebDAVClient::new(
//         config.webdav_url.as_ref().unwrap().clone(),
//         config.username.as_ref().unwrap().clone(),
//         config.password.as_ref().unwrap().clone(),
//     )
//     .await
// }

// #[tokio::test]
// #[cfg_attr(not(feature = "integration_tests"), ignore)]
// #[serial]
// async fn test_cloud_clipboard_push_text() {
//     setup();
//     let client = create_webdav_client().await.unwrap();
//     let handler = WebDavSync::new(client);
//     let payload = Payload::new_text(
//         Bytes::from("test content"),
//         "test_device".to_string(),
//         Utc::now(),
//     );

//     let result = handler.push_and_return_path(payload).await;
//     assert!(result.is_ok());
//     let path = result.unwrap();
//     let client_clone = handler.get_client();
//     let meta = client_clone
//         .fetch_latest_file_meta(handler.base_path.clone())
//         .await
//         .unwrap();
//     assert_eq!(meta.get_path(), path.to_string());

//     client_clone.delete(path).await.unwrap();
// }

// #[tokio::test]
// #[cfg_attr(not(feature = "integration_tests"), ignore)]
// #[serial]
// async fn test_cloud_clipboard_push_image() {
//     setup();
//     let client = create_webdav_client().await.unwrap();
//     let handler = WebDavSync::new(client);
//     let payload = Payload::new_image(
//         Bytes::from(vec![0u8; 100]), // 模拟图片数据
//         "test_device".to_string(),
//         Utc::now(),
//         100,
//         100,
//         "png".to_string(),
//         100,
//     );

//     let result = handler.push_and_return_path(payload).await;
//     assert!(result.is_ok());
//     let path = result.unwrap();
//     let client_clone = handler.get_client();
//     let meta = client_clone
//         .fetch_latest_file_meta(handler.base_path.clone())
//         .await
//         .unwrap();
//     assert_eq!(meta.get_path(), path);

//     client_clone.delete(path).await.unwrap();
// }

// #[tokio::test]
// #[cfg_attr(not(feature = "integration_tests"), ignore)]
// #[serial]
// async fn test_cloud_clipboard_pull_text() {
//     setup();
//     let client = create_webdav_client().await.unwrap();
//     let handler = WebDavSync::new(client);

//     // push a text file to the cloud clipboard
//     let payload = Payload::new_text(
//         Bytes::from("test content"),
//         "device-1".to_string(),
//         Utc::now(),
//     );

//     let result = handler.push(payload).await;
//     assert!(result.is_ok());
//     let _path = result.unwrap();

//     // pull the file from the cloud clipboard
//     let result = handler.pull(Some(Duration::from_secs(1))).await;
//     if let Err(e) = &result {
//         println!("Pull error: {:?}", e);
//     }
//     assert!(result.is_ok());
//     let pulled_payload = result.unwrap();
//     assert!(matches!(pulled_payload, Payload::Text(_)));
//     if let Payload::Text(text_payload) = pulled_payload {
//         assert_eq!(*text_payload.content, Bytes::from("test content"));
//     }
// }

// #[tokio::test]
// #[cfg_attr(not(feature = "integration_tests"), ignore)]
// #[serial]
// async fn test_cloud_clipboard_pull_image() {
//     setup();
//     let client = create_webdav_client().await.unwrap();
//     let handler = WebDavSync::new(client);

//     // push an image file to the cloud clipboard
//     let image_data = vec![0u8; 100]; // 模拟图片数据
//     let payload = Payload::new_image(
//         Bytes::from(image_data.clone()),
//         "device-1".to_string(),
//         Utc::now(),
//         100,
//         100,
//         "png".to_string(),
//         100,
//     );

//     let result = handler.push(payload).await;
//     assert!(result.is_ok());
//     let _path = result.unwrap();

//     // pull the file from the cloud clipboard
//     let result = handler.pull(Some(Duration::from_secs(1))).await;
//     if let Err(e) = &result {
//         println!("Pull error: {:?}", e);
//     }
//     assert!(result.is_ok());
//     let pulled_payload = result.unwrap();
//     assert!(matches!(pulled_payload, Payload::Image(_)));
//     if let Payload::Image(image_payload) = pulled_payload {
//         assert_eq!(*image_payload.content, Bytes::from(image_data));
//         assert_eq!(image_payload.width, 100);
//         assert_eq!(image_payload.height, 100);
//         assert_eq!(image_payload.format, "png");
//         assert_eq!(image_payload.size, 100);
//     }
// }

// #[tokio::test]
// #[cfg_attr(not(feature = "integration_tests"), ignore)]
// #[serial]
// async fn test_push_and_pull_image() -> Result<(), Box<dyn std::error::Error>> {
//     setup();
//     // 1. 读取测试图片
//     let image_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
//         .join("test_resources")
//         .join("2048.jpeg");

//     let img = ImageReader::open(&image_path)?.decode()?;
//     let (width, height) = (img.width(), img.height());
//     let image_bytes = fs::read(&image_path)?;

//     // 创建 Payload
//     let original_payload = Payload::new_image(
//         Bytes::from(image_bytes.clone()),
//         "test_device".to_string(),
//         Utc::now(),
//         width as usize,
//         height as usize,
//         "jpeg".to_string(),
//         image_bytes.len(),
//     );

//     // 创建 CloudClipboardHandler
//     let client = WebDAVClient::new(
//         CONFIG.read().unwrap().webdav_url.as_ref().unwrap().clone(),
//         CONFIG.read().unwrap().username.as_ref().unwrap().clone(),
//         CONFIG.read().unwrap().password.as_ref().unwrap().clone(),
//     )
//     .await?;
//     let cloud_handler = WebDavSync::new(client);
//     let local_handler = LocalClipboard::new();

//     // 2. 将图片 push 到云端
//     let path = cloud_handler
//         .push_and_return_path(original_payload.clone())
//         .await?;
//     println!("上传后的文件路径: {}", path);

//     // 3. 从云端 pull 到本地
//     let pulled_payload = cloud_handler.pull(Some(Duration::from_secs(5))).await?;
//     local_handler.write(pulled_payload.clone()).await?;

//     // 4. 对比两份数据是否一致
//     match (original_payload, pulled_payload.clone()) {
//         (Payload::Image(original), Payload::Image(pulled)) => {
//             assert_eq!(original.width, pulled.width);
//             assert_eq!(original.height, pulled.height);
//             assert_eq!(original.format, pulled.format);
//             assert_eq!(original.size, pulled.size);
//             assert_eq!(original.content, pulled.content);
//             println!("图片数据一致性验证通过");
//             println!("拉取的图片大小: {} bytes", pulled.content.len());

//             // 解码 base64 并保存图片以进行视觉验证
//             // fs::write("pulled_image.jpeg", &pulled.content)?;
//             // println!("拉取的图片已保存为 'pulled_image.jpeg'");
//             // local_handler.write(pulled_payload.clone())?;
//         }
//         _ => panic!("Payload 类型不匹配"),
//     }

//     Ok(())
// }
