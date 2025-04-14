// use crate::{
//     cache::CacheClient,
//     db::DBClient,
//     error::AppError,
//     models::{CreateKv, KvPair},
// };
// use http_body_util::{BodyExt, Full};
// use hyper::{
//     Request, Response,
//     body::Bytes,
//     http::{Method, StatusCode},
// };
// use serde_json::json;
// use std::sync::Arc;
// use tracing::{info, instrument};
//
// pub struct KvService {
//     db: Arc<DBClient>,
//     cache: Arc<CacheClient>,
// }
//
// impl KvService {
//     pub fn new(db: Arc<DBClient>, cache: Arc<CacheClient>) -> Self {
//         Self { db, cache }
//     }
//
//     #[instrument(skip(self, req), fields(layer = "kv_tower"), target = "service::kv")]
//     pub async fn handle(
//         &self,
//         req: Request<hyper::body::Incoming>,
//     ) -> Result<Response<Full<Bytes>>, AppError> {
//         let method = req.method().clone();
//         let path = req.uri().path().to_string();
//
//         match (method, path.as_str()) {
//             (Method::POST, "/kv") => {
//                 let body_bytes = req.collect().await?.to_bytes();
//                 let input: CreateKv = serde_json::from_slice(&body_bytes)?;
//                 if input.key.is_empty()
//                     || input.key.len() > 50
//                     || !input.key.chars().all(|c| c.is_alphanumeric() || c == '_')
//                 {
//                     return Err(AppError::InvalidInput("Invalid key".into()));
//                 }
//                 if input.value.is_empty() || input.value.len() > 1000 {
//                     return Err(AppError::InvalidInput("Invalid value".into()));
//                 }
//                 let kv = self.db.set(input).await?;
//                 self.cache.set(&format!("kv:{}", kv.key), &kv, 300).await?;
//                 let body = serde_json::to_vec(&kv)?;
//                 Ok(Response::builder()
//                     .status(StatusCode::CREATED)
//                     .header("Content-Type", "application/json")
//                     .body(Full::new(Bytes::from(body)))?)
//             }
//             (Method::PUT, path) if path.starts_with("/kv/") => {
//                 let key = path
//                     .strip_prefix("/kv/")
//                     .ok_or_else(|| AppError::InvalidInput("Invalid path".into()))?;
//                 let body_bytes = req.collect().await?.to_bytes();
//                 let value: String = serde_json::from_slice(&body_bytes)?;
//                 if value.is_empty() || value.len() > 1000 {
//                     return Err(AppError::InvalidInput("Invalid value".into()));
//                 }
//                 let kv = self.db.update(key, &value).await?;
//                 self.cache.set(&format!("kv:{}", key), &kv, 300).await?;
//                 let body = serde_json::to_vec(&kv)?;
//                 Ok(Response::builder()
//                     .status(StatusCode::OK)
//                     .header("Content-Type", "application/json")
//                     .body(Full::new(Bytes::from(body)))?)
//             }
//             (Method::GET, path) if path.starts_with("/kv/") => {
//                 let key = path
//                     .strip_prefix("/kv/")
//                     .ok_or_else(|| AppError::InvalidInput("Invalid path".into()))?;
//                 let cache_key = format!("kv:{}", key);
//                 if let Some(kv) = self.cache.get::<KvPair>(&cache_key).await? {
//                     info!("Cache hit for key: {}", key);
//                     let body = serde_json::to_vec(&kv)?;
//                     return Ok(Response::builder()
//                         .status(StatusCode::OK)
//                         .header("Content-Type", "application/json")
//                         .body(Full::new(Bytes::from(body)))?);
//                 }
//                 let kv = self
//                     .db
//                     .get(key)
//                     .await?
//                     .ok_or_else(|| AppError::NotFound(format!("Key {} not found", key)))?;
//                 self.cache.set(&cache_key, &kv, 300).await?;
//                 let body = serde_json::to_vec(&kv)?;
//                 Ok(Response::builder()
//                     .status(StatusCode::OK)
//                     .header("Content-Type", "application/json")
//                     .body(Full::new(Bytes::from(body)))?)
//             }
//             (Method::DELETE, path) if path.starts_with("/kv/") => {
//                 let key = path
//                     .strip_prefix("/kv/")
//                     .ok_or_else(|| AppError::InvalidInput("Invalid path".into()))?;
//                 let deleted = self.db.delete(key).await?;
//                 if !deleted {
//                     return Err(AppError::NotFound(format!("Key {} not found", key)));
//                 }
//                 self.cache.delete(&format!("kv:{}", key)).await?;
//                 Ok(Response::builder()
//                     .status(StatusCode::NO_CONTENT)
//                     .body(Full::new(Bytes::new()))?)
//             }
//             _ => {
//                 let body = json!({ "error": "Method not allowed" }).to_string();
//                 Ok(Response::builder()
//                     .status(StatusCode::METHOD_NOT_ALLOWED)
//                     .header("Content-Type", "application/json")
//                     .body(Full::new(Bytes::from(body)))?)
//             }
//         }
//     }
// }
