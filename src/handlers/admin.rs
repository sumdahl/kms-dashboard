use axum::{
    extract::{Json, Path, State},
    http::header,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::db::Db;
use crate::error::{AppResult, AppError};
use crate::middleware::auth::{AdminClaims, PageClaims};
use crate::models::{Role, RolePermission};
use crate::models::types::{AccessLevel, Resource};

// ── Form types ───────────────────────────────────────────────────

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
pub struct CreateRoleForm {
    pub name: String,
    pub description: String,
}

#[derive(Deserialize)]
pub struct AssignRoleForm {
    pub email: String,
    pub role_name: String,
    pub duration_hours: Option<i64>,
}

// ── Template structs ─────────────────────────────────────────────

#[derive(askama::Template)]
#[template(path = "admin/roles.html")]
pub struct RolesTemplate {
    pub css_version: &'static str,
    pub user_email: String,
    pub is_admin: bool,
    pub sidebar_pinned: bool,
    pub show_banner: bool,
    pub active_tab: &'static str,
    pub roles: Vec<Role>,
}

#[derive(askama::Template)]
#[template(path = "admin/assignments.html")]
pub struct AssignmentsTemplate {
    pub css_version: &'static str,
    pub user_email: String,
    pub is_admin: bool,
    pub sidebar_pinned: bool,
    pub show_banner: bool,
    pub active_tab: &'static str,
    pub assignments: Vec<AssignmentView>,
    pub roles: Vec<Role>,
}

#[derive(askama::Template)]
#[template(path = "admin/partials/roles_list.html")]
pub struct RolesListPartial {
    pub roles: Vec<Role>,
}

#[derive(askama::Template)]
#[template(path = "admin/partials/assignments_list.html")]
pub struct AssignmentsListPartial {
    pub assignments: Vec<AssignmentView>,
}

pub struct AssignmentView {
    pub assignment_id: Uuid,
    pub user_email: String,
    pub role_name: String,
    pub assigned_at: String,
    pub expires_at: Option<String>,
    pub is_expired: bool,
}

// ── Helper: fetch roles with permissions ─────────────────────────

async fn fetch_roles_with_perms(pool: &Db) -> AppResult<Vec<Role>> {
    let mut roles = sqlx::query_as::<_, Role>(
        "SELECT role_id, name, description, created_at FROM roles ORDER BY created_at DESC"
    )
    .fetch_all(pool)
    .await?;

    for role in &mut roles {
        let perms = sqlx::query(
            "SELECT resource, access_level FROM role_permissions WHERE role_id = $1"
        )
        .bind(role.role_id)
        .fetch_all(pool)
        .await?;

        role.permissions = perms
            .into_iter()
            .map(|p| {
                use sqlx::Row;
                let res_str: String = p.get("resource");
                let acc_str: String = p.get("access_level");

                let resource = serde_json::from_value(serde_json::Value::String(res_str))
                    .unwrap_or(Resource::Orders);
                let access = serde_json::from_value(serde_json::Value::Value::String(acc_str))
                    .unwrap_or(AccessLevel::Read);

                RolePermission { resource, access }
            })
            .collect();
    }

    Ok(roles)
}

// ── Page handlers ────────────────────────────────────────────────

pub async fn roles_page(
    PageClaims(claims): PageClaims,
    State(pool): State<Db>,
) -> AppResult<impl IntoResponse> {
    let roles = fetch_roles_with_perms(&pool).await?;

    Ok(RolesTemplate {
        css_version: env!("CSS_VERSION"),
        user_email: claims.email,
        is_admin: claims.is_admin,
        sidebar_pinned: false,
        show_banner: false,
        active_tab: "roles",
        roles,
    })
}

pub async fn assignments_page(
    PageClaims(claims): PageClaims,
    State(pool): State<Db>,
) -> AppResult<impl IntoResponse> {
    use sqlx::Row;

    let rows = sqlx::query(
        r#"
        SELECT
            ra.assignment_id,
            u.email as user_email,
            r.name as role_name,
            ra.assigned_at,
            ra.expires_at
        FROM role_assignments ra
        JOIN users u ON ra.user_id = u.user_id
        JOIN roles r ON ra.role_id = r.role_id
        ORDER BY ra.assigned_at DESC
        "#
    )
    .fetch_all(&pool)
    .await?;

    let assignments: Vec<AssignmentView> = rows
        .into_iter()
        .map(|row| {
            let assigned_at: chrono::DateTime<chrono::Utc> = row.get("assigned_at");
            let expires_at: Option<chrono::DateTime<chrono::Utc>> = row.get("expires_at");
            let is_expired = expires_at
                .map(|exp| chrono::Utc::now() > exp)
                .unwrap_or(false);

            AssignmentView {
                assignment_id: row.get("assignment_id"),
                user_email: row.get("user_email"),
                role_name: row.get("role_name"),
                assigned_at: assigned_at.format("%Y-%m-%d %H:%M UTC").to_string(),
                expires_at: expires_at.map(|e| e.format("%Y-%m-%d %H:%M UTC").to_string()),
                is_expired,
            }
        })
        .collect();

    let roles = fetch_roles_with_perms(&pool).await?;

    Ok(AssignmentsTemplate {
        css_version: env!("CSS_VERSION"),
        user_email: claims.email,
        is_admin: claims.is_admin,
        sidebar_pinned: false,
        show_banner: false,
        active_tab: "assignments",
        assignments,
        roles,
    })
}

// ── HTMX partial handlers ────────────────────────────────────────

pub async fn roles_list(
    _admin: AdminClaims,
    State(pool): State<Db>,
) -> AppResult<impl IntoResponse> {
    let roles = fetch_roles_with_perms(&pool).await?;
    Ok(RolesListPartial { roles })
}

pub async fn create_role_htmx(
    _admin: AdminClaims,
    State(pool): State<Db>,
    Json(form): Json<CreateRoleForm>,
) -> AppResult<Response> {
    let mut tx = pool.begin().await?;

    let role_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO roles (role_id, name, description) VALUES ($1, $2, $3)"
    )
    .bind(role_id)
    .bind(&form.name)
    .bind(&form.description)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    // Return updated roles list
    let roles = fetch_roles_with_perms(&pool).await?;
    let partial = RolesListPartial { roles };
    let html = partial.render().map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Response::builder()
        .status(200)
        .header(header::CONTENT_TYPE, "text/html")
        .body(axum::body::Body::from(html))
        .unwrap())
}

pub async fn delete_role(
    _admin: AdminClaims,
    State(pool): State<Db>,
    Path(role_id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    sqlx::query("DELETE FROM roles WHERE role_id = $1")
        .bind(role_id)
        .execute(&pool)
        .await?;

    let roles = fetch_roles_with_perms(&pool).await?;
    Ok(RolesListPartial { roles })
}

pub async fn assignments_list(
    _admin: AdminClaims,
    State(pool): State<Db>,
) -> AppResult<impl IntoResponse> {
    use sqlx::Row;

    let rows = sqlx::query(
        r#"
        SELECT
            ra.assignment_id,
            u.email as user_email,
            r.name as role_name,
            ra.assigned_at,
            ra.expires_at
        FROM role_assignments ra
        JOIN users u ON ra.user_id = u.user_id
        JOIN roles r ON ra.role_id = r.role_id
        ORDER BY ra.assigned_at DESC
        "#
    )
    .fetch_all(&pool)
    .await?;

    let assignments: Vec<AssignmentView> = rows
        .into_iter()
        .map(|row| {
            let assigned_at: chrono::DateTime<chrono::Utc> = row.get("assigned_at");
            let expires_at: Option<chrono::DateTime<chrono::Utc>> = row.get("expires_at");
            let is_expired = expires_at
                .map(|exp| chrono::Utc::now() > exp)
                .unwrap_or(false);

            AssignmentView {
                assignment_id: row.get("assignment_id"),
                user_email: row.get("user_email"),
                role_name: row.get("role_name"),
                assigned_at: assigned_at.format("%Y-%m-%d %H:%M UTC").to_string(),
                expires_at: expires_at.map(|e| e.format("%Y-%m-%d %H:%M UTC").to_string()),
                is_expired,
            }
        })
        .collect();

    Ok(AssignmentsListPartial { assignments })
}

pub async fn assign_role_htmx(
    _admin: AdminClaims,
    State(pool): State<Db>,
    Json(form): Json<AssignRoleForm>,
) -> AppResult<Response> {
    use sqlx::Row;

    // 1. Find user_id by email
    let user_row = sqlx::query("SELECT user_id FROM users WHERE email = $1")
        .bind(&form.email)
        .fetch_optional(&pool)
        .await?
        .ok_or(AppError::UserNotFound)?;
    let user_id: Uuid = user_row.get("user_id");

    // 2. Find role_id by name
    let role_row = sqlx::query("SELECT role_id FROM roles WHERE name = $1")
        .bind(&form.role_name)
        .fetch_optional(&pool)
        .await?
        .ok_or(AppError::RoleNotFound)?;
    let role_id: Uuid = role_row.get("role_id");

    // 3. Calculate expiry
    let expires_at = form.duration_hours.map(|hours| {
        chrono::Utc::now() + chrono::Duration::hours(hours)
    });

    // 4. Get admin user id from claims
    let admin_user_id: Option<Uuid> = _admin.0.sub.parse::<Uuid>().ok();

    // 5. Create or update assignment
    sqlx::query(
        "INSERT INTO role_assignments (user_id, role_id, assigned_by, expires_at)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (user_id, role_id)
         DO UPDATE SET expires_at = EXCLUDED.expires_at, assigned_at = NOW()"
    )
    .bind(user_id)
    .bind(role_id)
    .bind(admin_user_id)
    .bind(expires_at)
    .execute(&pool)
    .await?;

    // Return updated assignments list
    let rows = sqlx::query(
        r#"
        SELECT
            ra.assignment_id,
            u.email as user_email,
            r.name as role_name,
            ra.assigned_at,
            ra.expires_at
        FROM role_assignments ra
        JOIN users u ON ra.user_id = u.user_id
        JOIN roles r ON ra.role_id = r.role_id
        ORDER BY ra.assigned_at DESC
        "#
    )
    .fetch_all(&pool)
    .await?;

    let assignments: Vec<AssignmentView> = rows
        .into_iter()
        .map(|row| {
            let assigned_at: chrono::DateTime<chrono::Utc> = row.get("assigned_at");
            let expires_at: Option<chrono::DateTime<chrono::Utc>> = row.get("expires_at");
            let is_expired = expires_at
                .map(|exp| chrono::Utc::now() > exp)
                .unwrap_or(false);

            AssignmentView {
                assignment_id: row.get("assignment_id"),
                user_email: row.get("user_email"),
                role_name: row.get("role_name"),
                assigned_at: assigned_at.format("%Y-%m-%d %H:%M UTC").to_string(),
                expires_at: expires_at.map(|e| e.format("%Y-%m-%d %H:%M UTC").to_string()),
                is_expired,
            }
        })
        .collect();

    let partial = AssignmentsListPartial { assignments };
    let html = partial.render().map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Response::builder()
        .status(200)
        .header(header::CONTENT_TYPE, "text/html")
        .body(axum::body::Body::from(html))
        .unwrap())
}

pub async fn revoke_assignment(
    _admin: AdminClaims,
    State(pool): State<Db>,
    Path(assignment_id): Path<Uuid>,
) -> AppResult<Response> {
    sqlx::query("DELETE FROM role_assignments WHERE assignment_id = $1")
        .bind(assignment_id)
        .execute(&pool)
        .await?;

    // Return updated assignments list
    let rows = sqlx::query(
        r#"
        SELECT
            ra.assignment_id,
            u.email as user_email,
            r.name as role_name,
            ra.assigned_at,
            ra.expires_at
        FROM role_assignments ra
        JOIN users u ON ra.user_id = u.user_id
        JOIN roles r ON ra.role_id = r.role_id
        ORDER BY ra.assigned_at DESC
        "#
    )
    .fetch_all(&pool)
    .await?;

    let assignments: Vec<AssignmentView> = rows
        .into_iter()
        .map(|row| {
            use sqlx::Row;
            let assigned_at: chrono::DateTime<chrono::Utc> = row.get("assigned_at");
            let expires_at: Option<chrono::DateTime<chrono::Utc>> = row.get("expires_at");
            let is_expired = expires_at
                .map(|exp| chrono::Utc::now() > exp)
                .unwrap_or(false);

            AssignmentView {
                assignment_id: row.get("assignment_id"),
                user_email: row.get("user_email"),
                role_name: row.get("role_name"),
                assigned_at: assigned_at.format("%Y-%m-%d %H:%M UTC").to_string(),
                expires_at: expires_at.map(|e| e.format("%Y-%m-%d %H:%M UTC").to_string()),
                is_expired,
            }
        })
        .collect();

    let partial = AssignmentsListPartial { assignments };
    let html = partial.render().map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Response::builder()
        .status(200)
        .header(header::CONTENT_TYPE, "text/html")
        .body(axum::body::Body::from(html))
        .unwrap())
}

// ── Original JSON API handlers (kept for API compatibility) ──────

pub async fn list_roles(
    _admin: AdminClaims,
    State(pool): State<Db>,
) -> AppResult<axum::Json<Vec<Role>>> {
    let roles = fetch_roles_with_perms(&pool).await?;
    Ok(axum::Json(roles))
}

pub async fn create_role(
    _admin: AdminClaims,
    State(pool): State<Db>,
    axum::Json(payload): axum::Json<CreateRoleRequest>,
) -> AppResult<axum::Json<serde_json::Value>> {
    let mut tx = pool.begin().await?;

    let role_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO roles (role_id, name, description) VALUES ($1, $2, $3)"
    )
    .bind(role_id)
    .bind(&payload.name)
    .bind(&payload.description)
    .execute(&mut *tx)
    .await?;

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
            "INSERT INTO role_permissions (role_id, resource, access_level) VALUES ($1, $2, $3)"
        )
        .bind(role_id)
        .bind(resource_str)
        .bind(access_str)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(axum::Json(serde_json::json!({
        "status": "success",
        "role_id": role_id
    })))
}

pub async fn assign_role(
    _admin: AdminClaims,
    State(pool): State<Db>,
    axum::Json(payload): axum::Json<AssignRoleRequest>,
) -> AppResult<axum::Json<serde_json::Value>> {
    use sqlx::Row;

    let user_row = sqlx::query("SELECT user_id FROM users WHERE email = $1")
        .bind(&payload.email)
        .fetch_optional(&pool)
        .await?
        .ok_or(AppError::UserNotFound)?;
    let user_id: Uuid = user_row.get("user_id");

    let role_row = sqlx::query("SELECT role_id FROM roles WHERE name = $1")
        .bind(&payload.role_name)
        .fetch_optional(&pool)
        .await?
        .ok_or(AppError::RoleNotFound)?;
    let role_id: Uuid = role_row.get("role_id");

    let expires_at = payload.duration_secs.map(|secs| {
        chrono::Utc::now() + chrono::Duration::seconds(secs)
    });

    sqlx::query(
        "INSERT INTO role_assignments (user_id, role_id, assigned_by, expires_at)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (user_id, role_id)
         DO UPDATE SET expires_at = EXCLUDED.expires_at, assigned_at = NOW()"
    )
    .bind(user_id)
    .bind(role_id)
    .bind(_admin.0.sub.parse::<Uuid>().ok())
    .bind(expires_at)
    .execute(&pool)
    .await?;

    Ok(axum::Json(serde_json::json!({
        "status": "success",
        "message": format!("Role '{}' assigned to {}", payload.role_name, payload.email),
        "expires_at": expires_at
    })))
}
