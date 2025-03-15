use anyhow::Result;
use bytes::Bytes;
use chrono::Utc;
use dotenv::dotenv;
use std::env;
use uniclipboard::{message::Payload, network::WebDAVClient};

fn load_env() {
    dotenv().ok();
}

async fn create_webdav_client() -> Result<WebDAVClient> {
    load_env();
    let webdav_url = env::var("WEBDAV_URL").expect("WEBDAV_URL not set");
    let username = env::var("WEBDAV_USERNAME").expect("WEBDAV_USERNAME not set");
    let password = env::var("WEBDAV_PASSWORD").expect("WEBDAV_PASSWORD not set");

    WebDAVClient::new(webdav_url, username, password).await
}

#[tokio::test]
#[cfg_attr(not(feature = "network_tests"), ignore)]
async fn test_webdav_client_upload_and_download_text() -> Result<()> {
    let client = create_webdav_client().await?;

    let test_content = "测试内容".as_bytes().to_vec();
    let payload = Payload::new_text(
        Bytes::from(test_content.clone()),
        "test_device".to_string(),
        Utc::now(),
    );

    // 上传文件
    let upload_path = "/test_upload.txt".to_string();
    let filepath = client.upload(upload_path.clone(), payload).await?;

    // 下载文件
    let downloaded_payload = client.download(filepath.clone()).await?;

    // 验证下载的内容
    if let Payload::Text(text_payload) = downloaded_payload {
        assert_eq!(text_payload.device_id, "test_device");
        assert_eq!(text_payload.content, Bytes::from(test_content));
    } else {
        panic!("Expected TextPayload");
    }

    // 删除文件
    client.delete(filepath.clone()).await?;
    Ok(())
}

#[tokio::test]
#[cfg_attr(not(feature = "network_tests"), ignore)]
async fn test_webdav_client_upload_and_download_image() -> Result<()> {
    let client = create_webdav_client().await?;

    let test_content = vec![0u8; 100]; // 模拟图片数据
    let payload = Payload::new_image(
        Bytes::from(test_content.clone()),
        "test_device".to_string(),
        Utc::now(),
        100,
        100,
        "png".to_string(),
        100,
    );

    // 上传文件
    let upload_path = "/test_upload_image.png".to_string();
    let filepath = client.upload(upload_path.clone(), payload).await?;

    // 下载文件
    let downloaded_payload = client.download(filepath.clone()).await?;

    // 验证下载的内容
    if let Payload::Image(image_payload) = downloaded_payload {
        assert_eq!(image_payload.device_id, "test_device");
        assert_eq!(image_payload.content, Bytes::from(test_content));
        assert_eq!(image_payload.width, 100);
        assert_eq!(image_payload.height, 100);
        assert_eq!(image_payload.format, "png");
        assert_eq!(image_payload.size, 100);
    } else {
        panic!("Expected ImagePayload");
    }

    // 删除文件
    client.delete(filepath.clone()).await?;
    Ok(())
}

#[tokio::test]
#[cfg_attr(not(feature = "network_tests"), ignore)]
async fn test_webdav_client_get_latest_added_file() -> Result<()> {
    let client = create_webdav_client().await?;

    // 上传两个文件
    let payload1 = Payload::new_text(
        Bytes::from("文件1"),
        "test_device".to_string(),
        Utc::now(),
    );
    let file1_path = client
        .upload("/dir1".to_string(), payload1)
        .await?;
    
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await; // 确保时间戳不同

    let payload2 = Payload::new_text(
        Bytes::from("文件2"),
        "test_device".to_string(),
        Utc::now(),
    );
    let file2_path: String = client
        .upload("/dir1".to_string(), payload2)
        .await?;

    // 获取最新添加的文件
    let latest_file_meta = client.fetch_latest_file_meta("/dir1".to_string()).await?;

    assert_eq!(latest_file_meta.get_path(), file2_path);

    // 清理测试文件
    client.delete(file1_path).await?;
    client.delete(file2_path).await?;

    Ok(())
}
