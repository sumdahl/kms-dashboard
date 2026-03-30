use axum::{extract::State, Json};
use serde::Deserialize;
use uuid::Uuid;
use crate::db::Db;
use crate::error::AppResult;
use crate::models::types::{AccessLevel, Resource};
use crate::middleware::auth::AdminClaims;

#[derive(Deserialize)]
pub struct CreateRoleRequest {
    pub name: String,
    pub description: String,
    pub permissions: Vec<PermissionRequest>,
}

#[derive(Deserialize)]
pub struct PermissionRequest {
    pub resource: Resource,
    pub access: AccessLevel,
}

pub async fn create_role(
    _admin: AdminClaims,
    State(pool): State<Db>,
    Json(payload): Json<CreateRoleRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let mut tx = pool.begin().await?;

    // 1. Insert the Role
    let role_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO roles (role_id, name, description) VALUES ($1, $2, $3)"
    )
    .bind(role_id)
    .bind(&payload.name)
    .bind(&payload.description)
    .execute(&mut *tx)
    .await?;

    // 2. Insert Permissions
    for perm in payload.permissions {
        // Convert enums to strings for database storage
        let resource_str = serde_json::to_value(&perm.resource)?
            .as_str()
            .unwrap_or("unknown")
            .to_string();
        let access_str = serde_json::to_value(&perm.access)?
            .as_str()
            .unwrap_or("read")
            .to_string();

        sqlx::query(
            "INSERT INTO role_permissions (role_id, resource, access_level) VALUES ($1, $2, $3)"
        )
        .bind(role_id)
        .bind(resource_str)
        .bind(access_str)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(Json(serde_json::json!({
        "status": "success",
        "role_id": role_id
    })))
}
