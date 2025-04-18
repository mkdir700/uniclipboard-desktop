use crate::utils::errors::LockError;

use super::routes::{device, download, websocket};
use anyhow::Result;
use log::warn;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::{oneshot, Mutex};
use warp::Filter;

use super::handlers::websocket::WebSocketHandler;
use super::response::ApiResponse;
use warp::Rejection;


pub async fn handle_rejection(
    err: Rejection,
) -> Result<impl warp::Reply, std::convert::Infallible> {
    if err.is_not_found() {
        // 直接返回 404 错误，不使用 ApiResponse
        return Ok(ApiResponse::<()>::error(
            warp::http::StatusCode::NOT_FOUND.as_u16(),
            "未找到资源".to_string(),
        )
        .into_response()
        .unwrap());
    }

    let (code, message) = if let Some(lock_error) = err.find::<LockError>() {
        (
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            lock_error.to_string(),
        )
    } else if let Some(warp_error) = err.find::<warp::Error>() {
        (
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            warp_error.to_string(),
        )
    } else {
        println!("Unhandled rejection: {:?}", err);
        warn!("Unhandled rejection: {:?}", err);
        (
            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            "内部服务器错误".to_string(),
        )
    };

    Ok(ApiResponse::<()>::error(code.as_u16(), message)
        .into_response()
        .unwrap())
}


// 定义 WebServer 结构体
pub struct WebServer {
    address: SocketAddr,
    websocket_handler: Arc<WebSocketHandler>,
    shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

impl WebServer {
    // 创建新的 WebServer 实例
    pub fn new(address: SocketAddr, websocket_handler: Arc<WebSocketHandler>) -> Self {
        Self {
            address,
            websocket_handler,
            shutdown_tx: Arc::new(Mutex::new(None)),
        }
    }

    // 启动 web 服务器的方法
    pub async fn run(&self) -> Result<()> {
        // API 路由
        let api_routes = warp::path("api").and(download::route().or(device::route()));
        // websocket 路由
        let websocket_routes = websocket::route(Arc::clone(&self.websocket_handler));

        // 合并路由
        let routes = api_routes.or(websocket_routes).recover(handle_rejection);

        // 创建关闭通道
        let (tx, rx) = oneshot::channel();
        {
            *self.shutdown_tx.lock().await = Some(tx);
        }

        // 启动服务器
        let (_, server) = warp::serve(routes).bind_with_graceful_shutdown(self.address, async {
            rx.await.ok();
        });

        // 运行服务器
        server.await;

        Ok(())
    }

    pub async fn shutdown(&self) -> Result<()> {
        if let Some(tx) = self.shutdown_tx.lock().await.take() {
            tx.send(())
                .map_err(|_| anyhow::anyhow!("无法发送关闭信号"))?;
        }
        Ok(())
    }
}
