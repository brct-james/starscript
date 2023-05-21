use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, sqlx::FromRow)]
pub struct Agent {
    pub token: String,
    pub account_id: String,
    pub symbol: String,
    pub headquarters: String,
    pub credits: i64,
    pub registered_timestamp: i64,
    pub last_updated_timestamp: i64,
}
