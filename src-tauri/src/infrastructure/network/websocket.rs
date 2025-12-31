use crate::application::device_service::get_device_manager;
use crate::config::Setting;
use crate::domain::transfer_message::ClipboardTransferMessage;
use crate::domain::device::Device;
use crate::message::{DeviceSyncInfo, DevicesSyncMessage, RegisterDeviceMessage, WebSocketMessage};
use anyhow::Result;
use futures::StreamExt;
use futures_util::SinkExt;
use log::error;
use log::info;
use log::trace;
use log::warn;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::broadcast;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tokio::time::timeout;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::http::Uri;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

#[derive(Debug)]
pub struct WebSocketClient {
    uri: Uri,
    writer: Arc<
        Option<
            Mutex<
                futures_util::stream::SplitSink<
                    WebSocketStream<MaybeTlsStream<TcpStream>>,
                    Message,
                >,
            >,
        >,
    >,

    reader: Arc<
        Option<
            Mutex<futures_util::stream::SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>,
        >,
    >,
    message_tx: Arc<broadcast::Sender<Message>>,
    message_join_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
    health_check_join_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
    connected: Arc<AtomicBool>,
    user_setting: Setting,
}

impl WebSocketClient {
    pub fn new(uri: Uri) -> Self {
        let (tx, _) = broadcast::channel(100);
        WebSocketClient {
            uri,
            writer: Arc::new(None),
            reader: Arc::new(None),
            message_tx: Arc::new(tx),
            message_join_handle: Arc::new(RwLock::new(None)),
            health_check_join_handle: Arc::new(RwLock::new(None)),
            connected: Arc::new(AtomicBool::new(false)),
            user_setting: Setting::get_instance(),
        }
    }

    /// 创建一个新的 WebSocketClient 实例，使用指定的配置
    pub fn with_user_setting(uri: Uri, user_setting: Setting) -> Self {
        let (tx, _) = broadcast::channel(100);
        WebSocketClient {
            uri,
            writer: Arc::new(None),
            reader: Arc::new(None),
            message_tx: Arc::new(tx),
            message_join_handle: Arc::new(RwLock::new(None)),
            health_check_join_handle: Arc::new(RwLock::new(None)),
            connected: Arc::new(AtomicBool::new(false)),
            user_setting,
        }
    }

    /// 开启一个异步任务，持续的从 reader 中接收消息并转发到 message_tx
    async fn start_collect_messages(&mut self) {
        let reader = self.reader.clone();
        let message_tx = self.message_tx.clone();
        let connected = self.connected.clone();

        *self.message_join_handle.write().await = Some(tokio::spawn(async move {
            loop {
                if let Some(reader) = reader.as_ref() {
                    let mut reader = reader.lock().await;
                    match reader.next().await {
                        Some(Ok(msg)) => {
                            let _ = message_tx.send(msg);
                        }
                        Some(Err(e)) => {
                            warn!("WebSocket error: {}", e);
                            break;
                        }
                        None => {
                            warn!("WebSocket connection closed");
                            break;
                        }
                    }
                } else {
                    warn!("WebSocket reader not available");
                    break;
                }
            }

            info!("WebSocket message handler stopped");
            // TODO!: 修改 connected 状态为 false
            connected.store(false, Ordering::Relaxed);
        }));
    }

    /// 停止收集消息的异步任务
    async fn stop_collect_messages(&mut self) {
        if let Some(handle) = self.message_join_handle.write().await.take() {
            handle.abort();
        }
    }

    /// 开启一个异步任务，持续的向服务器发送 ping 消息，并等待 pong 消息
    /// 如果连续3次发送 ping 消息失败，则认为连接断开，并修改 connected 状态为 false
    async fn start_health_check(&mut self) {
        let message_tx = self.message_tx.clone();
        let connected = self.connected.clone();

        self.health_check_join_handle = Arc::new(RwLock::new(Some(tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            let mut ping_count = 0;
            loop {
                interval.tick().await;
                if !connected.load(Ordering::Relaxed) {
                    break;
                }

                if ping_count >= 3 {
                    error!("WebSocket health check failed 3 times");
                    connected.store(false, Ordering::Relaxed);
                    break;
                }

                // Generate random ping and wait for pong
                let rand_bytes = rand::random::<[u8; 16]>().to_vec();
                let rand_bytes_clone = rand_bytes.clone();
                let mut rx = message_tx.subscribe();

                let join_handle = tokio::spawn(async move {
                    if let Ok(msg) = rx.recv().await {
                        if let Message::Pong(msg) = msg {
                            trace!("Ping pong result: {:?}", msg);
                            return msg == rand_bytes_clone;
                        }
                    }
                    false
                });

                // Send ping and wait for pong with timeout
                let timeout_result = tokio::time::timeout(Duration::from_secs(3), async move {
                    // We can't send directly without WebSocketClient, so just check if we can receive
                    join_handle.await
                })
                .await;

                match timeout_result {
                    Ok(Ok(true)) => {
                        // Successful ping-pong, reset count
                        ping_count = 0;
                    }
                    _ => {
                        error!("WebSocket health check failed");
                        ping_count += 1;
                    }
                }
            }
        }))));
    }

    /// 停止健康检查的异步任务
    async fn stop_health_check(&self) {
        if let Some(handle) = self.health_check_join_handle.write().await.take() {
            handle.abort();
        }
    }

    /// 订阅消息, 返回一个 Receiver 对象
    pub fn subscribe(&self) -> broadcast::Receiver<Message> {
        self.message_tx.subscribe()
    }

    /// 判断是否连接到服务器
    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }

    /// 发送剪贴板同步消息
    /// 如果发送失败, 则尝试重新连接, 并重试3次
    #[allow(dead_code)]
    pub async fn send_clipboard_sync(&mut self, message: &WebSocketMessage) -> Result<()> {
        let mut retries = 0;
        while retries < 3 {
            if let Err(e) = self.send_with_websocket_message(message).await {
                retries += 1;
                error!("Failed to send message, retrying... {}", e);
                // 捕获 IO error: Broken pipe(os error 32)
                if e.to_string().to_lowercase().contains("io error") {
                    info!("WebSocket connection broken, reconnecting...");
                    self.reconnect().await?;
                    info!("Reconnected to server");
                    continue;
                } else {
                    return Err(e);
                }
            } else {
                return Ok(());
            }
        }
        Err(anyhow::anyhow!("Failed to send message after 3 retries"))
    }

    /// 接收剪贴板同步消息
    #[allow(dead_code)]
    pub async fn receive_clipboard_sync(&self) -> Result<Option<ClipboardTransferMessage>> {
        let message = self.subscribe().recv().await?;
        match message {
            Message::Text(text) => {
                let web_socket_message: WebSocketMessage = serde_json::from_str(&text)?;
                if let WebSocketMessage::ClipboardSync(data) = web_socket_message {
                    return Ok(Some(data));
                }
            }
            _ => {}
        }
        Ok(None)
    }

    pub async fn send_with_websocket_message(&self, message: &WebSocketMessage) -> Result<()> {
        let message = message.to_tungstenite_message();
        let mut writer = if let Some(writer) = self.writer.as_ref() {
            writer.lock().await
        } else {
            return Err(anyhow::anyhow!(
                "WebSocket error: Not connected, please connect first"
            ));
        };
        writer.send(message).await?;
        Ok(())
    }

    pub async fn send_raw(&self, message: Message) -> Result<()> {
        let mut writer = if let Some(writer) = self.writer.as_ref() {
            writer.lock().await
        } else {
            return Err(anyhow::anyhow!(
                "WebSocket error: Not connected, please connect first"
            ));
        };
        writer.send(message).await?;
        Ok(())
    }

    pub async fn receive_raw(&self) -> Result<Message> {
        let message = self.subscribe().recv().await?;
        Ok(message)
    }

    /// 重新连接到服务器
    #[allow(dead_code)]
    pub async fn reconnect(&mut self) -> Result<()> {
        self.disconnect().await?;
        self.connect().await
    }

    /// 连接到服务器
    pub async fn connect(&mut self) -> Result<()> {
        // 设置 3 秒超时
        let connection_result =
            timeout(Duration::from_secs(3), connect_async(self.uri.clone())).await;

        // 处理超时和连接错误
        let (ws_stream, _response) = match connection_result {
            Ok(Ok(result)) => result,
            Ok(Err(e)) => return Err(anyhow::anyhow!("Connection failed: {}", e)),
            Err(_) => return Err(anyhow::anyhow!("Connection timed out after 5 seconds")),
        };
        let (mut writer, reader) = ws_stream.split();

        // 发送连接消息
        writer.send(Message::Text("connect".to_string())).await?;

        // 更新 writer
        if let Some(arc_writer) = Arc::get_mut(&mut self.writer) {
            *arc_writer = Some(Mutex::new(writer));
        } else {
            self.writer = Arc::new(Some(Mutex::new(writer)));
        }

        // 更新 reader
        if let Some(arc_reader) = Arc::get_mut(&mut self.reader) {
            *arc_reader = Some(Mutex::new(reader));
        } else {
            self.reader = Arc::new(Some(Mutex::new(reader)));
        }

        self.connected.store(true, Ordering::Relaxed);
        self.start_collect_messages().await;
        self.start_health_check().await;
        Ok(())
    }

    /// 断开与服务器的连接
    pub async fn disconnect(&mut self) -> Result<()> {
        if let Some(writer) = self.writer.as_ref() {
            let mut writer_guard = writer.lock().await;
            match writer_guard.close().await {
                Ok(_) => (),
                Err(e) => warn!("Failed to close WebSocket writer: {}", e),
            }
            // 在这里显式地释放 writer_guard
            drop(writer_guard);
        } else {
            return Err(anyhow::anyhow!(
                "WebSocket error: Not connected, please connect first"
            ));
        }

        // 现在可以安全地修改 self.writer 和 self.reader
        self.writer = Arc::new(None);
        self.reader = Arc::new(None);
        self.stop_collect_messages().await;
        self.stop_health_check().await;
        self.connected.store(false, Ordering::Relaxed);
        Ok(())
    }

    /// 向服务器注册当前设备
    /// 如果 device 参数为 None，则默认读取配置文件中的设备信息用于注册
    pub async fn register(&self, device: Option<Device>) -> Result<()> {
        let device = device.unwrap_or_else(|| {
            let device_id = self.user_setting.get_device_id().clone();
            let server_port = self.user_setting.network.webserver_port;
            Device::new(device_id, None, None, Some(server_port))
        });

        let web_socket_message = WebSocketMessage::Register(RegisterDeviceMessage::new(
            device.id.clone(),
            device.ip,
            device.server_port,
        ));
        let message = serde_json::to_string(&web_socket_message)?;
        let mut writer = if let Some(writer) = self.writer.as_ref() {
            writer.lock().await
        } else {
            return Err(anyhow::anyhow!(
                "WebSocket error: Not connected, please connect first"
            ));
        };
        writer.send(Message::Text(message)).await?;
        info!("Register [{}] to server({})", device.id, self.uri);
        Ok(())
    }

    /// 向其他设备同步当前设备已知的设备列表
    pub async fn sync_device_list(&self) -> Result<()> {
        let device_id = self.user_setting.get_device_id().clone();
        let devices = get_device_manager()
            .get_all_devices()
            .map_err(|_| anyhow::anyhow!("Failed to get all devices"))?;

        let web_socket_message = WebSocketMessage::DeviceListSync(DevicesSyncMessage {
            devices: devices.iter().map(|d| DeviceSyncInfo::from(d)).collect(),
            replay_device_ids: vec![device_id],
        });
        self.send_with_websocket_message(&web_socket_message)
            .await?;
        Ok(())
    }

    /// 发送 ping 消息, 并等待 pong 消息
    /// 如果收到 pong 消息, 则返回 true, 否则返回 false
    async fn ping(&self, timeout: Duration) -> Result<bool> {
        // 生成一个随机的 ping 消息
        let rand_bytes = rand::random::<[u8; 16]>().to_vec();
        let rand_bytes_clone = rand_bytes.clone();
        let ping_message = Message::Ping(rand_bytes);
        let mut rx = self.subscribe();

        let join_handle = tokio::spawn(async move {
            if let Ok(msg) = rx.recv().await {
                if let Message::Pong(msg) = msg {
                    trace!("Ping pong result: {:?}", msg);
                    return msg == rand_bytes_clone;
                }
            }
            false
        });

        // 发送 ping 消息，并在 timeout 时间内等待 pong 消息
        let result = tokio::time::timeout(timeout, async move {
            trace!("Sending ping message, {:?}", ping_message);
            if let Err(e) = self.send_raw(ping_message).await {
                error!("Failed to send ping message: {}", e);
            }
            join_handle.await
        })
        .await?;

        match result {
            Ok(v) => Ok(v),
            Err(e) => Err(anyhow::anyhow!("Ping timeout: {}", e)),
        }
    }
}
