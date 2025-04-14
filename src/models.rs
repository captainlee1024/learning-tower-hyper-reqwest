use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Clone, Default, Debug, Serialize, Deserialize, FromRow, ToSchema)]
pub struct KvPair {
    pub key: String,
    pub value: String,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateKv {
    pub key: String,
    pub value: String,
}

// 接触Debug实现Display
impl std::fmt::Display for CreateKv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
