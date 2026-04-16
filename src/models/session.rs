use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow, Clone)]
pub struct UserSession {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub refresh_token: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub data: Option<Value>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
}
