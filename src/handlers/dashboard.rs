use crate::db::Db;
use crate::error::AppResult;
use crate::middleware::rbac::Permissions;
use crate::models::types::{AccessLevel, Resource};
use crate::models::Claims;
use axum::{extract::State, Json};
use serde::Serialize;
use uuid::Uuid;

pub async fn inventory_status(
    perms: Permissions, // Our new RBAC lock
) -> AppResult<Json<serde_json::Value>> {
    // Check if user has "inventory" with at least "read" access
    perms.require(Resource::Inventory, AccessLevel::Read)?;

    Ok(Json(serde_json::json!({
        "status": "online",
        "items_count": 150,
        "message": "You have active access to inventory data."
    })))
}

// ── User's own roles ────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct MyPermission {
    pub resource: String,
    pub access: String,
}

#[derive(Serialize)]
pub struct MyRole {
    pub name: String,
    pub description: String,
    pub permissions: Vec<MyPermission>,
    pub expires_at: Option<String>,
}

/// Load active role assignments (and permissions) for the authenticated user.
pub async fn load_my_roles(claims: &Claims, pool: &Db) -> AppResult<Vec<MyRole>> {
    use sqlx::Row;

    let user_id = claims
        .sub
        .parse::<Uuid>()
        .map_err(|_| crate::error::AppError::Unauthorized)?;

    let rows = sqlx::query(
        r#"
        SELECT r.role_id, r.name, r.description, ra.expires_at
        FROM role_assignments ra
        JOIN roles r ON ra.role_id = r.role_id
        WHERE ra.user_id = $1
          AND (ra.expires_at IS NULL OR ra.expires_at > NOW())
        ORDER BY r.name
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let mut result: Vec<MyRole> = Vec::new();

    for row in rows {
        let role_id: Uuid = row.get("role_id");
        let name: String = row.get("name");
        let description: String = row.get("description");
        let expires_at: Option<chrono::DateTime<chrono::Utc>> = row.get("expires_at");

        let perm_rows =
            sqlx::query("SELECT resource, access_level FROM role_permissions WHERE role_id = $1")
                .bind(role_id)
                .fetch_all(pool)
                .await?;

        let permissions = perm_rows
            .iter()
            .map(|p| MyPermission {
                resource: p.get::<String, _>("resource"),
                access: p.get::<String, _>("access_level"),
            })
            .collect();

        result.push(MyRole {
            name,
            description,
            permissions,
            expires_at: expires_at.map(|e| e.to_rfc3339()),
        });
    }

    Ok(result)
}

pub async fn my_roles(claims: Claims, State(pool): State<Db>) -> AppResult<Json<Vec<MyRole>>> {
    let roles = load_my_roles(&claims, &pool).await?;
    Ok(Json(roles))
}
