use crate::db::Db;
use crate::error::{AppError, AppResult};
use crate::middleware::auth::AdminClaims;
use crate::models::types::{AccessLevel, Resource};
use crate::models::{Role, RolePermission, user::UserSummary};
use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response, Redirect},
    Form, Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(askama::Template)]
#[template(path = "partials/permission_row.html")]
pub struct PermissionRowTemplate {
    pub resources: Vec<&'static str>,
    pub access_levels: Vec<&'static str>,
}

pub async fn permission_row() -> PermissionRowTemplate {
    PermissionRowTemplate {
        resources: vec!["orders", "customers", "reports", "inventory", "admin_panel"],
        access_levels: vec!["read", "write", "admin"],
    }
}

pub async fn list_users(
    _admin: AdminClaims,
    State(pool): State<Db>,
) -> AppResult<Json<Vec<UserSummary>>> {
    let users = sqlx::query_as::<_, UserSummary>(
        "SELECT user_id, email, full_name, is_admin, is_active, disabled_reason
         FROM users
         WHERE is_admin = FALSE
         ORDER BY created_at DESC",
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
                RolePermission {
                    resource: res_str.parse().unwrap_or(Resource::Orders),
                    access: acc_str.parse().unwrap_or(AccessLevel::Read),
                }
            })
            .collect();
    }

    let pages = if total == 0 {
        1
    } else {
        (total + size - 1) / size
    };

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
    headers: axum::http::HeaderMap,
    Form(payload): Form<CreateRoleRequest>,
) -> AppResult<Response> {
    let mut tx = pool.begin().await?;
    let role_id = Uuid::new_v4();

    let insert_result =
        sqlx::query("INSERT INTO roles (role_id, name, description) VALUES ($1, $2, $3)")
            .bind(role_id)
            .bind(&payload.name)
            .bind(&payload.description)
            .execute(&mut *tx)
            .await;

    match insert_result {
        Ok(_) => {}
        Err(sqlx::Error::Database(db_err)) if db_err.code().as_deref() == Some("23505") => {
            return Ok(AppError::Conflict(format!(
                "A role named '{}' already exists.",
                payload.name
            ))
            .smart_response(&headers));
        }
        Err(e) => return Err(e.into()),
    }

    for perm in payload.permissions {
        let resource_str = perm.resource.to_string();
        let access_str = perm.access.to_string();

        match sqlx::query(
            "INSERT INTO role_permissions (role_id, resource, access_level) VALUES ($1, $2, $3)",
        )
        .bind(role_id)
        .bind(&resource_str)
        .bind(&access_str)
        .execute(&mut *tx)
        .await
        {
            Ok(_) => {}
            Err(sqlx::Error::Database(db_err)) if db_err.code().as_deref() == Some("23505") => {
                return Ok(AppError::Conflict(format!(
                    "Duplicate permission: '{}' with '{}' is already assigned to this role.",
                    resource_str, access_str
                ))
                .smart_response(&headers));
            }
            Err(e) => return Err(e.into()),
        }
    }

    tx.commit().await?;

    // On success, redirect to roles
    let mut headers = axum::http::HeaderMap::new();
    headers.insert("HX-Redirect", "/roles".parse().unwrap());
    Ok((axum::http::StatusCode::CREATED, headers, Redirect::to("/roles")).into_response())
}
pub async fn assign_role(
    _admin: AdminClaims,
    State(pool): State<Db>,
    headers: axum::http::HeaderMap,
    Form(payload): Form<AssignRoleRequest>,
) -> AppResult<Response> {
    use sqlx::Row;

    // 1. Find user_id by email
    let user_row = match sqlx::query("SELECT user_id FROM users WHERE email = $1")
        .bind(&payload.email)
        .fetch_optional(&pool)
        .await?
    {
        Some(row) => row,
        None => return Ok(AppError::UserNotFound.smart_response(&headers)),
    };
    let user_id: Uuid = user_row.get("user_id");

    // 2. Find role_id by name
    let role_row = match sqlx::query("SELECT role_id FROM roles WHERE name = $1")
        .bind(&payload.role_name)
        .fetch_optional(&pool)
        .await?
    {
        Some(row) => row,
        None => return Ok(AppError::RoleNotFound.smart_response(&headers)),
    };
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

    // On success, redirect to roles or roles list
    let mut headers = axum::http::HeaderMap::new();
    headers.insert("HX-Redirect", "/roles".parse().unwrap());
    Ok((axum::http::StatusCode::SEE_OTHER, headers, Redirect::to("/roles")).into_response())
}

pub async fn delete_role(
    _admin: AdminClaims,
    State(pool): State<Db>,
    Path(role_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let result = sqlx::query("DELETE FROM roles WHERE role_id = $1")
        .bind(role_id)
        .execute(&pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::RoleNotFound);
    }

    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "Role deleted."
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
    _admin: AdminClaims,
    State(pool): State<Db>,
    Path(role_id): Path<Uuid>,
) -> AppResult<Json<RoleDetailResponse>> {
    use sqlx::Row;

    let mut role = sqlx::query_as::<_, Role>(
        "SELECT role_id, name, description, created_at FROM roles WHERE role_id = $1",
    )
    .bind(role_id)
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::RoleNotFound)?;

    let perms =
        sqlx::query("SELECT resource, access_level FROM role_permissions WHERE role_id = $1")
            .bind(role_id)
            .fetch_all(&pool)
            .await?;

    role.permissions = perms
        .into_iter()
        .map(|p| {
            let res_str: String = p.get("resource");
            let acc_str: String = p.get("access_level");
            RolePermission {
                resource: res_str.parse().unwrap_or(Resource::Orders),
                access: acc_str.parse().unwrap_or(AccessLevel::Read),
            }
        })
        .collect();

    let assignment_rows = sqlx::query(
        "SELECT u.email, ra.assigned_at, ra.expires_at,
                CASE WHEN ra.expires_at IS NULL OR ra.expires_at > NOW() THEN true ELSE false END AS is_active
         FROM role_assignments ra
         JOIN users u ON u.user_id = ra.user_id
         WHERE ra.role_id = $1
         ORDER BY ra.assigned_at DESC",
    )
    .bind(role_id)
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

#[derive(Deserialize)]
pub struct DisableUserRequest {
    pub reason: Option<String>,
}

pub async fn disable_user(
    admin: AdminClaims,
    State(pool): State<Db>,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<DisableUserRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let actor_id = Uuid::parse_str(&admin.0.sub).map_err(|_| AppError::Unauthorized)?;

    // Prevent self-disable
    if actor_id == user_id {
        return Err(AppError::BadRequest(
            "You cannot disable your own account.".into(),
        ));
    }

    let mut tx = pool.begin().await?;

    // Atomically disable + bump session_version
    // WHERE is_active = TRUE prevents double-firing on race conditions
    let updated = sqlx::query(
        r#"
        UPDATE users
        SET
            is_active       = FALSE,
            session_version = session_version + 1,
            disabled_at     = NOW(),
            disabled_by     = $1,
            disabled_reason = $2
        WHERE user_id = $3
          AND is_active = TRUE
        RETURNING user_id
        "#,
    )
    .bind(actor_id)
    .bind(&payload.reason)
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await?;

    if updated.is_none() {
        tx.rollback().await?;
        return Err(AppError::BadRequest(
            "User not found or already disabled.".into(),
        ));
    }

    // Write audit log
    sqlx::query(
        r#"
        INSERT INTO user_audit_log (target_user_id, actor_id, action, reason)
        VALUES ($1, $2, 'disabled', $3)
        "#,
    )
    .bind(user_id)
    .bind(actor_id)
    .bind(&payload.reason)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "User has been disabled. All active sessions are now invalid."
    })))
}

pub async fn enable_user(
    admin: AdminClaims,
    State(pool): State<Db>,
    Path(user_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let actor_id = Uuid::parse_str(&admin.0.sub).map_err(|_| AppError::Unauthorized)?;

    let mut tx = pool.begin().await?;

    let updated = sqlx::query(
        r#"
        UPDATE users
        SET
            is_active       = TRUE,
            disabled_at     = NULL,
            disabled_by     = NULL,
            disabled_reason = NULL
        WHERE user_id = $1
          AND is_active = FALSE
        RETURNING user_id
        "#,
    )
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await?;

    if updated.is_none() {
        tx.rollback().await?;
        return Err(AppError::BadRequest(
            "User not found or already active.".into(),
        ));
    }

    sqlx::query(
        "INSERT INTO user_audit_log (target_user_id, actor_id, action) VALUES ($1, $2, 'enabled')",
    )
    .bind(user_id)
    .bind(actor_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(Json(
        serde_json::json!({ "status": "success", "message": "User re-enabled." }),
    ))
}

#[derive(Deserialize)]
pub struct WizardStep1Form {
    pub name: String,
    pub description: String,
}

pub async fn wizard_step_1(
    _admin: AdminClaims,
    jar: axum_extra::extract::cookie::CookieJar,
    Form(payload): Form<WizardStep1Form>,
) -> impl IntoResponse {
    let sidebar_pinned = jar.get("sidebar_pinned").map(|c| c.value() == "true").unwrap_or(true);

    if payload.name.trim().is_empty() {
        return crate::handlers::dashboard::CreateRoleWizardTemplate {
            sidebar_pinned,
            user_email: _admin.0.email,
            show_banner: false,
            css_version: env!("CSS_VERSION"),
            is_admin: true,
            step: 1,
            role_name: payload.name,
            role_description: payload.description,
            error: "Role name is required.".to_string(),
            permissions: Vec::new(),
        }
        .into_response();
    }

    // Advance to Step 2
    crate::handlers::dashboard::CreateRoleWizardTemplate {
        sidebar_pinned,
        user_email: _admin.0.email,
        show_banner: false,
        css_version: env!("CSS_VERSION"),
        is_admin: true,
        step: 2,
        role_name: payload.name,
        role_description: payload.description,
        error: String::new(),
        permissions: Vec::new(),
    }
    .into_response()
}

#[derive(Deserialize)]
pub struct WizardStep2Form {
    pub role_name: String,
    pub role_description: String,
    pub resources: Vec<String>,
    pub access_levels: Vec<String>,
}

pub async fn wizard_step_2(
    _admin: AdminClaims,
    jar: axum_extra::extract::cookie::CookieJar,
    Form(payload): Form<WizardStep2Form>,
) -> impl IntoResponse {
    let sidebar_pinned = jar.get("sidebar_pinned").map(|c| c.value() == "true").unwrap_or(true);

    let mut permissions = Vec::new();
    for (res, acc) in payload.resources.iter().zip(payload.access_levels.iter()) {
        if !res.is_empty() && !acc.is_empty() {
            permissions.push((res.clone(), acc.clone()));
        }
    }

    if permissions.is_empty() {
        return crate::handlers::dashboard::CreateRoleWizardTemplate {
            sidebar_pinned,
            user_email: _admin.0.email,
            show_banner: false,
            css_version: env!("CSS_VERSION"),
            is_admin: true,
            step: 2,
            role_name: payload.role_name,
            role_description: payload.role_description,
            error: "At least one permission is required.".to_string(),
            permissions: Vec::new(),
        }
        .into_response();
    }

    // Advance to Step 3 (Review)
    crate::handlers::dashboard::CreateRoleWizardTemplate {
        sidebar_pinned,
        user_email: _admin.0.email,
        show_banner: false,
        css_version: env!("CSS_VERSION"),
        is_admin: true,
        step: 3,
        role_name: payload.role_name,
        role_description: payload.role_description,
        error: String::new(),
        permissions,
    }
    .into_response()
}
