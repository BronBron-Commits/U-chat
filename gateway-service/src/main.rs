use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade, Utf8Bytes},
    extract::ConnectInfo,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use std::{collections::HashMap, net::SocketAddr, sync::{Arc, Mutex}};
use tokio::sync::broadcast;

#[tokio::main]
async fn main() {
    let state = Arc::new(Mutex::new(HashMap::<SocketAddr, broadcast::Sender<String>>::new()));

    let app = Router::new().route("/ws", get(ws_handler.with_state(state.clone())));

    println!("Gateway running on 0.0.0.0:9000");

    axum::serve(
        tokio::net::TcpListener::bind("0.0.0.0:9000").await.unwrap(),
        app,
    )
    .await
    .unwrap();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    state: axum::extract::State<Arc<Mutex<HashMap<SocketAddr, broadcast::Sender<String>>>>>,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, addr, state))
}

async fn handle_socket(
    socket: WebSocket,
    addr: SocketAddr,
    state: axum::extract::State<Arc<Mutex<HashMap<SocketAddr, broadcast::Sender<String>>>>>,
) {
    let (mut sender, mut receiver) = socket.split();

    let (tx, _rx) = broadcast::channel::<String>(100);

    state.lock().unwrap().insert(addr, tx.clone());

    // Incoming loop (from WebSocket)
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text_bytes) = msg {
                let text: String = text_bytes.to_string();

                let _ = tx_clone.send(text.clone());
            }
        }
    });

    // Outgoing loop (broadcast -> WebSocket)
    let mut rx = tx.subscribe();
    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let _ = sender.send(Message::Text(msg.into())).await;
        }
    });
}
