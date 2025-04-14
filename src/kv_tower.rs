use crate::{
    cache::CacheClient,
    db::DBClient,
    error::AppError,
    models::{CreateKv, KvPair},
};
use http_body_util::{BodyExt, Full};
use hyper::{
    Request, Response,
    body::Bytes,
    http::{Method, StatusCode, header},
};
use std::sync::Arc;
use tracing::{Span, info, instrument, warn};

#[derive(Clone)]
pub struct KvService {
    db: Arc<DBClient>,
    cache: Arc<CacheClient>,
}

impl KvService {
    #[allow(unused)]
    pub fn new(db: Arc<DBClient>, cache: Arc<CacheClient>) -> Self {
        Self { db, cache }
    }

    #[instrument(skip(self, req), fields(layer = "kv_tower"), target = "service::kv")]
    pub async fn handle(
        &self,
        req: Request<hyper::body::Incoming>,
    ) -> Result<Response<Full<Bytes>>, AppError> {
        let method = req.method().clone();
        let path = req.uri().path().to_string();
        tracing::info!(target: "service::kv", %method, %path, "handle request");

        match (method, path.as_str()) {
            (Method::POST, "/kv") => self.handle_set_kv(req).await,
            (Method::GET, path) if path.starts_with("/kv/") => self.handle_get_kv(path, req).await,
            (Method::PUT, path) if path.starts_with("/kv/") => {
                self.handle_update_kv(path, req).await
            }
            (Method::DELETE, path) if path.starts_with("/kv/") => {
                self.handle_delete_kv(path, req).await
            }
            _ => self.handle_not_allowed().await,
        }
    }

    #[instrument(skip(self, req), fields(payload), target = "service::kv")]
    async fn handle_set_kv(
        &self,
        req: Request<hyper::body::Incoming>,
    ) -> Result<Response<Full<Bytes>>, AppError> {
        info!("📥 incoming set key-value request");

        // 解析请求体
        let body_bytes = req.collect().await?.to_bytes();
        let input: CreateKv = serde_json::from_slice(&body_bytes)
            .map_err(|e| AppError::InvalidInput(format!("Invalid JSON: {}", e)))?;
        Span::current().record("payload", format!("{}", input));

        // 验证输入
        if input.key.is_empty()
            || input.key.len() > 50
            || !input.key.chars().all(|c| c.is_alphanumeric() || c == '_')
        {
            warn!("⚠️ invalid key: {}", input.key);
            return Err(AppError::InvalidInput("Invalid key".into()));
        }
        if input.value.is_empty() || input.value.len() > 1000 {
            warn!("⚠️ invalid value: {}", input.value);
            return Err(AppError::InvalidInput("Invalid value".into()));
        }

        // 更新数据库
        info!("✏️ update db");
        let kv = self.db.set(input).await?;

        // TODO: 缓存脏读问题待优化（可改为删除缓存）
        info!("✏️ update cache");
        self.cache.set(&format!("kv:{}", kv.key), &kv, 300).await?;

        info!("📦 set key-value successful");
        let body = serde_json::to_vec(&kv)?;
        Ok(Response::builder()
            .status(StatusCode::CREATED)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Full::new(Bytes::from(body)))
            .map_err(|e| {
                AppError::InvalidInput(format!(
                    "Failed to construct response body:{}",
                    e.to_string()
                ))
            })?)
    }

    #[instrument(skip(self, _req), fields(key), target = "service::kv")]
    async fn handle_get_kv(
        &self,
        path: &str,
        _req: Request<hyper::body::Incoming>,
    ) -> Result<Response<Full<Bytes>>, AppError> {
        let key = path
            .strip_prefix("/kv/")
            .ok_or_else(|| AppError::InvalidInput("Invalid path".into()))?;
        Span::current().record("key", key);
        info!("📥 incoming get request");

        let cache_key = format!("kv:{}", key);
        if let Some(kv) = self.cache.get::<KvPair>(&cache_key).await? {
            info!("✅ cache hit");
            let body = serde_json::to_vec(&kv)?;
            return Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Full::new(Bytes::from(body)))
                .map_err(|e| {
                    AppError::InvalidInput(format!(
                        "Failed to construct response body:{}",
                        e.to_string()
                    ))
                })?);
        }
        info!("⚠️ cache miss");

        let kv = self.db.get(key).await?.ok_or_else(|| {
            warn!("⚠️ key not found in db");
            AppError::NotFound(format!("Key {} not found", key))
        })?;

        info!("✏️ update cache");
        self.cache.set(&cache_key, &kv, 300).await?;
        info!("📦 fetched from db");

        let body = serde_json::to_vec(&kv)?;
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Full::new(Bytes::from(body)))
            .map_err(|e| {
                AppError::InvalidInput(format!(
                    "Failed to construct response body:{}",
                    e.to_string()
                ))
            })?)
    }

    #[instrument(skip(self, req), fields(key, value), target = "service::kv")]
    async fn handle_update_kv(
        &self,
        path: &str,
        req: Request<hyper::body::Incoming>,
    ) -> Result<Response<Full<Bytes>>, AppError> {
        let key = path
            .strip_prefix("/kv/")
            .ok_or_else(|| AppError::InvalidInput("Invalid path".into()))?;
        Span::current().record("key", key);
        info!("📥 incoming update request");

        // 解析 JSON 字符串
        let body_bytes = req.collect().await?.to_bytes();
        let value: String = serde_json::from_slice(&body_bytes)
            .map_err(|e| AppError::InvalidInput(format!("Invalid JSON string: {}", e)))?;
        Span::current().record("value", &value);

        // 验证输入
        if value.is_empty() || value.len() > 1000 {
            warn!("⚠️ invalid value: {}", value);
            return Err(AppError::InvalidInput("Invalid value".into()));
        }

        // 更新数据库
        info!("✏️ update db");
        let kv = self.db.update(&key, &value).await?;

        // TODO: 缓存脏读问题待优化（可改为删除缓存）
        info!("✏️ update cache");
        self.cache.set(&format!("kv:{}", key), &kv, 300).await?;

        info!("📦 update successful");
        let body = serde_json::to_vec(&kv)?;
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Full::new(Bytes::from(body)))
            .map_err(|e| {
                AppError::InvalidInput(format!(
                    "Failed to construct response body:{}",
                    e.to_string()
                ))
            })?)
    }

    #[instrument(skip(self, _req), fields(key), target = "service::kv")]
    async fn handle_delete_kv(
        &self,
        path: &str,
        _req: Request<hyper::body::Incoming>,
    ) -> Result<Response<Full<Bytes>>, AppError> {
        let key = path
            .strip_prefix("/kv/")
            .ok_or_else(|| AppError::InvalidInput("Invalid path".into()))?;
        Span::current().record("key", key);
        info!("📥 incoming delete request");

        info!("🗑️ delete from db");
        let deleted = self.db.delete(key).await?;
        if !deleted {
            warn!("⚠️ key not found in db");
            return Err(AppError::NotFound(format!("Key {} not found", key)));
        }

        info!("🗑️ delete from cache");
        self.cache.delete(&format!("kv:{}", key)).await?;

        info!("📦 delete successful");
        Ok(Response::builder()
            .status(StatusCode::NO_CONTENT)
            .body(Full::new(Bytes::new()))
            .map_err(|e| {
                AppError::InvalidInput(format!(
                    "Failed to construct response body:{}",
                    e.to_string()
                ))
            })?)
    }

    async fn handle_not_allowed(&self) -> Result<Response<Full<Bytes>>, AppError> {
        warn!("⚠️ method not allowed");
        Err(AppError::InvalidInput("Method not allowed".into()))
    }
}

#[allow(dead_code)]
pub async fn serve_req(
    svc: &KvService,
    req: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, AppError> {
    svc.handle(req).await
}
