use crate::{
    error::AppError,
    models::{CreateKv, KvPair},
};
use sqlx::PgPool;
use tracing::instrument;

pub struct DBClient {
    pool: PgPool,
}

impl DBClient {
    pub async fn new(database_url: &str) -> Result<Self, AppError> {
        let pool = PgPool::connect(database_url).await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(Self { pool })
    }

    #[instrument(skip(self))]
    pub async fn set(&self, input: CreateKv) -> Result<KvPair, AppError> {
        // let kv = sqlx::query_as!(
        //     KvPair,
        //     r#"
        //     INSERT INTO kv_store (key, value)
        //     VALUES ($1, $2)
        //     ON CONFLICT (key)
        //     DO NOTHING
        //     RETURNING key, value, updated_at
        //     "#,
        //     input.key,
        //     input.value
        // )
        // .fetch_one(&self.pool)
        // .await
        // .map_err(|_| AppError::InvalidInput("Key already exists".to_string()))?;
        // Ok(kv)

        tracing::info!(target: "db::kv", "set {:?} to db", input);
        let kv = sqlx::query_as::<_, KvPair>(
            r#"
            INSERT INTO kv_store (key, value)
            VALUES ($1, $2)
            ON CONFLICT (key)
            DO NOTHING
            RETURNING key, value, updated_at
            "#,
        )
        .bind(input.key)
        .bind(input.value)
        .fetch_one(&self.pool)
        .await?;
        // .map_err(|_| AppError::InvalidInput("Key already exists".to_string()))?;

        tracing::info!(target: "db::kv", "set success!");
        Ok(kv)
    }

    #[instrument(skip(self))]
    pub async fn update(&self, key: &str, value: &str) -> Result<KvPair, AppError> {
        // let kv = sqlx::query_as!(
        //     KvPair,
        //     r#"
        //     UPDATE kv_store
        //     SET value = $2, updated_at = CURRENT_TIMESTAMP
        //     WHERE key = $1
        //     RETURNING key, value, updated_at
        //     "#,
        //     key,
        //     value
        // )
        // .fetch_one(&self.pool)
        // .await
        // .map_err(|_| AppError::NotFound(format!("Key {} not found", key)))?;
        // Ok(kv)

        tracing::info!(target: "db::kv", "update db, {} to {}", key, value);
        let kv = sqlx::query_as::<_, KvPair>(
            r#"
            UPDATE kv_store
            SET value = $2, updated_at = CURRENT_TIMESTAMP
            WHERE key = $1
            RETURNING key, value, updated_at
            "#,
        )
        .bind(key)
        .bind(value)
        .fetch_one(&self.pool)
        .await?;
        // .map_err(|_| AppError::NotFound(format!("Key {} not found", key)))?;

        tracing::info!(target: "db::kv", "update db success");
        Ok(kv)
    }

    #[instrument(skip(self))]
    pub async fn get(&self, key: &str) -> Result<Option<KvPair>, AppError> {
        // let kv = sqlx::query_as!(
        //     KvPair,
        //     r#"
        //     SELECT key, value, updated_at
        //     FROM kv_store
        //     WHERE key = $1
        //     "#,
        //     key
        // )
        // .fetch_optional(&self.pool)
        // .await?;
        // Ok(kv)

        tracing::info!(target: "db::kv", "get {} from db", key);
        let kv = sqlx::query_as::<_, KvPair>(
            r#"
            SELECT key, value, updated_at
            FROM kv_store
            WHERE key = $1
            "#,
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;

        tracing::info!(target: "db::kv", "get {} from db success", key);
        Ok(kv)
    }

    #[instrument(skip(self))]
    pub async fn delete(&self, key: &str) -> Result<bool, AppError> {
        // let result = sqlx::query!(
        //     r#"
        //     DELETE FROM kv_store
        //     WHERE key = $1
        //     "#,
        //     key
        // )
        // .execute(&self.pool)
        // .await?;
        // Ok(result.rows_affected() > 0)

        tracing::info!(target: "db::kv", "delete {} from db", key);
        let result = sqlx::query(
            r#"
            DELETE FROM kv_store
            WHERE key = $1
            "#,
        )
        .bind(key)
        .execute(&self.pool)
        .await?;
        tracing::info!(target: "db::kv", "delete {} from db success", key);
        Ok(result.rows_affected() > 0)
    }
}
