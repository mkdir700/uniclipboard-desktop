pub mod handlers;
mod response;
mod routes;
pub mod server;
pub use handlers::websocket::WebSocketHandler;
pub use handlers::websocket_message::WebSocketMessageHandler;
use log::warn;
pub use server::WebServer;

use crate::errors::LockError;
use crate::web::response::ApiResponse;
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
