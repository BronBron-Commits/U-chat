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
    let state = Arc::new(AppState { tx });

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(state);

    println!("Gateway running on 0.0.0.0:9000");

    axum::serve(
        tokio::net::TcpListener::bind("0.0.0.0:9000")
            .await
            .unwrap(),
        app,
    )
    .await
    .unwrap();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(query): Query<WsQuery>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // Validate JWT
    let secret = std::env::var("JWT_SECRET").unwrap_or("secret".into());
    let validation = Validation::default();

    let token_ok = decode::<serde_json::Value>(
        &query.token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .is_ok();

    if !token_ok {
        return "INVALID TOKEN".into_response();
    }

    // Good token → upgrade
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    let mut rx = state.tx.subscribe();
    let tx = state.tx.clone();

    let mut write = socket; // move socket into writer
    let mut read = write.split(); // reader/writer split

    let mut writer = read.0;
    let mut reader = read.1;

    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let _ = writer.send(Message::Text(msg)).await;
        }
    });

    let recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = reader.next().await {
            let _ = tx.send(text);
        }
    });

    let _ = tokio::join!(send_task, recv_task);
}
