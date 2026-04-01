use crate::db::Db;
use crate::error::{AppError, AppResult};
use crate::middleware::auth::AdminClaims;
use crate::models::types::{AccessLevel, Resource};
use crate::models::{Role, RolePermission};
use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UserSummary {
    pub user_id: Uuid,
    pub email: String,
    pub full_name: String,
    pub is_admin: bool,
}

pub async fn list_users(
    _admin: AdminClaims,
    State(pool): State<Db>,
) -> AppResult<Json<Vec<UserSummary>>> {
    let users = sqlx::query_as::<_, UserSummary>(
        "SELECT user_id, email, full_name, is_admin FROM users ORDER BY created_at DESC",
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(users))
}

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

#[derive(Deserialize)]
pub struct AssignRoleRequest {
    pub email: String,
    pub role_name: String,
    pub duration_secs: Option<i64>,
}

pub async fn list_roles(_admin: AdminClaims, State(pool): State<Db>) -> AppResult<Json<Vec<Role>>> {
    let mut roles = sqlx::query_as::<_, Role>(
        "SELECT role_id, name, description, created_at FROM roles ORDER BY created_at DESC",
    )
    .fetch_all(&pool)
    .await?;

    for role in &mut roles {
        let perms =
            sqlx::query("SELECT resource, access_level FROM role_permissions WHERE role_id = $1")
                .bind(role.role_id)
                .fetch_all(&pool)
                .await?;

        role.permissions = perms
            .into_iter()
            .map(|p| {
                use sqlx::Row;
                let res_str: String = p.get("resource");
                let acc_str: String = p.get("access_level");

                let resource = serde_json::from_value(serde_json::Value::String(res_str))
                    .unwrap_or(Resource::Orders);
                let access = serde_json::from_value(serde_json::Value::String(acc_str))
                    .unwrap_or(AccessLevel::Read);

                RolePermission { resource, access }
            })
            .collect();
    }

    Ok(Json(roles))
}

pub async fn create_role(
    _admin: AdminClaims,
    State(pool): State<Db>,
    Json(payload): Json<CreateRoleRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let mut tx = pool.begin().await?;
    let role_id = Uuid::new_v4();

    let insert_result =
        sqlx::query("INSERT INTO roles (role_id, name, description) VALUES ($1, $2, $3)")
            .bind(role_id)
            .bind(&payload.name)
            .bind(&payload.description)
            .execute(&mut *tx)
            .await; //

    match insert_result {
        Ok(_) => {}
        Err(sqlx::Error::Database(db_err)) if db_err.code().as_deref() == Some("23505") => {
            return Err(AppError::Conflict(format!(
                "A role named '{}' already exists.",
                payload.name
            )));
        }
        Err(e) => return Err(e.into()),
    }

    for perm in payload.permissions {
        let resource_str = serde_json::to_value(&perm.resource)?
            .as_str()
            .unwrap_or("unknown")
            .to_string();
        let access_str = serde_json::to_value(&perm.access)?
            .as_str()
            .unwrap_or("read")
            .to_string();

        sqlx::query(
            "INSERT INTO role_permissions (role_id, resource, access_level) VALUES ($1, $2, $3)",
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
pub async fn assign_role(
    _admin: AdminClaims,
    State(pool): State<Db>,
    Json(payload): Json<AssignRoleRequest>,
) -> AppResult<Json<serde_json::Value>> {
    use sqlx::Row;

    // 1. Find user_id by email
    let user_row = sqlx::query("SELECT user_id FROM users WHERE email = $1")
        .bind(&payload.email)
        .fetch_optional(&pool)
        .await?
        .ok_or(AppError::UserNotFound)?;
    let user_id: Uuid = user_row.get("user_id");

    // 2. Find role_id by name
    let role_row = sqlx::query("SELECT role_id FROM roles WHERE name = $1")
        .bind(&payload.role_name)
        .fetch_optional(&pool)
        .await?
        .ok_or(AppError::RoleNotFound)?;
    let role_id: Uuid = role_row.get("role_id");

    // 3. Calculate expiry
    let expires_at = payload
        .duration_secs
        .map(|secs| chrono::Utc::now() + chrono::Duration::seconds(secs));

    // 4. Create or update assignment
    sqlx::query(
        "INSERT INTO role_assignments (user_id, role_id, assigned_by, expires_at)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (user_id, role_id)
         DO UPDATE SET expires_at = EXCLUDED.expires_at, assigned_at = NOW()",
    )
    .bind(user_id)
    .bind(role_id)
    .bind(_admin.0.sub.parse::<Uuid>().ok())
    .bind(expires_at)
    .execute(&pool)
    .await?;

    Ok(Json(serde_json::json!({
        "status": "success",
        "message": format!("Role '{}' assigned to {}", payload.role_name, payload.email),
        "expires_at": expires_at
    })))
}
