use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow, Clone)]
pub struct AppSetting {
    pub key: String,
    pub value: String,
    pub description: Option<String>,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthStrategy {
    Jwt,
    Session,
    Hybrid,
}

impl AuthStrategy {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuthStrategy::Jwt => "jwt",
            AuthStrategy::Session => "session",
            AuthStrategy::Hybrid => "hybrid",
        }
    }
}

impl ToString for AuthStrategy {
    fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}

impl From<String> for AuthStrategy {
    fn from(s: String) -> Self {
        match s.as_str() {
            "session" => AuthStrategy::Session,
            "hybrid" => AuthStrategy::Hybrid,
            _ => AuthStrategy::Jwt,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub auth_strategy: AuthStrategy,
    pub jwt_access_ttl_minutes: i32,
    pub session_refresh_ttl_hours: i32,
    pub max_concurrent_sessions: i32,
    pub logout_on_browser_close: bool,
    pub force_logout_on_password_change: bool,
    pub ip_restriction_enabled: bool,
    pub remember_me_extension_hours: i32,
}
