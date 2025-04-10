#![allow(dead_code)]

use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, instrument};

// 状态结构体
#[derive(Clone)]
pub struct AppState {
    pub(crate) message: String,
}

// 提取器：请求体
#[derive(Deserialize)]
pub struct EchoRequest {
    text: String,
}

// 响应体
#[derive(Serialize)]
pub struct EchoResponse {
    echoed: String,
}

// GET /health Handler
#[instrument(skip(state), fields(layer = "appv2"), target = "service::health")]
pub async fn health_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    info!(target: "service::health", "health check");
    state.message.clone()
}

// POST /echo Handler
#[instrument(
    skip(state, payload),
    fields(layer = "appv2"),
    target = "service::echo"
)]
pub async fn echo_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<EchoRequest>,
) -> impl IntoResponse {
    let echoed = format!("{}: {}", state.message, payload.text);
    info!(target: "service::echo_v2", "Transformed data: {:?}", echoed);
    (StatusCode::OK, Json(EchoResponse { echoed }))
}
