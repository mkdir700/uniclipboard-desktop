use anyhow::Result;
use bytes::Bytes;
use chrono::Utc;
use serial_test::serial;
use std::env;
use std::fs;
use std::time::Duration;
use uniclipboard::{
    config::CONFIG,
    uni_clipboard::{UniClipboard, UniClipboardBuilder},
};
use uniclipboard::{Config, Payload};

mod tests {
    use uniclipboard::context::AppContextBuilder;

    use super::*;

    fn setup_test_env() {
        env::set_var("DATABASE_URL", "uniclipboard_tests.db");
    }

    fn setup_config() -> Config {
        let mut config = CONFIG.write().unwrap();
        config.webserver_addr = Some("127.0.0.1".to_string());
        config.webserver_port = Some(8333);
        config.peer_device_addr = Some("127.0.0.1".to_string());
        config.peer_device_port = Some(8333);
        config.clone()
    }

    #[ctor::ctor]
    fn setup() {
        setup_test_env();
        setup_config();
    }

    // 这个函数会在模块中的所有测试运行之后执行
    #[ctor::dtor]
    fn teardown() {
        // 删除测试数据库文件
        if let Err(e) = fs::remove_file("uniclipboard_tests.db") {
            println!("清理测试数据库时出错: {}", e);
        }
    }

    // 辅助函数：创建测试用的 UniClipboard 实例
    async fn create_test_uni_clipboard(config: Config) -> Result<UniClipboard> {
        let app_context = AppContextBuilder::new(config).build().await?;
        let remote_sync_manager = app_context.remote_sync_manager;
        let webserver = app_context.webserver;
        let local_clipboard = app_context.local_clipboard;
        let connection_manager = app_context.connection_manager;

        UniClipboardBuilder::new()
            .set_webserver(webserver)
            .set_local_clipboard(local_clipboard)
            .set_remote_sync(remote_sync_manager)
            .set_connection_manager(connection_manager)
            .build()
    }

    #[tokio::test]
    #[cfg_attr(not(feature = "integration_tests"), ignore)]
    #[serial]
    async fn test_uni_clipboard_start_stop() -> Result<()> {
        let config = setup_config();
        let uni_clipboard = create_test_uni_clipboard(config).await?;

        assert!(
            uni_clipboard.start().await.is_ok(),
            "启动 UniClipboard 失败"
        );
        assert!(
            uni_clipboard.start().await.is_err(),
            "重复启动 UniClipboard 应该失败"
        );
        assert!(uni_clipboard.stop().await.is_ok(), "停止 UniClipboard 失败");
        assert!(
            uni_clipboard.stop().await.is_err(),
            "重复停止 UniClipboard 应该失败"
        );

        Ok(())
    }

    #[tokio::test]
    #[cfg_attr(not(feature = "integration_tests"), ignore)]
    #[serial]
    async fn test_uni_clipboard_pause_resume() -> Result<()> {
        let config = setup_config();
        let uni_clipboard = create_test_uni_clipboard(config).await?;

        assert!(
            uni_clipboard.start().await.is_ok(),
            "启动 UniClipboard 失败"
        );
        assert!(
            uni_clipboard.pause().await.is_ok(),
            "暂停 UniClipboard 失败"
        );
        assert!(
            uni_clipboard.resume().await.is_ok(),
            "恢复 UniClipboard 失败"
        );
        assert!(uni_clipboard.stop().await.is_ok(), "停止 UniClipboard 失败");

        Ok(())
    }

    #[tokio::test]
    #[cfg_attr(not(feature = "integration_tests"), ignore)]
    #[serial]
    async fn test_uni_clipboard_client_server_sync() -> Result<()> {
        let mut config = setup_config();
        config.webserver_port = Some(8333);
        // 创建服务器和客户端实例
        let server = create_test_uni_clipboard(config.clone()).await?;
        config.webserver_port = Some(8334);
        let client = create_test_uni_clipboard(config).await?;

        // 启动服务器和客户端
        server.start().await?;
        // 等待连接建立
        tokio::time::sleep(Duration::from_secs(1)).await;
        client.start().await?;

        let client_clipboard = client.get_clipboard();
        let test_payload =
            Payload::new_text(Bytes::from("test"), "device_id".to_string(), Utc::now());
        // 从客户端写入一条消息，然后从服务器读取
        client_clipboard.write(test_payload.clone()).await?;

        let content = server.get_clipboard().read().await?;
        assert_eq!(content, test_payload, "客户端到服务器的同步失败");

        // 停止服务器和客户端
        server.stop().await?;
        client.stop().await?;

        Ok(())
    }

    #[tokio::test]
    #[cfg_attr(not(feature = "integration_tests"), ignore)]
    #[serial]
    async fn test_uni_clipboard_server_to_client_sync() -> Result<()> {
        let mut config = setup_config();
        config.webserver_port = Some(8333);
        // 创建服务器和客户端实例
        let server = create_test_uni_clipboard(config.clone()).await?;
        config.webserver_port = Some(8334);
        let client = create_test_uni_clipboard(config).await?;

        // 启动服务器和客户端
        server.start().await?;
        // 等待连接建立
        tokio::time::sleep(Duration::from_secs(1)).await;
        client.start().await?;

        let test_payload =
            Payload::new_text(Bytes::from("test"), "device_id".to_string(), Utc::now());
        // 从服务器写入一条消息，然后从客户端读取
        server.get_clipboard().write(test_payload.clone()).await?;

        let content = client.get_clipboard().read().await?;
        assert_eq!(content, test_payload, "服务器到客户端的同步失败");

        // 停止服务器和客户端
        server.stop().await?;
        client.stop().await?;

        Ok(())
    }

    #[tokio::test]
    #[cfg_attr(not(feature = "integration_tests"), ignore)]
    #[serial]
    async fn test_uni_clipboard_duplicate_content_handling() -> Result<()> {
        let config = setup_config();
        let uni_clipboard = create_test_uni_clipboard(config).await?;
        uni_clipboard.start().await?;

        let clipboard = uni_clipboard.get_clipboard();
        let test_payload = Payload::new_text(
            Bytes::from("Duplicate content"),
            "device_id".to_string(),
            Utc::now(),
        );

        // 写入内容
        clipboard.write(test_payload.clone()).await?;

        // 等待同步
        tokio::time::sleep(Duration::from_secs(1)).await;

        // 再次写入相同的内容
        clipboard.write(test_payload.clone()).await?;

        // 等待同步
        tokio::time::sleep(Duration::from_secs(1)).await;

        // 读取内容，应该与最初写入的内容相同
        let content = clipboard.read().await?;
        assert_eq!(content, test_payload, "重复内容处理失败");

        uni_clipboard.stop().await?;

        Ok(())
    }
}
