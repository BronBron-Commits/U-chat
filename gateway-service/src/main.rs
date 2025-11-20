use axum::{
    extract::{
        ws::{Message, WebSocket},
        Query, State, WebSocketUpgrade,
    },
    response::{IntoResponse, Html},
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

    let listener = tokio::net::TcpListener::bind("0.0.0.0:9000")
        .await
        .unwrap();

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(WsQuery { token }): Query<WsQuery>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let validation = Validation::default();
    let key = DecodingKey::from_secret(b"secret");

    if decode::<serde_json::Value>(&token, &key, &validation).is_err() {
        return Html("INVALID TOKEN");
    }

    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut write, mut read) = socket.split();
    let mut rx = state.tx.subscribe();

    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if write.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    let recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = read.next().await {
            let _ = state.tx.send(text);
        }
    });

    tokio::select! {
        _ = send_task => (),
        _ = recv_task => (),
    }
}
