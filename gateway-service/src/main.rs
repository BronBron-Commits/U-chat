//! Gateway Service - WebSocket Real-Time Fabric
//!
//! This service provides secure WebSocket connections for real-time
//! bidirectional communication between browser clients and IoT devices.
//!
//! # Security Features
//! - JWT token authentication via Sec-WebSocket-Protocol header
//! - Origin validation for CSRF protection
//! - Room-based isolation with broadcast channels
//! - TLS encryption required in production (wss://)
//!
//! # Architecture
//! - DashMap for lock-free concurrent room management
//! - Tokio broadcast channels for efficient fan-out messaging
//! - Automatic resource cleanup on disconnect

mod state;
mod ws_handler;

use axum::{http::Method, routing::get, Router};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

use state::AppState;
use ws_handler::ws_handler;

/// Server configuration from environment variables.
struct Config {
    /// Server bind address
    bind_addr: String,
    /// JWT secret for token validation
    jwt_secret: String,
    /// Allowed origins for WebSocket connections (comma-separated)
    allowed_origins: Vec<String>,
}

impl Config {
    fn from_env() -> Self {
        let bind_addr =
            std::env::var("GATEWAY_BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:9000".to_string());

        let jwt_secret =
            std::env::var("JWT_SECRET").unwrap_or_else(|_| "supersecret".to_string());

        let allowed_origins = std::env::var("ALLOWED_ORIGINS")
            .map(|s| s.split(',').map(|o| o.trim().to_string()).collect())
            .unwrap_or_else(|_| {
                // Default: allow localhost for development
                vec![
                    "http://localhost:3000".to_string(),
                    "http://127.0.0.1:3000".to_string(),
                ]
            });

        Self {
            bind_addr,
            jwt_secret,
            allowed_origins,
        }
    }
}

#[tokio::main]
async fn main() {
    // Initialize tracing for structured logging
    tracing_subscriber_init();

    let config = Config::from_env();

    // Create shared application state
    let state = Arc::new(AppState::new(
        config.jwt_secret,
        config.allowed_origins.clone(),
    ));

    // Configure CORS for WebSocket handshake
    // Note: WebSocket connections use GET method for upgrade
    let cors = CorsLayer::new()
        .allow_methods([Method::GET])
        .allow_headers(Any)
        .allow_origin(
            config
                .allowed_origins
                .iter()
                .filter_map(|o| o.parse().ok())
                .collect::<Vec<_>>(),
        );

    // Build the router with WebSocket endpoint
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/health", get(health_check))
        .layer(cors)
        .with_state(state);

    info!(
        bind_addr = config.bind_addr,
        origins = ?config.allowed_origins,
        "Gateway service starting"
    );

    // Start the server
    let listener = tokio::net::TcpListener::bind(&config.bind_addr)
        .await
        .expect("Failed to bind to address");

    info!("Gateway service running on {}", config.bind_addr);

    axum::serve(listener, app)
        .await
        .expect("Server failed to start");
}

/// Health check endpoint for load balancers and monitoring.
async fn health_check() -> &'static str {
    "OK"
}

/// Initialize tracing subscriber for structured logging.
fn tracing_subscriber_init() {
    use tracing_subscriber::{fmt, EnvFilter};

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("gateway_service=info,tower_http=info"));

    fmt().with_env_filter(filter).init();
}
