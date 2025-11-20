use axum::{Router, routing::post, routing::get, Json};
use serde_json::json;

async fn login_handler() -> &'static str {
    "login placeholder"
}

async fn health_handler() -> Json<serde_json::Value> {
    Json(json!({"status": "ok"}))
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/login", post(login_handler))
        .route("/health", get(health_handler));

    let addr = "0.0.0.0:9200".parse().unwrap();
    println!("Auth API running on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
