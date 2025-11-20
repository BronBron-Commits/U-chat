use axum::{
    extract::{
        ws::{Message, WebSocket},
        Query, State, WebSocketUpgrade,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Clone)]
struct AppState {
    tx: broadcast::Sender<String>,
}

#[derive(Deserialize)]
struct WsQuery {
    token: String,
}

#[tokio::main]
async fn main() {
    let (tx, _) = broadcast::channel::<String>(100);
    let app_state = Arc::new(AppState { tx });

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(app_state);

    println!("Gateway running on 0.0.0.0:9000");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:9000")
        .await
        .expect("bind failed");
    axum::serve(listener, app).await.expect("server failed");
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(query): Query<WsQuery>,
    State(app): State<Arc<AppState>>,
) -> impl IntoResponse {
    let valid = decode::<serde_json::Value>(
        &query.token,
        &DecodingKey::from_secret(b"secret"),
        &Validation::default(),
    )
    .is_ok();

    if !valid {
        return axum::response::Html("INVALID TOKEN");
    }

    ws.on_upgrade(|socket| handle_socket(socket, app))
}

async fn handle_socket(socket: WebSocket, app: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = app.tx.subscribe();
    let tx = app.tx.clone();

    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let _ = sender.send(Message::Text(msg)).await;
        }
    });

    let recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            let _ = tx.send(text);
        }
    });

    let _ = tokio::join!(send_task, recv_task);
}
