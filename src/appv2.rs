#![allow(dead_code)]

use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, instrument};
use utoipa::ToSchema;

// 状态结构体
#[derive(Clone)]
pub struct AppState {
    pub(crate) message: String,
}

// 提取器：请求体
#[derive(Deserialize, ToSchema)]
pub struct EchoRequest {
    text: String,
}

// 响应体
#[derive(Serialize, ToSchema)]
pub struct EchoResponse {
    echoed: String,
}

// GET /health Handler
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Return the server's health message", body = String)
    ),
    params(
        ("Auth-Key", Header, description = "custom auth middleware token"),
    )
)]
#[instrument(skip(state), fields(layer = "appv2"), target = "service::health")]
pub async fn health_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    info!(target: "service::health", "health check");
    state.message.clone()
}

// POST /echo Handler
#[utoipa::path(
    post,
    path = "/echo",
    request_body = EchoRequest,
    responses(
        (status = 200, description = "Echo the input text with server message", body = EchoResponse)
    ),
    params(
        ("Auth-Key", Header, description = "custom auth middleware token"),
    )
)]
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

// // OpenAPI 文档定义
// #[derive(OpenApi)]
// #[openapi(
//     paths(health_handler, echo_handler),
//     components(schemas(EchoRequest, EchoResponse)),
//     info(
//         title = "My Echo API",
//         version = "1.0.0",
//         description = "A simple echo service with tracing"
//     )
// )]
// pub struct ApiDoc;
