use axum::{http::StatusCode, response::IntoResponse};
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Error as HyperError, Response};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("MigrateError error:{0}")]
    MigrateError(#[from] sqlx::migrate::MigrateError),
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Not Found: {0}")]
    NotFound(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AppError::InvalidInput(msg) => {
                (StatusCode::BAD_REQUEST, format!("Invalid input: {}", msg))
            }
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, format!("Not Found: {}", msg)),
            // AppError::Serialization(err) => (
            //     StatusCode::INTERNAL_SERVER_ERROR,
            //     format!("Serialization error: {}", err),
            // ),
            AppError::Database(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::MigrateError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::Redis(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::Serialization(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        };
        (status, message).into_response()
    }
}

// for tower service

impl From<HyperError> for AppError {
    fn from(err: HyperError) -> Self {
        AppError::InvalidInput(format!("Failed to read request body: {}", err))
    }
}

impl AppError {
    #[allow(dead_code)]
    pub fn into_tower_response(self) -> Result<Response<Full<Bytes>>, AppError> {
        let (status, message) = match self {
            AppError::InvalidInput(msg) => {
                (StatusCode::BAD_REQUEST, format!("Invalid input: {}", msg))
            }
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, format!("Not Found: {}", msg)),
            AppError::Database(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::MigrateError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::Redis(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::Serialization(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        };
        let body = serde_json::to_vec(&serde_json::json!({ "error": message }))?;
        Ok(Response::builder()
            .status(status)
            .header("Content-Type", "application/json")
            .body(Full::new(Bytes::from(body)))
            .map_err(|e| {
                AppError::InvalidInput(format!(
                    "Failed to construct response body:{}",
                    e.to_string()
                ))
            })?)
    }
}
