use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::models::types::{AccessLevel, Resource};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub:      String,
    pub email:    String,
    pub is_admin: bool,
    pub exp:      usize,
    pub iat:      usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResolvedPermission {
    pub resource:         Resource,
    pub access:           AccessLevel,
    pub expires_at:       Option<DateTime<Utc>>,
    pub granted_by_roles: Vec<String>,
}
