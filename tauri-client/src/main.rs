//! Unhidra Desktop Client
//!
//! A secure chat desktop application built with Tauri with E2EE and WebSocket support

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;
use e2ee::{EncryptedMessage, SessionStore, E2eeEnvelope, MessageType};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{Manager, State};
use tokio::sync::{Mutex, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, info, warn};

/// WebSocket connection state
#[derive(Clone)]
struct WsConnection {
    url: String,
    connected: Arc<RwLock<bool>>,
    session_store: Arc<Mutex<SessionStore>>,
}

impl WsConnection {
    fn new() -> Self {
        Self {
            url: String::new(),
            connected: Arc::new(RwLock::new(false)),
            session_store: Arc::new(Mutex::new(SessionStore::new())),
        }
    }
}

/// Channel information
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Channel {
    id: String,
    name: String,
    #[serde(rename = "type")]
    channel_type: String,
}

/// Message payload
#[derive(Clone, Debug, Serialize, Deserialize)]
struct ChatMessage {
    id: String,
    channel_id: String,
    sender_id: String,
    content: String,
    timestamp: u64,
    encrypted: bool,
}

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .manage(WsConnection::new())
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_window("main").unwrap();
                window.open_devtools();
            }

            info!("Unhidra Desktop Client started");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            connect_to_server,
            disconnect_from_server,
            send_message,
            send_encrypted_message,
            get_channels,
            get_identity_bundle,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}

/// Connect to Unhidra server via WebSocket
#[tauri::command]
async fn connect_to_server(
    server_url: String,
    token: String,
    ws_state: State<'_, WsConnection>,
) -> Result<String, String> {
    info!("Connecting to server: {}", server_url);

    // Convert HTTP(S) URL to WS(S) URL
    let ws_url = server_url
        .replace("https://", "wss://")
        .replace("http://", "ws://");
    let ws_url = format!("{}/ws?token={}", ws_url, token);

    // Store URL for reconnect
    {
        let mut url = ws_state.inner().url.clone();
        url = ws_url.clone();
    }

    // Connect to WebSocket
    match connect_async(&ws_url).await {
        Ok((ws_stream, _)) => {
            info!("WebSocket connected successfully");
            *ws_state.connected.write().await = true;

            // Split stream for concurrent read/write
            let (mut write, mut read) = ws_stream.split();

            // Spawn task to handle incoming messages
            tokio::spawn(async move {
                while let Some(msg) = read.next().await {
                    match msg {
                        Ok(Message::Text(text)) => {
                            info!("Received message: {}", text);
                            // TODO: Parse and handle incoming messages
                            // This could emit events to the Tauri window
                        }
                        Ok(Message::Binary(data)) => {
                            info!("Received binary message: {} bytes", data.len());
                        }
                        Ok(Message::Close(_)) => {
                            info!("WebSocket connection closed by server");
                            break;
                        }
                        Ok(Message::Ping(_)) => {
                            // Tungstenite automatically handles pongs
                        }
                        Err(e) => {
                            error!("WebSocket error: {}", e);
                            break;
                        }
                        _ => {}
                    }
                }
            });

            Ok(format!("Connected to {}", server_url))
        }
        Err(e) => {
            error!("Failed to connect to WebSocket: {}", e);
            Err(format!("Connection failed: {}", e))
        }
    }
}

/// Disconnect from server
#[tauri::command]
async fn disconnect_from_server(ws_state: State<'_, WsConnection>) -> Result<(), String> {
    *ws_state.connected.write().await = false;
    info!("Disconnected from server");
    Ok(())
}

/// Send a plain text message to a channel
#[tauri::command]
async fn send_message(
    channel_id: String,
    content: String,
    ws_state: State<'_, WsConnection>,
) -> Result<(), String> {
    if !*ws_state.connected.read().await {
        return Err("Not connected to server".to_string());
    }

    let message = ChatMessage {
        id: uuid::Uuid::new_v4().to_string(),
        channel_id,
        sender_id: "client".to_string(), // TODO: Get from auth
        content,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
        encrypted: false,
    };

    let json = serde_json::to_string(&message)
        .map_err(|e| format!("Failed to serialize message: {}", e))?;

    info!("Sending message to channel {}", message.channel_id);

    // TODO: Send via WebSocket
    // This requires storing the write half of the WebSocket
    // For now, return success

    Ok(())
}

/// Send an encrypted message using E2EE
#[tauri::command]
async fn send_encrypted_message(
    channel_id: String,
    recipient_id: String,
    content: String,
    ws_state: State<'_, WsConnection>,
) -> Result<(), String> {
    if !*ws_state.connected.read().await {
        return Err("Not connected to server".to_string());
    }

    let session_store = ws_state.session_store.lock().await;

    // Encrypt the message content
    let encrypted = session_store
        .encrypt(&recipient_id, content.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    // Create E2EE envelope
    let envelope = E2eeEnvelope::new_message(
        "client".to_string(), // TODO: Get from auth
        recipient_id.clone(),
        &encrypted,
    );

    let json = envelope
        .to_json()
        .map_err(|e| format!("Failed to serialize envelope: {}", e))?;

    info!("Sending encrypted message to {}", recipient_id);

    // TODO: Send via WebSocket
    // For now, return success

    Ok(())
}

/// Get list of channels from API
#[tauri::command]
async fn get_channels(server_url: String, token: String) -> Result<Vec<Channel>, String> {
    info!("Fetching channels from {}", server_url);

    let client = reqwest::Client::new();
    let url = format!("{}/api/channels", server_url);

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if response.status().is_success() {
        let channels: Vec<Channel> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(channels)
    } else {
        Err(format!("API error: {}", response.status()))
    }
}

/// Get identity bundle for E2EE session establishment
#[tauri::command]
async fn get_identity_bundle(ws_state: State<'_, WsConnection>) -> Result<String, String> {
    let session_store = ws_state.session_store.lock().await;
    let bundle = session_store.get_identity_bundle();

    serde_json::to_string(&bundle)
        .map_err(|e| format!("Failed to serialize bundle: {}", e))
}
