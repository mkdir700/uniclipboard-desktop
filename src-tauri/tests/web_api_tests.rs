use reqwest_dav::re_exports::reqwest::{self, StatusCode};
use serial_test::serial;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
    time::Duration,
};

use uniclipboard::{
    device::Device,
    message::{DeviceSyncInfo, DevicesSyncMessage, RegisterDeviceMessage},
    web::{handlers::message_handler::MessageSource, WebServer},
    Config, WebSocketHandler, WebSocketMessageHandler, CONFIG,
};

#[cfg(test)]
#[serial]
mod tests {
    use uniclipboard::connection::ConnectionManager;
    use uniclipboard::db::DB_POOL;

    use super::*;
    use std::env;
    use std::fs;

    fn setup_test_env() {
        env::set_var("DATABASE_URL", "uniclipboard_tests.db");
    }

    #[ctor::ctor]
    fn setup() {
        setup_test_env();

        DB_POOL.run_migrations().unwrap();
    }

    // 这个函数会在模块中的所有测试运行之后执行
    #[ctor::dtor]
    fn teardown() {
        // 删除测试数据库文件
        if let Err(e) = fs::remove_file("uniclipboard_tests.db") {
            println!("清理测试数据库时出错: {}", e);
        }
    }

    struct WebServerWrapper {
        websocket_message_handler: Arc<WebSocketMessageHandler>,
        #[allow(unused)]
        websocket_handler: Arc<WebSocketHandler>,
        webserver: Arc<WebServer>,
    }

    fn setup_config() -> Config {
        let mut config = CONFIG.write().unwrap();
        config.webserver_addr = Some("127.0.0.1".to_string());
        config.webserver_port = Some(8333);
        config.clone()
    }

    async fn setup_webserver() -> WebServerWrapper {
        let config = setup_config();
        let connection_manager = Arc::new(ConnectionManager::new());
        let websocket_message_handler =
            Arc::new(WebSocketMessageHandler::new(connection_manager.clone()));
        let websocket_handler = Arc::new(WebSocketHandler::new(
            websocket_message_handler.clone(),
            connection_manager.clone(),
        ));
        let webserver = WebServer::new(
            SocketAddr::new(
                config.webserver_addr.unwrap().parse().unwrap(),
                config.webserver_port.unwrap(),
            ),
            websocket_handler.clone(),
        );

        WebServerWrapper {
            websocket_message_handler,
            websocket_handler,
            webserver: Arc::new(webserver),
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_404() {
        let w = setup_webserver().await;
        let webserver_clone = Arc::clone(&w.webserver);
        tokio::spawn(async move { webserver_clone.run().await });
        tokio::time::sleep(Duration::from_millis(500)).await;

        let response = reqwest::get("http://127.0.0.1:8333/api/device111")
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_device_list() {
        let w = setup_webserver().await;
        let webserver_clone = Arc::clone(&w.webserver);
        tokio::spawn(async move { webserver_clone.run().await });
        tokio::time::sleep(Duration::from_millis(500)).await;

        // 第一次运行是空列表
        let response = reqwest::get("http://127.0.0.1:8333/api/device")
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.text().await.unwrap(),
            "{\"code\":200,\"message\":\"成功\",\"data\":{\"list\":[],\"total\":0}}"
        );

        // 添加一个设备
        let device = Device::new(
            "device1".to_string(),
            Some("127.0.0.1".to_string()),
            Some(8114),
            Some(8333),
        );
        // let data = DeviceListData {
        //     devices: vec![device],
        //     replay_device_ids: vec![],
        // };
        w.websocket_message_handler
            .message_handler
            .handle_register(
                RegisterDeviceMessage::new(
                    "device1".to_string(),
                    Some("127.0.0.1".to_string()),
                    Some(8114),
                ),
                SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8114),
            )
            .await;

        w.websocket_message_handler
            .message_handler
            .handle_device_list_sync(
                DevicesSyncMessage {
                    devices: vec![DeviceSyncInfo::from(&device)],
                    replay_device_ids: vec![],
                },
                MessageSource::IpPort(SocketAddr::new(
                    IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                    8114,
                )),
            )
            .await;

        // 再次运行
        let response = reqwest::get("http://127.0.0.1:8333/api/device")
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
        response.text().await.unwrap(),
        "{\"code\":200,\"message\":\"成功\",\"data\":{\"list\":[{\"id\":\"device1\",\"ip\":\"127.0.0.1\",\"port\":8333,\"server_port\":8333,\"status\":\"Unknown\",\"self_device\":false,\"updated_at\":0}],\"total\":1}}"
    );
    }
}
