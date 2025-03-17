use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use log::{error, info};
use tokio::sync::{broadcast, RwLock};
use warp::filters::ws::{Message as WarpMessage, WebSocket};

#[derive(Clone)]
pub enum WebSocketMessage {
    Message(WarpMessage),
    Close,
}

#[derive(Clone)]
pub struct IncommingWebsocketClient {
    addr: SocketAddr,
    ws_sender: Arc<RwLock<SplitSink<WebSocket, WarpMessage>>>,
    ws_receiver: Arc<RwLock<SplitStream<WebSocket>>>,
    health_check_join_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
    connected: Arc<AtomicBool>,
    receive_message_tx: broadcast::Sender<WebSocketMessage>,
    receive_messages_join_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
    send_messages_tx: broadcast::Sender<WebSocketMessage>,
    send_messages_join_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

impl IncommingWebsocketClient {
    pub fn new(addr: SocketAddr, ws: WebSocket) -> Self {
        let (ws_sender, ws_receiver) = ws.split();
        let (send_messages_tx, _) = broadcast::channel(100);
        let (receive_message_tx, _) = broadcast::channel(100);

        Self {
            addr,
            ws_sender: Arc::new(RwLock::new(ws_sender)),
            ws_receiver: Arc::new(RwLock::new(ws_receiver)),
            send_messages_tx,
            receive_message_tx,
            health_check_join_handle: Arc::new(RwLock::new(None)),
            connected: Arc::new(AtomicBool::new(true)),
            receive_messages_join_handle: Arc::new(RwLock::new(None)),
            send_messages_join_handle: Arc::new(RwLock::new(None)),
        }
    }

    pub fn id(&self) -> String {
        format!("{}:{}", self.addr.ip(), self.addr.port())
    }

    /// 将消息加入到广播队列中， 等待发送给客户端
    pub async fn send(&self, msg: WarpMessage) -> anyhow::Result<()> {
        self.send_messages_tx
            .send(WebSocketMessage::Message(msg))
            .map_err(|e| anyhow::anyhow!("Failed to send message: {}", e))?;
        Ok(())
    }

    /// 发送关闭消息
    pub async fn close(&self) -> anyhow::Result<()> {
        self.send_messages_tx
            .send(WebSocketMessage::Close)
            .map_err(|_| anyhow::anyhow!("Failed to send close message"))?;
        Ok(())
    }

    /// 开启消息发送、接收、健康检查的异步任务
    pub async fn start(&self) -> anyhow::Result<()> {
        self.start_send_messages().await;
        self.start_receive_messages().await;
        self.start_health_check().await;
        Ok(())
    }

    /// 停止消息发送、接收、健康检查的异步任务
    pub async fn stop(&self) -> anyhow::Result<()> {
        self.close().await?;
        self.stop_send_messages().await;
        self.stop_receive_messages().await;
        self.stop_health_check().await;
        Ok(())
    }

    /// 开启一个异步任务，持续的从 message_tx 中接收消息并发送给客户端
    async fn start_send_messages(&self) {
        let ws_sender = self.ws_sender.clone();
        let mut rx = self.send_messages_tx.subscribe();

        *self.send_messages_join_handle.write().await = Some(tokio::spawn(async move {
            loop {
                if let Ok(msg) = rx.recv().await {
                    match msg {
                        WebSocketMessage::Message(msg) => {
                            if let Err(e) = ws_sender.write().await.send(msg).await {
                                error!("Error sending message to client: {}", e);
                            }
                        }
                        WebSocketMessage::Close => {
                            info!("Client closed");
                            break;
                        }
                    }
                }
            }
        }));
    }

    /// 停止消息发送的异步任务
    async fn stop_send_messages(&self) {
        if let Some(handle) = self.send_messages_join_handle.write().await.take() {
            handle.abort();
        }
    }

    /// 开启一个异步任务，持续的从 ws_receiver 中接收消息并转发到 message_tx
    async fn start_receive_messages(&self) {
        let reader = self.ws_receiver.clone();
        let message_tx = self.receive_message_tx.clone();
        let connected = self.connected.clone();

        *self.receive_messages_join_handle.write().await = Some(tokio::spawn(async move {
            loop {
                if !connected.load(Ordering::Relaxed) {
                    let _ = message_tx.send(WebSocketMessage::Close);
                    break;
                }

                reader.write().await.next().await.map(|msg| match msg {
                    Ok(msg) => {
                        let _ = message_tx.send(WebSocketMessage::Message(msg));
                    }
                    Err(e) => {
                        error!("Error receiving message from client: {}", e);
                    }
                });
            }
        }));
    }

    /// 停止接收消息的异步任务
    async fn stop_receive_messages(&self) {
        if let Some(handle) = self.receive_messages_join_handle.write().await.take() {
            handle.abort();
        }
    }

    /// 开启一个异步任务，持续的向服务器发送 ping 消息，并等待 pong 消息
    /// 如果连续3次发送 ping 消息失败，则认为连接断开，并修改 connected 状态为 false
    async fn start_health_check(&self) {
        let self_clone = self.clone();
        let connected = self.connected.clone();
        let message_tx = self.receive_message_tx.clone();
        
        *self.health_check_join_handle.write().await = Some(tokio::spawn(async move {
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
                    let _ = message_tx.send(WebSocketMessage::Close);
                    break;
                }

                if let Err(e) = self_clone.ping(Duration::from_secs(3)).await {
                    error!(
                        "WebSocket health check failed: {}, count: {}",
                        e, ping_count
                    );
                    ping_count += 1;
                }
            }
        }));
    }

    /// 停止健康检查的异步任务
    async fn stop_health_check(&self) {
        if let Some(handle) = self.health_check_join_handle.write().await.take() {
            handle.abort();
        }
    }

    /// 发送 ping 消息, 并等待 pong 消息
    /// 如果收到 pong 消息, 则返回 true, 否则返回 false
    async fn ping(&self, timeout: Duration) -> Result<bool> {
        // 生成一个随机的 ping 消息
        let rand_bytes = rand::random::<[u8; 16]>().to_vec();
        let rand_bytes_clone = rand_bytes.clone();
        let ping_message = WarpMessage::ping(rand_bytes);
        let mut rx = self.receive_message_tx.subscribe();

        let join_handle = tokio::spawn(async move {
            if let Ok(msg) = rx.recv().await {
                match msg {
                    WebSocketMessage::Message(msg) => {
                        if msg.is_pong() {
                            return msg.as_bytes() == rand_bytes_clone;
                        }
                    }
                    WebSocketMessage::Close => {
                        return false;
                    }
                }
            }
            false
        });

        // 发送 ping 消息，并在 timeout 时间内等待 pong 消息
        let result = tokio::time::timeout(timeout, async move {
            let _ = self.send(ping_message).await;
            join_handle.await
        })
        .await?;

        match result {
            Ok(v) => Ok(v),
            Err(e) => Err(anyhow::anyhow!("Ping timeout: {}", e)),
        }
    }

    /// 订阅接收消息的广播通道
    pub fn subscribe_messages(&self) -> broadcast::Receiver<WebSocketMessage> {
        self.receive_message_tx.subscribe()
    }
}
