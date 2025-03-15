use std::time::Duration;

use bytes::Bytes;
use chrono::Utc;
use serial_test::serial;
use tokio::time::{sleep, timeout};
use uniclipboard::{LocalClipboard, Payload, LocalClipboardTrait};

#[tokio::test]
#[cfg_attr(not(feature = "integration_tests"), ignore)]
#[serial]
async fn test_local_clipboard_write_and_read() {
    let handler = LocalClipboard::new().unwrap();

    // 准备测试数据
    let test_content = "测试剪贴板内容";
    let payload = Payload::new_text(Bytes::from(test_content), "local".to_string(), Utc::now());
    // 测试写入
    assert!(handler.write(payload.clone()).await.is_ok());

    // 测试读取
    let read_result = handler.read().await;
    assert!(read_result.is_ok());

    let read_payload = read_result.unwrap();
    assert_eq!(read_payload.get_content(), payload.get_content());
    // FIXEME: 因为没有读取全局配置，所以 device_id 拿到的是空字符串
    // assert_eq!(read_payload.get_device_id(), payload.get_device_id());
}

#[tokio::test]
#[cfg_attr(not(feature = "integration_tests"), ignore)]
#[serial]
async fn test_local_clipboard_pull() {
    let handler = LocalClipboard::new().unwrap();

    // 准备初始测试数据
    let initial_content = "123";
    let initial_payload = Payload::new_text(
        Bytes::from(initial_content),
        "local".to_string(),
        Utc::now(),
    );
    handler.write(initial_payload).await.unwrap();

    // 在另一个任务中更改剪贴板内容
    tokio::spawn(async move {
        let handler_clone = LocalClipboard::new().unwrap();
        let new_content = "456";
        let new_payload =
            Payload::new_text(Bytes::from(new_content), "local".to_string(), Utc::now());
        handler_clone.write(new_payload).await.unwrap();
        println!("new_content: {}", new_content);
    });

    // 等待 100ms 后，确保新的内容已经写入
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // 测试 read 函数
    let pulled_payload = handler
        .read()
        .await
        .unwrap();

    assert_eq!(*pulled_payload.get_content(), Bytes::from("456"));
    // assert_eq!(pulled_payload.get_device_id(), "local");
}

#[tokio::test]
#[cfg_attr(not(feature = "integration_tests"), ignore)]
#[serial]
async fn test_start_monitoring() {
    // 创建 LocalClipboard 实例
    let local_clipboard = LocalClipboard::new().unwrap();

    // 启动监控
    let mut receiver = local_clipboard.start_monitoring().await.expect("Failed to start monitoring");

    // 等待一小段时间，确保监控已经启动
    sleep(Duration::from_millis(100)).await;

    // 测试场景1：自写操作应该被跳过
    let test_content1 = "self write";
    let payload1 = Payload::new_text(Bytes::from(test_content1), "test_device".to_string(), Utc::now());

    // 使用 write 方法写入内容（这应该触发自写标记）
    local_clipboard.set_clipboard_content(payload1.clone()).await.expect("Failed to write content");

    // 尝试接收变化通知，应该超时
    match timeout(Duration::from_millis(1000), receiver.recv()).await {
        Ok(Some(data)) => {
            println!("Received unexpected change notification for self-write: {}", data);
            panic!("Received unexpected change notification for self-write")
        },
        Ok(None) => panic!("Channel closed unexpectedly"),
        Err(_) => println!("Correctly timed out waiting for self-write change"),
    }

    // 等待冷却时间过去
    sleep(Duration::from_millis(600)).await;

    // 测试场景2：非自写操作应该被检测到
    let test_content2 = "非自写测试内容";
    let payload2 = Payload::new_text(Bytes::from(test_content2), "test_device".to_string(), Utc::now());

    // 模拟外部剪贴板变化
    local_clipboard.write(payload2.clone()).await.expect("Failed to set clipboard content");

    // 尝试接收变化通知
    match timeout(Duration::from_secs(1), receiver.recv()).await {
        Ok(Some(received_payload)) => {
            assert_eq!(received_payload.get_content(), payload2.get_content());
            println!("Correctly received change notification for non-self-write");
        },
        Ok(None) => panic!("Channel closed unexpectedly"),
        Err(_) => panic!("Timeout waiting for clipboard change"),
    }

    // 停止监控
    local_clipboard.stop_monitoring().await.expect("Failed to stop monitoring");
}


#[tokio::test]
#[cfg_attr(not(feature = "integration_tests"), ignore)]
#[serial]
async fn test_pause_and_resume() {
    // 创建 LocalClipboard 实例
    let local_clipboard = LocalClipboard::new().unwrap();

    // 启动监控
    let mut receiver = local_clipboard.start_monitoring().await.expect("Failed to start monitoring");

    // 等待一小段时间，确保监控已经启动
    sleep(Duration::from_millis(100)).await;

    // 准备测试数据
    let test_content1 = "测试内容1";
    let payload1 = Payload::new_text(Bytes::from(test_content1), "test_device".to_string(), Utc::now());

    // 设置剪贴板内容
    local_clipboard.write(payload1.clone()).await.expect("Failed to set clipboard content");

    // 等待并接收第一次变化
    let received_payload1 = tokio::time::timeout(Duration::from_secs(5), receiver.recv()).await
        .expect("Timeout waiting for clipboard change")
        .expect("Channel closed unexpectedly");
    assert_eq!(received_payload1.get_content(), payload1.get_content());

    // 暂停监控
    local_clipboard.pause().await;

    // 准备新的测试数据
    let test_content2 = "test content 2";
    let payload2 = Payload::new_text(Bytes::from(test_content2), "test_device".to_string(), Utc::now());

    // 在暂停状态下设置剪贴板内容
    local_clipboard.write(payload2.clone()).await.expect("Failed to set clipboard content");

    // 尝试接收变化，应该超时
    let timeout_result = tokio::time::timeout(Duration::from_secs(1), receiver.recv()).await;
    // 打印接收的内容
    assert!(timeout_result.is_err(), "Received unexpected clipboard change while paused");

    // 恢复监控
    local_clipboard.resume().await;

    // 准备新的测试数据
    let test_content3 = "测试内容3";
    let payload3 = Payload::new_text(Bytes::from(test_content3), "test_device".to_string(), Utc::now());

    // 在恢复状态下设置剪贴板内容
    local_clipboard.write(payload3.clone()).await.expect("Failed to set clipboard content");

    // 等待并接收恢复后的变化
    let received_payload3 = tokio::time::timeout(Duration::from_secs(5), receiver.recv()).await
        .expect("Timeout waiting for clipboard change after resume")
        .expect("Channel closed unexpectedly");
    assert_eq!(received_payload3.get_content(), payload3.get_content());

    // 停止监控
    local_clipboard.stop_monitoring().await.expect("Failed to stop monitoring");
}