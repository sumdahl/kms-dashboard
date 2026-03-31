use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::models::types::{AccessLevel, Resource};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Role {
    pub role_id:     Uuid,
    pub name:        String,
    pub description: String,
    #[sqlx(skip)]
    pub permissions: Vec<RolePermission>,
    pub created_at:  DateTime<Utc>,
}

impl Role {
    pub fn new(name: &str, description: &str, permissions: Vec<RolePermission>) -> Self {
        Self {
            role_id:     Uuid::new_v4(),
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
    pub assignment_id: Uuid,
    pub user_id:       Uuid,
    pub role_id:       Uuid,
    pub assigned_by:   Option<Uuid>,
    pub assigned_at:   DateTime<Utc>,
    pub expires_at:    Option<DateTime<Utc>>,
}

impl RoleAssignment {
    pub fn new(
        user_id:     Uuid,
        role_id:     Uuid,
        assigned_by: Option<Uuid>,
        expires_at:  Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            assignment_id: Uuid::new_v4(),
            user_id,
            role_id,
            assigned_by,
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
