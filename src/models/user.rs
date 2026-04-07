use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub user_id: Uuid,
    pub email: String,
    pub full_name: String,
    pub password_hash: String,
    pub is_admin: bool,
    pub is_active: bool,
    pub session_version: i32,
    pub created_at: DateTime<Utc>,
}

impl User {
    pub fn new(email: &str, full_name: &str, password_hash: &str) -> Self {
        Self {
            user_id: Uuid::new_v4(),
            email: email.to_string(),
            full_name: full_name.to_string(),
            password_hash: password_hash.to_string(),
            is_admin: false,
            is_active: true,
            session_version: 0,
            created_at: Utc::now(),
        }
    }
}
