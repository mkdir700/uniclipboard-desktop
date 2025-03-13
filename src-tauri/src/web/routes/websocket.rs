use crate::web::handlers::websocket::WebSocketHandler;
use std::net::SocketAddr;
use std::sync::Arc;

use warp::Filter;

pub fn route(
    websocket_handler: Arc<WebSocketHandler>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("ws")
        .and(warp::ws())
        .and(warp::addr::remote())
        .and(warp::any().map(move || Arc::clone(&websocket_handler)))
        .map(
            |ws: warp::ws::Ws, addr: Option<SocketAddr>, handler: Arc<WebSocketHandler>| {
                ws.on_upgrade(move |socket| {
                    let handler = Arc::clone(&handler);
                    async move { handler.client_connected(socket, addr).await }
                })
            },
        )
}
