use axum::{
    extract::{Query, WebSocketUpgrade},
    response::IntoResponse,
    Router,
    routing::get,
};
use axum::extract::ws::{Message, WebSocket};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Clone)]
struct AppState {
    jwt_secret: String,
    tx: broadcast::Sender<String>,
}

#[derive(Deserialize)]
struct WsParams {
    token: String,
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<WsParams>,
    axum::extract::State(app): axum::extract::State<Arc<AppState>>,
) -> impl IntoResponse {
    let validation = Validation::default();
    let token_result = decode::<serde_json::Value>(
        &params.token,
        &DecodingKey::from_secret(app.jwt_secret.as_bytes()),
        &validation,
    );

    if token_result.is_err() {
        return "INVALID TOKEN";
    }

    ws.on_upgrade(move |socket| handle_socket(socket, app.clone()))
}

async fn handle_socket(mut socket: WebSocket, app: Arc<AppState>) {
    let mut rx = app.tx.subscribe();

    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if socket.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    let tx2 = app.tx.clone();
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = socket.recv().await {
            let _ = tx2.send(text.to_string());
        }
    });

    let _ = tokio::join!(send_task, recv_task);
}

#[tokio::main]
async fn main() {
    let (tx, _) = broadcast::channel::<String>(100);

    let state = Arc::new(AppState {
        jwt_secret: "supersecretjwtkey".into(),
        tx,
    });

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(state);

    println!("Gateway running on 0.0.0.0:9000");

    axum::serve(
        tokio::net::TcpListener::bind("0.0.0.0:9000").await.unwrap(),
        app
    )
    .await
    .unwrap();
}
