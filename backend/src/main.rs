mod executor;

use axum::{
    routing::{get, post},
    http::StatusCode,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing_subscriber;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

#[derive(Deserialize)]
struct CodeExecutionRequest {
    code: String,
    mode: String,
}

#[derive(Serialize)]
struct CodeExecutionResponse {
    success: bool,
    output: String,
    error: Option<String>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cors = CorsLayer::permissive();

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/api/execute", post(execute_code))
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Rust Book Online API"
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: "0.1.0".to_string(),
    })
}

async fn execute_code(
    Json(payload): Json<CodeExecutionRequest>,
) -> Result<Json<CodeExecutionResponse>, StatusCode> {
    match crate::executor::execute_rust_code(payload.code).await {
        Ok(result) => {
            Ok(Json(CodeExecutionResponse {
                success: result.success,
                output: result.stdout,
                error: if result.stderr.is_empty() {
                    None
                } else {
                    Some(result.stderr)
                },
            }))
        }
        Err(e) => {
            Ok(Json(CodeExecutionResponse {
                success: false,
                output: String::new(),
                error: Some(e),
            }))
        }
    }
}