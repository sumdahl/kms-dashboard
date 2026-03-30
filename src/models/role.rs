use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::models::types::{AccessLevel, Resource};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Role {
    pub role_id:     String,
    pub name:        String,
    pub description: String,
    // Note: permissions are usually joined or stored in a separate table, 
    // so we don't include them in the base FromRow for Role.
    #[sqlx(skip)]
    pub permissions: Vec<RolePermission>,
    pub created_at:  DateTime<Utc>,
}

impl Role {
    pub fn new(name: &str, description: &str, permissions: Vec<RolePermission>) -> Self {
        Self {
            role_id:     Uuid::new_v4().to_string(),
            name:        name.to_string(),
            description: description.to_string(),
            permissions,
            created_at:  Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RolePermission {
    pub resource: Resource,
    pub access:   AccessLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RoleAssignment {
    pub assignment_id: String,
    pub user_id:       String,
    pub role_id:       String,
    pub assigned_by:   String,
    pub assigned_at:   DateTime<Utc>,
    pub expires_at:    Option<DateTime<Utc>>,
}

impl RoleAssignment {
    pub fn new(
        user_id:     &str,
        role_id:     &str,
        assigned_by: &str,
        expires_at:  Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            assignment_id: Uuid::new_v4().to_string(),
            user_id:       user_id.to_string(),
            role_id:       role_id.to_string(),
            assigned_by:   assigned_by.to_string(),
            assigned_at:   Utc::now(),
            expires_at,
        }
    }

    pub fn is_active(&self) -> bool {
        match self.expires_at {
            None      => true,
            Some(exp) => Utc::now() < exp,
        }
    }

    pub fn remaining_secs(&self) -> Option<i64> {
        self.expires_at
            .map(|exp| (exp - Utc::now()).num_seconds().max(0))
    }
}
