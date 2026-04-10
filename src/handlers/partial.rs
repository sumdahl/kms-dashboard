use askama::Template;
use axum::{
    extract::{Path, Query, State},
    response::{Html, IntoResponse, Response},
};
use chrono::Utc;
use serde::Deserialize;
use uuid::Uuid;

use crate::db::Db;
use crate::error::AppResult;
use crate::models::Claims;
use crate::page_context::PageContext;

// ── Account Menu ──────────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "partials/account_menu.html")]
struct AccountMenuTemplate {
    pub user_email: String,
    pub theme_label: String,
    pub theme_icon: String,
}

pub async fn account_menu(ctx: PageContext) -> Response {
    let (theme_label, theme_icon) = if ctx.dark_mode {
        (
            "Light mode".to_string(),
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M4.93 19.07l1.41-1.41M17.66 6.34l1.41-1.41"/></svg>"#.to_string(),
        )
    } else {
        (
            "Dark mode".to_string(),
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z"/></svg>"#.to_string(),
        )
    };

    AccountMenuTemplate {
        user_email: ctx.user_email,
        theme_label,
        theme_icon,
    }
    .into_response()
}

// ── Users List ────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct UsersListQuery {
    pub q: Option<String>,
    pub filter: Option<String>,
}

#[derive(Debug)]
pub struct UserRow {
    pub user_id: String,
    pub full_name: String,
    pub email: String,
    pub initials: String,
    pub is_active: bool,
    pub is_admin: bool,
    pub disabled_reason: Option<String>,
}

#[derive(Template)]
#[template(path = "partials/users/list.html")]
struct UsersListTemplate {
    pub users: Vec<UserRow>,
    pub total: usize,
}

pub async fn users_list(
    State(pool): State<Db>,
    Query(params): Query<UsersListQuery>,
) -> AppResult<Response> {
    let q = params.q.unwrap_or_default();
    let filter = params.filter.unwrap_or_else(|| "all".into());

    let rows = sqlx::query!(
        r#"SELECT user_id, email, full_name, is_active, is_admin, disabled_reason
           FROM users
           WHERE is_admin = FALSE
             AND (
               $1 = '' OR
               email ILIKE '%' || $1 || '%' OR
               full_name ILIKE '%' || $1 || '%'
             )
             AND (
               $2 = 'all' OR
               ($2 = 'active' AND is_active = TRUE) OR
               ($2 = 'disabled' AND is_active = FALSE)
             )
           ORDER BY created_at DESC"#,
        q,
        filter
    )
    .fetch_all(&pool)
    .await?;

    let total = rows.len();
    let users = rows
        .into_iter()
        .map(|r| UserRow {
            user_id: r.user_id.to_string(),
            initials: initials(&r.full_name),
            full_name: r.full_name,
            email: r.email,
            is_active: r.is_active,
            is_admin: r.is_admin,
            disabled_reason: r.disabled_reason,
        })
        .collect();

    Ok(UsersListTemplate { users, total }.into_response())
}

// ── User Detail ───────────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "partials/users/detail.html")]
struct UserDetailTemplate {
    pub user: UserRow,
}

pub async fn user_detail(State(pool): State<Db>, Path(user_id): Path<Uuid>) -> AppResult<Response> {
    let r = sqlx::query!(
        "SELECT user_id, email, full_name, is_active, is_admin, disabled_reason
         FROM users WHERE user_id = $1",
        user_id
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(crate::error::AppError::UserNotFound)?;

    Ok(UserDetailTemplate {
        user: UserRow {
            user_id: r.user_id.to_string(),
            initials: initials(&r.full_name),
            full_name: r.full_name,
            email: r.email,
            is_active: r.is_active,
            is_admin: r.is_admin,
            disabled_reason: r.disabled_reason,
        },
    }
    .into_response())
}

// ── Disable Modal ─────────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "partials/users/disable_modal.html")]
struct DisableModalTemplate {
    pub user: DisableModalUser,
    pub error: Option<String>,
    pub reason_error: String,
}

pub struct DisableModalUser {
    pub user_id: String,
    pub email: String,
}

pub async fn disable_modal(
    State(pool): State<Db>,
    Path(user_id): Path<Uuid>,
) -> AppResult<Response> {
    let r = sqlx::query!(
        "SELECT user_id, email FROM users WHERE user_id = $1",
        user_id
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(crate::error::AppError::UserNotFound)?;

    Ok(DisableModalTemplate {
        user: DisableModalUser {
            user_id: r.user_id.to_string(),
            email: r.email,
        },
        error: None,
        reason_error: String::new(),
    }
    .into_response())
}

// ── User Stats ────────────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "partials/users/stats.html")]
struct UsersStatsTemplate {
    pub total: i64,
    pub active: i64,
    pub disabled: i64,
    pub admins: i64,
    pub summary_date: String,
}

pub async fn users_stats(State(pool): State<Db>) -> AppResult<Response> {
    let row = sqlx::query!(
        r#"SELECT
            COUNT(*) FILTER (WHERE is_admin = FALSE) AS "total!",
            COUNT(*) FILTER (WHERE is_admin = FALSE AND is_active = TRUE) AS "active!",
            COUNT(*) FILTER (WHERE is_admin = FALSE AND is_active = FALSE) AS "disabled!",
            COUNT(*) FILTER (WHERE is_admin = TRUE) AS "admins!"
           FROM users"#
    )
    .fetch_one(&pool)
    .await?;

    Ok(UsersStatsTemplate {
        total: row.total,
        active: row.active,
        disabled: row.disabled,
        admins: row.admins,
        summary_date: Utc::now().format("%B %d").to_string(),
    }
    .into_response())
}

// ── Roles List ────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct RolesListQuery {
    pub search: Option<String>,
    pub filter: Option<String>,
    pub sort: Option<String>,
    pub page: Option<i64>,
    pub size: Option<i64>,
}

pub struct RoleRow {
    pub role_id: String,
    pub name: String,
    pub description: String,
    pub initials: String,
    pub permissions_count: usize,
    pub resources: String,
    pub created_at: String,
}

#[derive(Template)]
#[template(path = "partials/roles/list.html")]
struct RolesListTemplate {
    pub roles: Vec<RoleRow>,
    pub total: i64,
    pub page: i64,
    pub pages: i64,
    pub size: i64,
    pub query: String,
    pub filter: String,
}

pub async fn roles_list(
    State(pool): State<Db>,
    Query(params): Query<RolesListQuery>,
) -> AppResult<Response> {
    let search = params.search.unwrap_or_default();
    let filter = params.filter.unwrap_or_else(|| "all".into());
    let page = params.page.unwrap_or(1).max(1);
    let size = params.size.unwrap_or(8).clamp(1, 100);
    let offset = (page - 1) * size;

    let pattern = format!("%{}%", search);

    let total: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(*) FROM roles r
           WHERE ($1 = '' OR r.name ILIKE $2 OR r.description ILIKE $2)
             AND ($3 = 'all'
               OR ($3 = 'with_permissions' AND EXISTS (SELECT 1 FROM role_permissions WHERE role_id = r.role_id))
               OR ($3 = 'empty' AND NOT EXISTS (SELECT 1 FROM role_permissions WHERE role_id = r.role_id))
             )"#,
    )
    .bind(&search)
    .bind(&pattern)
    .bind(&filter)
    .fetch_one(&pool)
    .await?;

    let rows = sqlx::query!(
        r#"SELECT r.role_id, r.name, r.description, r.created_at,
                  COUNT(rp.role_id) AS "perm_count!",
                  STRING_AGG(DISTINCT rp.resource, ', ') AS resources
           FROM roles r
           LEFT JOIN role_permissions rp ON rp.role_id = r.role_id
           WHERE ($1 = '' OR r.name ILIKE $2 OR r.description ILIKE $2)
             AND ($3 = 'all'
               OR ($3 = 'with_permissions' AND EXISTS (SELECT 1 FROM role_permissions WHERE role_id = r.role_id))
               OR ($3 = 'empty' AND NOT EXISTS (SELECT 1 FROM role_permissions WHERE role_id = r.role_id))
             )
           GROUP BY r.role_id, r.name, r.description, r.created_at
           ORDER BY r.created_at DESC
           LIMIT $4 OFFSET $5"#,
        search,
        pattern,
        filter,
        size,
        offset
    )
    .fetch_all(&pool)
    .await?;

    let roles = rows
        .into_iter()
        .map(|r| RoleRow {
            role_id: r.role_id.to_string(),
            initials: role_initials(&r.name),
            name: r.name,
            description: r.description,
            permissions_count: r.perm_count as usize,
            resources: r.resources.unwrap_or_default(),
            created_at: r.created_at.format("%b %d, %Y").to_string(),
        })
        .collect();

    let pages = if total == 0 {
        1
    } else {
        (total + size - 1) / size
    };

    Ok(RolesListTemplate {
        roles,
        total,
        page,
        pages,
        size,
        query: search,
        filter,
    }
    .into_response())
}

// ── Role Detail Partial ───────────────────────────────────────────────────────

pub struct RoleDetailPerm {
    pub resource: String,
    pub access: String,
}

pub struct RoleDetailData {
    pub role_id: String,
    pub name: String,
    pub description: String,
    pub initials: String,
    pub created_at: String,
    pub permissions_count: usize,
    pub permissions: Vec<RoleDetailPerm>,
}

#[derive(Template)]
#[template(path = "partials/roles/detail.html")]
struct RoleDetailPartialTemplate {
    pub role: RoleDetailData,
}

pub async fn role_detail_partial(
    State(pool): State<Db>,
    Path(role_id): Path<Uuid>,
) -> AppResult<Response> {
    let role = sqlx::query!(
        "SELECT role_id, name, description, created_at FROM roles WHERE role_id = $1",
        role_id
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(crate::error::AppError::RoleNotFound)?;

    let perms = sqlx::query!(
        "SELECT resource, access_level FROM role_permissions WHERE role_id = $1",
        role_id
    )
    .fetch_all(&pool)
    .await?;

    let permissions: Vec<RoleDetailPerm> = perms
        .into_iter()
        .map(|p| RoleDetailPerm {
            resource: p.resource,
            access: p.access_level,
        })
        .collect();

    let permissions_count = permissions.len();

    Ok(RoleDetailPartialTemplate {
        role: RoleDetailData {
            role_id: role.role_id.to_string(),
            initials: role_initials(&role.name),
            name: role.name,
            description: role.description,
            created_at: role.created_at.format("%b %d, %Y").to_string(),
            permissions_count,
            permissions,
        },
    }
    .into_response())
}

// ── Delete Modal ──────────────────────────────────────────────────────────────

pub struct DeleteModalRole {
    pub role_id: String,
    pub name: String,
}

#[derive(Template)]
#[template(path = "partials/roles/delete_modal.html")]
struct DeleteModalTemplate {
    pub role: DeleteModalRole,
    pub error: Option<String>,
}

pub async fn delete_modal(
    State(pool): State<Db>,
    Path(role_id): Path<Uuid>,
) -> AppResult<Response> {
    let role = sqlx::query!(
        "SELECT role_id, name FROM roles WHERE role_id = $1",
        role_id
    )
    .fetch_optional(&pool)
    .await?
    .ok_or(crate::error::AppError::RoleNotFound)?;

    Ok(DeleteModalTemplate {
        role: DeleteModalRole {
            role_id: role.role_id.to_string(),
            name: role.name,
        },
        error: None,
    }
    .into_response())
}

// ── Roles Stats ───────────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "partials/roles/stats.html")]
struct RolesStatsTemplate {
    pub total_roles: i64,
    pub total_permissions: i64,
    pub unique_resources: i64,
    pub admin_count: i64,
    pub summary_date: String,
}

pub async fn roles_stats(State(pool): State<Db>) -> AppResult<Response> {
    let row = sqlx::query!(
        r#"SELECT
            (SELECT COUNT(*) FROM roles) AS "total_roles!",
            COUNT(*) AS "total_permissions!",
            COUNT(DISTINCT resource) AS "unique_resources!",
            COUNT(*) FILTER (WHERE access_level = 'admin') AS "admin_count!"
           FROM role_permissions"#
    )
    .fetch_one(&pool)
    .await?;

    Ok(RolesStatsTemplate {
        total_roles: row.total_roles,
        total_permissions: row.total_permissions,
        unique_resources: row.unique_resources,
        admin_count: row.admin_count,
        summary_date: Utc::now().format("%B %d").to_string(),
    }
    .into_response())
}

// ── Permission Row ────────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "partials/roles/permission_row.html")]
struct PermissionRowTemplate {
    pub resources: Vec<&'static str>,
    pub access_levels: Vec<&'static str>,
}

pub async fn permission_row() -> impl IntoResponse {
    PermissionRowTemplate {
        resources: vec!["orders", "customers", "reports", "inventory", "admin_panel"],
        access_levels: vec!["read", "write", "admin"],
    }
}

// ── Creation Method Modal ─────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "partials/roles/create_method_modal.html")]
struct CreateMethodModalTemplate {}

pub async fn create_method_modal() -> impl IntoResponse {
    CreateMethodModalTemplate {}
}

// ── My Roles (home page partial) ──────────────────────────────────────────────

pub struct MyRolePermission {
    pub resource: String,
    pub access: String,
}

pub struct MyRole {
    pub name: String,
    pub description: String,
    pub permissions: Vec<MyRolePermission>,
    pub expires_at: Option<String>,
}

#[derive(Template)]
#[template(path = "partials/my_roles.html")]
struct MyRolesTemplate {
    pub user_roles: Vec<MyRole>,
}

pub async fn my_roles(claims: Claims, State(pool): State<Db>) -> AppResult<Response> {
    use sqlx::Row;
    let user_id = claims
        .sub
        .parse::<Uuid>()
        .map_err(|_| crate::error::AppError::Unauthorized)?;

    let rows = sqlx::query(
        r#"SELECT r.role_id, r.name, r.description, ra.expires_at
           FROM role_assignments ra
           JOIN roles r ON ra.role_id = r.role_id
           WHERE ra.user_id = $1
             AND (ra.expires_at IS NULL OR ra.expires_at > NOW())
           ORDER BY r.name"#,
    )
    .bind(user_id)
    .fetch_all(&pool)
    .await?;

    let mut user_roles: Vec<MyRole> = Vec::new();
    for row in rows {
        let role_id: Uuid = row.get("role_id");
        let name: String = row.get("name");
        let description: String = row.get("description");
        let expires_at: Option<chrono::DateTime<Utc>> = row.get("expires_at");

        let perm_rows =
            sqlx::query("SELECT resource, access_level FROM role_permissions WHERE role_id = $1")
                .bind(role_id)
                .fetch_all(&pool)
                .await?;

        let permissions = perm_rows
            .iter()
            .map(|p| MyRolePermission {
                resource: p.get::<String, _>("resource"),
                access: p.get::<String, _>("access_level"),
            })
            .collect();

        user_roles.push(MyRole {
            name,
            description,
            permissions,
            expires_at: expires_at.map(|e| e.format("%b %d, %Y").to_string()),
        });
    }

    Ok(MyRolesTemplate { user_roles }.into_response())
}

// ── Search Results ────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct SearchParams {
    pub q: Option<String>,
}

pub struct SearchResult {
    pub title: String,
    pub description: String,
    pub url: String,
    pub category: &'static str,
}

#[derive(Template)]
#[template(path = "partials/search_results.html")]
struct SearchResultsTemplate {
    pub results: Vec<SearchResult>,
    pub query: String,
}

pub async fn search_results(
    State(pool): State<Db>,
    Query(params): Query<SearchParams>,
) -> AppResult<Response> {
    let query = params.q.unwrap_or_default();
    let q = query.trim().to_string();

    if q.len() < 3 {
        return Ok(SearchResultsTemplate {
            results: vec![],
            query: q,
        }
        .into_response());
    }

    let pattern = format!("%{}%", q);
    let mut results = Vec::new();

    // Static pages
    for (title, desc, url) in [
        ("Account Home", "Overview of your account and security", "/"),
        ("Manage Roles", "View and edit system roles", "/roles"),
        ("Assign Roles", "Grant permissions to users", "/assign"),
    ] {
        if title.to_lowercase().contains(&q.to_lowercase())
            || desc.to_lowercase().contains(&q.to_lowercase())
        {
            results.push(SearchResult {
                title: title.to_string(),
                description: desc.to_string(),
                url: url.to_string(),
                category: "Page",
            });
        }
    }

    // Roles
    let roles = sqlx::query!(
        "SELECT name, description FROM roles WHERE name ILIKE $1 OR description ILIKE $1 LIMIT 5",
        pattern
    )
    .fetch_all(&pool)
    .await?;

    for role in roles {
        results.push(SearchResult {
            url: format!("/roles/{}", role.name),
            title: role.name,
            description: role.description,
            category: "Role",
        });
    }

    // Users
    let users = sqlx::query!(
        "SELECT email, full_name FROM users WHERE email ILIKE $1 OR full_name ILIKE $1 LIMIT 5",
        pattern
    )
    .fetch_all(&pool)
    .await?;

    for user in users {
        results.push(SearchResult {
            title: user.email,
            description: user.full_name,
            url: "#".to_string(),
            category: "User",
        });
    }

    Ok(SearchResultsTemplate { results, query: q }.into_response())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn initials(name: &str) -> String {
    let parts: Vec<&str> = name.trim().split_whitespace().collect();
    match parts.len() {
        0 => "?".to_string(),
        1 => parts[0].chars().take(2).collect::<String>().to_uppercase(),
        _ => format!(
            "{}{}",
            parts[0].chars().next().unwrap_or('?'),
            parts[1].chars().next().unwrap_or('?')
        )
        .to_uppercase(),
    }
}

fn role_initials(name: &str) -> String {
    initials(name)
}
