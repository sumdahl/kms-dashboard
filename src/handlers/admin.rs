use crate::db::Db;
use crate::error::{AppError, AppResult};
use crate::middleware::auth::AdminClaims;
use crate::models::types::{AccessLevel, Resource};
use crate::models::{Role, RolePermission};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::{DateTime, Utc};
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

#[derive(Deserialize)]
pub struct ListRolesQuery {
    pub page: Option<i64>,
    pub size: Option<i64>,
    pub search: Option<String>,
}

#[derive(Serialize)]
pub struct PaginatedRoles {
    pub roles: Vec<Role>,
    pub total: i64,
    pub page: i64,
    pub size: i64,
    pub pages: i64,
}

pub async fn list_roles(
    _admin: AdminClaims,
    State(pool): State<Db>,
    Query(params): Query<ListRolesQuery>,
) -> AppResult<Json<PaginatedRoles>> {
    let page = params.page.unwrap_or(1).max(1);
    let size = params.size.unwrap_or(8).clamp(1, 100);
    let search = params.search.unwrap_or_default().trim().to_string();

    let (total, mut roles) = if search.is_empty() {
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM roles")
            .fetch_one(&pool)
            .await?;

        let roles = sqlx::query_as::<_, Role>(
            "SELECT role_id, name, description, created_at
             FROM roles ORDER BY created_at DESC
             LIMIT $1 OFFSET $2",
        )
        .bind(size)
        .bind((page - 1) * size)
        .fetch_all(&pool)
        .await?;

        (total, roles)
    } else {
        let pattern = format!("%{}%", search);

        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM roles
             WHERE name ILIKE $1 OR description ILIKE $1",
        )
        .bind(&pattern)
        .fetch_one(&pool)
        .await?;

        let roles = sqlx::query_as::<_, Role>(
            "SELECT role_id, name, description, created_at
             FROM roles
             WHERE name ILIKE $1 OR description ILIKE $1
             ORDER BY created_at DESC
             LIMIT $2 OFFSET $3",
        )
        .bind(&pattern)
        .bind(size)
        .bind((page - 1) * size)
        .fetch_all(&pool)
        .await?;

        (total, roles)
    };

    // Load permissions for the current page only
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

    let pages = if total == 0 { 1 } else { (total + size - 1) / size };

    Ok(Json(PaginatedRoles {
        roles,
        total,
        page,
        size,
        pages,
    }))
}

#[derive(Serialize)]
pub struct RolesSummary {
    pub total_roles: i64,
    pub total_permissions: i64,
    pub unique_resources: i64,
    pub write_count: i64,
    pub admin_count: i64,
}

pub async fn roles_summary(
    _admin: AdminClaims,
    State(pool): State<Db>,
) -> AppResult<Json<RolesSummary>> {
    let row = sqlx::query(
        "SELECT
            (SELECT COUNT(*) FROM roles)                AS total_roles,
            COUNT(*)                                    AS total_permissions,
            COUNT(DISTINCT resource)                    AS unique_resources,
            COUNT(*) FILTER (WHERE access_level = 'write') AS write_count,
            COUNT(*) FILTER (WHERE access_level = 'admin') AS admin_count
         FROM role_permissions",
    )
    .fetch_one(&pool)
    .await?;

    use sqlx::Row;
    Ok(Json(RolesSummary {
        total_roles: row.get("total_roles"),
        total_permissions: row.get("total_permissions"),
        unique_resources: row.get("unique_resources"),
        write_count: row.get("write_count"),
        admin_count: row.get("admin_count"),
    }))
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

pub async fn delete_role(
    _admin: AdminClaims,
    State(pool): State<Db>,
    Path(role_name): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    eprintln!("[delete_role] Attempting to delete role: {:?}", role_name);

    let result = sqlx::query("DELETE FROM roles WHERE name = $1")
        .bind(&role_name)
        .execute(&pool)
        .await?;

    eprintln!("[delete_role] rows_affected: {}", result.rows_affected());

    if result.rows_affected() == 0 {
        return Err(AppError::RoleNotFound);
    }

    Ok(Json(serde_json::json!({
        "status": "success",
        "message": format!("Role '{}' deleted.", role_name)
    })))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct RoleAssignmentRow {
    pub email: String,
    pub assigned_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

#[derive(Serialize)]
pub struct RoleDetailResponse {
    pub role: Role,
    pub assignments: Vec<RoleAssignmentRow>,
}

pub async fn get_role_detail(
    claims: Option<crate::models::Claims>,
    State(pool): State<Db>,
    Path(role_name): Path<String>,
) -> AppResult<Json<RoleDetailResponse>> {
    use sqlx::Row;

    // Require login
    let _c = claims.ok_or(AppError::Unauthorized)?;

    // Fetch role
    let mut role = sqlx::query_as::<_, Role>(
        "SELECT role_id, name, description, created_at FROM roles WHERE name = $1",
    )
    .bind(&role_name)
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::RoleNotFound)?;

    // Load permissions
    let perms =
        sqlx::query("SELECT resource, access_level FROM role_permissions WHERE role_id = $1")
            .bind(role.role_id)
            .fetch_all(&pool)
            .await?;

    role.permissions = perms
        .into_iter()
        .map(|p| {
            let res_str: String = p.get("resource");
            let acc_str: String = p.get("access_level");
            let resource = serde_json::from_value(serde_json::Value::String(res_str))
                .unwrap_or(Resource::Orders);
            let access = serde_json::from_value(serde_json::Value::String(acc_str))
                .unwrap_or(AccessLevel::Read);
            RolePermission { resource, access }
        })
        .collect();

    // Load assignments with user email
    let assignment_rows = sqlx::query(
        "SELECT u.email, ra.assigned_at, ra.expires_at,
                CASE WHEN ra.expires_at IS NULL OR ra.expires_at > NOW() THEN true ELSE false END AS is_active
         FROM role_assignments ra
         JOIN users u ON u.user_id = ra.user_id
         WHERE ra.role_id = $1
         ORDER BY ra.assigned_at DESC",
    )
    .bind(role.role_id)
    .fetch_all(&pool)
    .await?;

    let assignments = assignment_rows
        .into_iter()
        .map(|r| RoleAssignmentRow {
            email: r.get("email"),
            assigned_at: r.get("assigned_at"),
            expires_at: r.get("expires_at"),
            is_active: r.get("is_active"),
        })
        .collect();

    Ok(Json(RoleDetailResponse { role, assignments }))
}
