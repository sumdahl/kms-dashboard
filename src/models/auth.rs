use crate::models::types::{AccessLevel, Resource};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub is_admin: bool,
    pub sv: i32,
    pub exp: usize,
    pub iat: usize,
    pub jti: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResolvedPermission {
    pub resource: Resource,
    pub access: AccessLevel,
    pub expires_at: Option<DateTime<Utc>>,
    pub granted_by_roles: Vec<String>,
}
