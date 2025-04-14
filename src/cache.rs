use crate::error::AppError;
use redis::{AsyncCommands, Client};
use serde::{Serialize, de::DeserializeOwned};
use tracing::instrument;

pub struct CacheClient {
    client: Client,
}

impl CacheClient {
    pub async fn new(redis_url: &str) -> Result<Self, AppError> {
        let client = Client::open(redis_url)?;
        Ok(Self { client })
    }

    #[instrument(skip(self, value))]
    pub async fn set<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl_secs: u64,
    ) -> Result<(), AppError> {
        // TODO: record the value
        tracing::info!(target: "redis::kv", "set {} to redis", key);

        // let mut con = self.client.get_async_connection().await?;
        let mut con = self.client.get_multiplexed_async_connection().await?;
        let serialized_value = serde_json::to_string(value)?;
        // con.set(key, serialized_value).await?;
        con.set_ex::<_, _, ()>(key, serialized_value, ttl_secs)
            .await?;

        tracing::info!(target: "redis::kv", "set {} to redis success", key);
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, AppError> {
        tracing::info!(target: "redis::kv", "get {} from redis", key);

        // let mut con = self.client.get_async_connection().await?;
        let mut con = self.client.get_multiplexed_async_connection().await?;
        let result: Option<String> = con.get(key).await?;

        tracing::info!(target: "redis::kv", "get {} from redis success", key);
        match result {
            Some(value) => {
                let deserialized_value: T = serde_json::from_str(&value)?;
                Ok(Some(deserialized_value))
            }
            None => Ok(None),
        }
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, key: &str) -> Result<(), AppError> {
        tracing::info!(target: "redis::kv", "delete {} from redis", key);

        // let mut con = self.client.get_async_connection().await?;
        let mut con = self.client.get_multiplexed_async_connection().await?;
        con.del::<_, ()>(key).await?;

        tracing::info!(target: "redis::kv", "delete {} from redis success", key);
        Ok(())
    }
}
