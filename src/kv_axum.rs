use crate::{
    cache::CacheClient,
    db::DBClient,
    error::AppError,
    models::{CreateKv, KvPair},
};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use std::sync::Arc;
use tracing::instrument;

#[allow(dead_code)]
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DBClient>,
    pub cache: Arc<CacheClient>,
}

#[utoipa::path(
    post,
    path = "/kv",
    request_body = CreateKv,
    responses(
        (status = 201, description = "Key-value pair created", body = KvPair),
        (status = 400, description = "Invalid input"),
        (status = 409, description = "Key already exists")
    )
)]
#[instrument(
    skip(state, payload),
    fields(layer = "kv_axum"),
    target = "service::kv"
)]
pub async fn set_kv(
    State(state): State<AppState>,
    Json(payload): Json<CreateKv>,
) -> Result<(StatusCode, Json<KvPair>), AppError> {
    tracing::info!(target: "service::kv", %payload, "📥 incoming set key-value request");
    if payload.key.is_empty()
        || payload.key.len() > 50
        || !payload.key.chars().all(|c| c.is_alphanumeric() || c == '_')
    {
        return Err(AppError::InvalidInput("Invalid key".into()));
    }
    if payload.value.is_empty() || payload.value.len() > 1000 {
        return Err(AppError::InvalidInput("Invalid value".into()));
    }

    // set to db
    tracing::info!(target: "service::kv", %payload, "✏️ update db");
    let kv = state.db.set(payload.clone()).await?;

    // update cache
    tracing::info!(target: "service::kv", %payload, "✏️ update cache");
    state.cache.set(&format!("kv:{}", kv.key), &kv, 300).await?;
    tracing::info!(target: "service::kv", %payload, "📦 set key-value successful");
    Ok((StatusCode::CREATED, Json(kv)))
}

#[utoipa::path(
    put,
    path = "/kv/{key}",
    params(
        ("key", Path, description = "Key to update")
    ),
    request_body(
        content = String,
        description = "New value for the key as a JSON string",
        content_type = "application/json",
        example = json!("new_value_of_the_key")
    ),
    responses(
        (status = 200, description = "Key-value pair updated", body = KvPair),
        (status = 400, description = "Invalid input"),
        (status = 404, description = "Key not found")
    )
)]
#[instrument(skip(state), fields(layer = "kv_axum"), target = "service::kv")]
pub async fn update_kv(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Json(value): Json<String>,
) -> Result<Json<KvPair>, AppError> {
    // TODO: 这里更新会造成缓存脏读的问题，需要后续优化
    tracing::info!(target: "service::kv", %key, %value, "📥 incoming update request");
    if value.is_empty() || value.len() > 1000 {
        return Err(AppError::InvalidInput("Invalid value".into()));
    }

    // update db
    tracing::info!(target: "service::kv", %key, %value, "✏️ update db");
    let kv = state.db.update(&key, &value).await?;
    // update cache
    tracing::info!(target: "service::kv", %key, %value, "✏️ update cache");
    state.cache.set(&format!("kv:{}", key), &kv, 300).await?;
    tracing::info!(target: "service::kv", %key, %value, "📦 update successful");
    Ok(Json(kv))
}

#[utoipa::path(
    get,
    path = "/kv/{key}",
    params(
        ("key", Path, description = "Key to retrieve")
    ),
    responses(
        (status = 200, description = "Key-value pair found", body = KvPair),
        (status = 404, description = "Key not found")
    )
)]
#[instrument(skip(state), fields(layer = "kv_axum"), target = "service::kv")]
pub async fn get_kv(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Result<Json<KvPair>, AppError> {
    tracing::info!(target: "service::kv", %key, "📥 incoming get request");

    let cache_key = format!("kv:{}", key);
    if let Some(kv) = state.cache.get::<KvPair>(&cache_key).await? {
        tracing::info!(target: "service::kv", %key, "✅ cache hit");

        return Ok(Json(kv));
    }
    tracing::info!(target: "service::kv", %key, "⚠️ cache miss");

    let kv = state.db.get(&key).await?.ok_or_else(|| {
        tracing::warn!(target: "service::kv", %key, "⚠️  key not found in db");
        AppError::NotFound(format!("Key {} not found", key))
    })?;

    tracing::info!(target: "service::kv", %key, "✏️ update cache");
    state.cache.set(&cache_key, &kv, 300).await?;
    tracing::info!(target: "service::kv", %key, "📦 fetched from db");

    Ok(Json(kv))
}

#[utoipa::path(
    delete,
    path = "/kv/{key}",
    params(
        ("key", Path, description = "Key to delete")
    ),
    responses(
        (status = 204, description = "Key deleted"),
        (status = 404, description = "Key not found")
    )
)]
#[instrument(skip(state), fields(layer = "kv_axum"), target = "service::kv")]
pub async fn delete_kv(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Result<StatusCode, AppError> {
    tracing::info!(target: "service::kv", %key, "📥 incoming delete request");

    tracing::info!(target: "service::kv", %key, "🗑️ delete from db");
    let deleted = state.db.delete(&key).await?;

    if !deleted {
        return Err(AppError::NotFound(format!("Key {} not found", key)));
    }

    tracing::info!(target: "service::kv", %key, "🗑️ delete from cache");
    state.cache.delete(&format!("kv:{}", key)).await?;

    tracing::info!(target: "service::kv", %key, "📦 delete successful");
    Ok(StatusCode::NO_CONTENT)
}

#[allow(dead_code)]
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/kv", post(set_kv))
        .route("/kv/{key}", get(get_kv).delete(delete_kv).put(update_kv))
        .with_state(state)
}

// #[derive(OpenApi)]
// #[openapi(
//     paths(set_kv, update_kv, get_kv, delete_kv),
//     components(schemas(CreateKv, KvPair)),
//     info(
//         title = "Key-Value Store API",
//         version = "1.0.0",
//         description = "A simple key-value store service"
//     )
// )]
// pub struct ApiDoc;
