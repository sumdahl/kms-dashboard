use crate::auth::dto::{
    first_field_message, validate_and_parse_create_role_form, CreateRoleFormRequest,
};
use crate::db::Db;
use crate::error::{AppError, AppResult};
use crate::middleware::auth::AdminClaims;
use crate::models::types::{AccessLevel, Resource, RolePermissionInput};
use crate::models::{Role, RolePermission};
use askama::Template;
use axum::{
    body::Body,
    extract::rejection::FormRejection,
    extract::{Form, Path, Query, State},
    http::header::REFERER,
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    Json,
};
use crate::ui::global_message;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow, Clone)]
pub struct UserSummary {
    pub user_id: Uuid,
    pub email: String,
    pub full_name: String,
    pub is_admin: bool,
    pub is_active: bool,
    pub disabled_reason: Option<String>,
}

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
#[derive(Clone)]
pub struct QuickPermissionRow {
    pub resource_sel: usize,
    pub access_sel: usize,
}

/// Quick create role page (`/roles/quick`) — shell + included form fragment.
#[derive(Template)]
#[template(path = "dashboard/quick_create_role.html")]
pub struct QuickCreateRoleView {
    pub sidebar_pinned: bool,
    pub user_email: String,
    pub css_version: &'static str,
    pub is_admin: bool,
    pub nav_active: &'static str,
    pub banner: Option<String>,
    pub name: String,
    pub description: String,
    pub name_error: Option<String>,
    pub description_error: Option<String>,
    pub resource_error: Option<String>,
    pub oob_swap: bool,
    pub resources: Vec<String>,
    pub access_levels: Vec<String>,
    pub permission_rows: Vec<QuickPermissionRow>,
}

/// Create role wizard (`/roles/new`) — shell + form fragment (same validation model as quick create).
#[derive(Template)]
#[template(path = "dashboard/create_role_wizard.html")]
pub struct CreateRoleWizardView {
    pub sidebar_pinned: bool,
    pub user_email: String,
    pub css_version: &'static str,
    pub is_admin: bool,
    pub nav_active: &'static str,
    pub banner: Option<String>,
    pub name: String,
    pub description: String,
    pub name_error: Option<String>,
    pub description_error: Option<String>,
    pub resource_error: Option<String>,
    pub oob_swap: bool,
    pub resources: Vec<String>,
    pub access_levels: Vec<String>,
    pub permission_rows: Vec<QuickPermissionRow>,
    pub redirect: String,
}

pub fn quick_create_resource_list() -> Vec<String> {
    vec![
        "orders".into(),
        "customers".into(),
        "reports".into(),
        "inventory".into(),
        "admin_panel".into(),
    ]
}

pub fn quick_create_access_level_list() -> Vec<String> {
    vec!["read".into(), "write".into(), "admin".into()]
}

fn option_index(opts: &[String], value: &str) -> usize {
    opts.iter()
        .position(|o| o == value)
        .unwrap_or(0)
}

pub fn quick_create_default_permission_rows() -> Vec<QuickPermissionRow> {
    vec![QuickPermissionRow {
        resource_sel: 0,
        access_sel: 0,
    }]
}

fn quick_permission_rows_from_form(form: &CreateRoleFormRequest) -> Vec<QuickPermissionRow> {
    let resources = quick_create_resource_list();
    let access_levels = quick_create_access_level_list();
    let n = form.resource.len().min(form.access.len());
    let mut rows = Vec::new();
    for i in 0..n {
        let r = form.resource[i].trim();
        let a = form.access[i].trim();
        rows.push(QuickPermissionRow {
            resource_sel: option_index(&resources, r),
            access_sel: option_index(&access_levels, a),
        });
    }
    if rows.is_empty() {
        quick_create_default_permission_rows()
    } else {
        rows
    }
}

fn referer_is_quick_create(headers: &HeaderMap) -> bool {
    headers
        .get(REFERER)
        .and_then(|v| v.to_str().ok())
        .map(|r| r.contains("/roles/quick"))
        .unwrap_or(false)
}

fn is_quick_create_htmx(headers: &HeaderMap, form: Option<&CreateRoleFormRequest>) -> bool {
    if !is_htmx(headers) {
        return false;
    }
    match form {
        Some(f) => f.error_redirect.as_deref().map(str::trim) == Some("/roles/quick"),
        None => referer_is_quick_create(headers),
    }
}

fn referer_is_wizard(headers: &HeaderMap) -> bool {
    headers
        .get(REFERER)
        .and_then(|v| v.to_str().ok())
        .map(|r| r.contains("/roles/new"))
        .unwrap_or(false)
}

fn is_wizard_htmx(headers: &HeaderMap, form: Option<&CreateRoleFormRequest>) -> bool {
    if !is_htmx(headers) {
        return false;
    }
    match form {
        Some(f) => f.error_redirect.as_deref().map(str::trim) == Some("/roles/new"),
        None => referer_is_wizard(headers),
    }
}

fn hx_redirect_response(target: &str) -> Response {
    let hv = HeaderValue::try_from(target).unwrap_or_else(|_| HeaderValue::from_static("/roles"));
    Response::builder()
        .status(StatusCode::NO_CONTENT)
        .header(HeaderName::from_static("hx-redirect"), hv)
        .body(Body::empty())
        .unwrap()
        .into_response()
}

fn quick_create_shell(
    admin: &AdminClaims,
    form: &CreateRoleFormRequest,
    banner: Option<String>,
    name_error: Option<String>,
    description_error: Option<String>,
    resource_error: Option<String>,
    oob_swap: bool,
) -> QuickCreateRoleView {
    QuickCreateRoleView {
        sidebar_pinned: true,
        user_email: admin.0.email.clone(),
        css_version: env!("CSS_VERSION"),
        is_admin: admin.0.is_admin,
        nav_active: "roles",
        banner,
        name: form.name.clone(),
        description: form.description.clone(),
        name_error,
        description_error,
        resource_error,
        oob_swap,
        resources: quick_create_resource_list(),
        access_levels: quick_create_access_level_list(),
        permission_rows: quick_permission_rows_from_form(form),
    }
}

fn wizard_shell(
    admin: &AdminClaims,
    form: &CreateRoleFormRequest,
    banner: Option<String>,
    name_error: Option<String>,
    description_error: Option<String>,
    resource_error: Option<String>,
    oob_swap: bool,
) -> CreateRoleWizardView {
    let redirect = form
        .redirect
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("/roles?skip_onboarding=true")
        .to_string();
    CreateRoleWizardView {
        sidebar_pinned: true,
        user_email: admin.0.email.clone(),
        css_version: env!("CSS_VERSION"),
        is_admin: admin.0.is_admin,
        nav_active: "roles",
        banner,
        name: form.name.clone(),
        description: form.description.clone(),
        name_error,
        description_error,
        resource_error,
        oob_swap,
        resources: quick_create_resource_list(),
        access_levels: quick_create_access_level_list(),
        permission_rows: quick_permission_rows_from_form(form),
        redirect,
    }
}

fn empty_quick_create_form() -> CreateRoleFormRequest {
    CreateRoleFormRequest {
        name: String::new(),
        description: String::new(),
        resource: Vec::new(),
        access: Vec::new(),
        redirect: None,
        error_redirect: Some("/roles/quick".into()),
    }
}

fn empty_wizard_form() -> CreateRoleFormRequest {
    CreateRoleFormRequest {
        name: String::new(),
        description: String::new(),
        resource: Vec::new(),
        access: Vec::new(),
        redirect: Some("/roles?skip_onboarding=true".into()),
        error_redirect: Some("/roles/new".into()),
    }
}



pub async fn list_users(
    _admin: AdminClaims,
    State(pool): State<Db>,
) -> AppResult<Json<Vec<UserSummary>>> {
    let users = fetch_user_summaries(&pool).await?;
    Ok(Json(users))
}

pub async fn fetch_user_summaries(pool: &Db) -> AppResult<Vec<UserSummary>> {
    sqlx::query_as::<_, UserSummary>(
        "SELECT user_id, email, full_name, is_admin, is_active, disabled_reason
         FROM users
         WHERE is_admin = FALSE
         ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

pub struct CreateRoleRequest {
    pub name: String,
    pub description: String,
    pub permissions: Vec<RolePermissionInput>,
}

#[derive(Deserialize)]
pub struct AssignRoleRequest {
    pub email: String,
    pub role_name: String,
    pub duration_secs: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct AssignRoleHtmlForm {
    pub email: String,
    pub role_name: String,
    pub duration_hours: Option<String>,
    #[serde(default)]
    pub redirect: Option<String>,
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

pub fn query_param_encode(value: &str) -> String {
    value.bytes().fold(String::new(), |mut acc, b| {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                acc.push(b as char)
            }
            b' ' => acc.push('+'),
            _ => acc.push_str(&format!("%{:02X}", b)),
        }
        acc
    })
}

fn is_htmx(headers: &HeaderMap) -> bool {
    headers
        .get("hx-request")
        .and_then(|v| v.to_str().ok())
        == Some("true")
}

fn users_htmx_initials(full_name: &str) -> String {
    let t = full_name.trim();
    if t.is_empty() {
        return "?".to_string();
    }
    let parts: Vec<&str> = t.split_whitespace().collect();
    if parts.len() == 1 {
        parts[0].chars().take(2).collect::<String>().to_uppercase()
    } else {
        let a = parts[0].chars().next().unwrap_or('?');
        let b = parts[1].chars().next().unwrap_or('?');
        format!("{}{}", a, b).to_uppercase()
    }
}

fn users_htmx_avatar_style(email: &str) -> String {
    let mut hash: i64 = 0;
    for b in email.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(b as i64);
    }
    let hue = hash.rem_euclid(360);
    format!(
        "background:hsl({hue},65%,28%);color:hsl({hue},75%,90%);box-shadow:inset 0 0 0 1px rgba(0,0,0,0.08)"
    )
}

#[derive(askama::Template)]
#[template(path = "dashboard/users_partial.html")]
struct UsersHtmxFragment {
    sidebar_pinned: bool,
    user_email: String,
    css_version: &'static str,
    is_admin: bool,
    nav_active: &'static str,
    users: Vec<UserSummary>,
    total_users: usize,
    active_users: usize,
    disabled_users: usize,
    admin_users: usize,
    summary_date: String,
}

impl UsersHtmxFragment {
    fn initials(full_name: &str) -> String {
        users_htmx_initials(full_name)
    }
    fn avatar_style(email: &str) -> String {
        users_htmx_avatar_style(email)
    }
}

async fn users_htmx_html(pool: &Db, admin: &AdminClaims) -> Result<String, AppError> {
    let users = fetch_user_summaries(pool).await?;
    let admin_users = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*)::bigint FROM users WHERE is_admin = TRUE",
    )
    .fetch_one(pool)
    .await? as usize;
    let total_users = users.len();
    let active_users = users.iter().filter(|u| u.is_active).count();
    let disabled_users = total_users.saturating_sub(active_users);
    let summary_date = Utc::now().format("%b %d").to_string();
    UsersHtmxFragment {
        sidebar_pinned: true,
        user_email: admin.0.email.clone(),
        css_version: env!("CSS_VERSION"),
        is_admin: admin.0.is_admin,
        nav_active: "users",
        users,
        total_users,
        active_users,
        disabled_users,
        admin_users,
        summary_date,
    }
    .render()
    .map_err(|e| AppError::Internal(e.to_string()))
}


// ── Role display helpers ──────────────────────────────────────────────────────

pub struct RoleDisplay {
    pub role_id: String,
    pub name: String,
    pub name_encoded: String,
    pub description_display: String,
    pub fg_color: String,
    pub bg_color: String,
    pub initial: String,
    pub perm_badges: String,
    pub perm_count: usize,
    pub has_permissions: bool,
    pub created_at_display: String,
}

fn role_color_hue(name: &str) -> i64 {
    let mut hash: i64 = 0;
    for b in name.bytes() {
        hash = hash
            .wrapping_shl(5)
            .wrapping_sub(hash)
            .wrapping_add(b as i64);
    }
    hash.rem_euclid(360)
}

pub fn role_to_display(role: Role) -> RoleDisplay {
    let hue = role_color_hue(&role.name);
    let fg_color = format!("hsl({}, 70%, 30%)", hue);
    let bg_color = format!("hsl({}, 80%, 90%)", hue);

    let t = role.name.trim();
    let initial = if t.is_empty() {
        "?".to_string()
    } else {
        let parts: Vec<&str> = t.split_whitespace().collect();
        if parts.len() == 1 {
            parts[0].chars().take(2).collect::<String>().to_uppercase()
        } else {
            let a = parts[0].chars().next().unwrap_or('?');
            let b = parts[1].chars().next().unwrap_or('?');
            format!("{}{}", a, b).to_uppercase()
        }
    };

    let perm_count = role.permissions.len();
    let has_permissions = perm_count > 0;
    let perm_badges = if has_permissions {
        role.permissions
            .iter()
            .map(|p| p.resource.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    } else {
        "No permissions assigned".to_string()
    };

    let description_display = if role.description.is_empty() {
        "No description".to_string()
    } else {
        role.description.clone()
    };

    RoleDisplay {
        role_id: role.role_id.to_string(),
        name_encoded: query_param_encode(&role.name),
        name: role.name,
        description_display,
        fg_color,
        bg_color,
        initial,
        perm_badges,
        perm_count,
        has_permissions,
        created_at_display: role.created_at.format("%b %d, %Y").to_string(),
    }
}

pub struct RolesListData {
    pub roles: Vec<RoleDisplay>,
    pub total: i64,
    pub page: i64,
    pub pages: i64,
    pub prev_page: i64,
    pub next_page: i64,
    pub search: String,
    pub start: i64,
    pub end: i64,
}

pub async fn load_roles_list_data(
    pool: &Db,
    page: i64,
    search: &str,
) -> AppResult<RolesListData> {
    let size = 8i64;
    let data = load_paginated_roles(pool, page, size, search).await?;

    let (start, end) = if data.total > 0 {
        let s = (page - 1) * size + 1;
        let e = s + data.roles.len() as i64 - 1;
        (s, e)
    } else {
        (0, 0)
    };

    let prev_page = (page - 1).max(1);
    let next_page = (page + 1).min(data.pages);

    Ok(RolesListData {
        roles: data.roles.into_iter().map(role_to_display).collect(),
        total: data.total,
        page: data.page,
        pages: data.pages,
        prev_page,
        next_page,
        search: search.to_string(),
        start,
        end,
    })
}

#[derive(askama::Template)]
#[template(path = "dashboard/roles_list_fragment.html")]
pub struct RolesListFragment {
    pub roles: Vec<RoleDisplay>,
    pub total: i64,
    pub page: i64,
    pub pages: i64,
    pub prev_page: i64,
    pub next_page: i64,
    pub search: String,
    pub start: i64,
    pub end: i64,
}


fn roles_list_fragment_render(d: RolesListData) -> Result<String, AppError> {
    let frag = RolesListFragment {
        roles: d.roles,
        total: d.total,
        page: d.page,
        pages: d.pages,
        prev_page: d.prev_page,
        next_page: d.next_page,
        search: d.search,
        start: d.start,
        end: d.end,
    };
    frag.render()
        .map_err(|e| AppError::Internal(e.to_string()))
}

pub async fn roles_list_htmx(
    _admin: AdminClaims,
    State(pool): State<Db>,
    Query(params): Query<ListRolesQuery>,
) -> AppResult<RolesListFragment> {
    let page = params.page.unwrap_or(1).max(1);
    let search = params.search.unwrap_or_default().trim().to_string();
    let d = load_roles_list_data(&pool, page, &search).await?;
    Ok(RolesListFragment {
        roles: d.roles,
        total: d.total,
        page: d.page,
        pages: d.pages,
        prev_page: d.prev_page,
        next_page: d.next_page,
        search: d.search,
        start: d.start,
        end: d.end,
    })
}

pub async fn delete_role_htmx(
    _admin: AdminClaims,
    State(pool): State<Db>,
    Path(role_id): Path<Uuid>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    let search = form
        .get("search")
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    let list_html = match load_roles_list_data(&pool, 1, &search).await {
        Ok(d) => match roles_list_fragment_render(d) {
            Ok(h) => h,
            Err(e) => {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    format!("<!-- {e} -->"),
                )
                    .into_response();
            }
        },
        Err(e) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("<!-- {e} -->"),
            )
                .into_response();
        }
    };

    let del = sqlx::query("DELETE FROM roles WHERE role_id = $1")
        .bind(role_id)
        .execute(&pool)
        .await;

    match del {
        Ok(result) if result.rows_affected() > 0 => {
            let html = match load_roles_list_data(&pool, 1, &search).await {
                Ok(d) => match roles_list_fragment_render(d) {
                    Ok(h) => h,
                    Err(e) => {
                        return (
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            format!("<!-- {e} -->"),
                        )
                            .into_response();
                    }
                },
                Err(e) => {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        format!("<!-- {e} -->"),
                    )
                        .into_response();
                }
            };
            Html(html + &global_message::with_success("Role deleted.")).into_response()
        }
        Ok(_) => Html(
            list_html
                + &global_message::with_error("Role not found or already deleted."),
        )
        .into_response(),
        Err(e) => {
            Html(list_html + &global_message::with_error(&e.to_string())).into_response()
        }
    }
}

pub async fn load_paginated_roles(
    pool: &Db,
    page: i64,
    size: i64,
    search: &str,
) -> AppResult<PaginatedRoles> {
    let (total, mut roles) = if search.is_empty() {
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM roles")
            .fetch_one(pool)
            .await?;

        let roles = sqlx::query_as::<_, Role>(
            "SELECT role_id, name, description, created_at
             FROM roles ORDER BY created_at DESC
             LIMIT $1 OFFSET $2",
        )
        .bind(size)
        .bind((page - 1) * size)
        .fetch_all(pool)
        .await?;

        (total, roles)
    } else {
        let pattern = format!("%{}%", search);

        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM roles
             WHERE name ILIKE $1 OR description ILIKE $1",
        )
        .bind(&pattern)
        .fetch_one(pool)
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
        .fetch_all(pool)
        .await?;

        (total, roles)
    };

    for role in &mut roles {
        let perms =
            sqlx::query("SELECT resource, access_level FROM role_permissions WHERE role_id = $1")
                .bind(role.role_id)
                .fetch_all(pool)
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

    Ok(PaginatedRoles {
        roles,
        total,
        page,
        size,
        pages,
    })
}

pub async fn list_roles(
    _admin: AdminClaims,
    State(pool): State<Db>,
    Query(params): Query<ListRolesQuery>,
) -> AppResult<Json<PaginatedRoles>> {
    let page = params.page.unwrap_or(1).max(1);
    let size = params.size.unwrap_or(8).clamp(1, 100);
    let search = params.search.unwrap_or_default().trim().to_string();
    let data = load_paginated_roles(&pool, page, size, &search).await?;
    Ok(Json(data))
}

pub async fn fetch_all_role_names(pool: &Db) -> AppResult<Vec<String>> {
    sqlx::query_scalar("SELECT name FROM roles ORDER BY name ASC")
        .fetch_all(pool)
        .await
        .map_err(Into::into)
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
    Ok(Json(load_roles_summary(&pool).await?))
}

pub async fn load_roles_summary(pool: &Db) -> AppResult<RolesSummary> {
    let row = sqlx::query(
        "SELECT
            (SELECT COUNT(*) FROM roles)                AS total_roles,
            COUNT(*)                                    AS total_permissions,
            COUNT(DISTINCT resource)                    AS unique_resources,
            COUNT(*) FILTER (WHERE access_level = 'write') AS write_count,
            COUNT(*) FILTER (WHERE access_level = 'admin') AS admin_count
         FROM role_permissions",
    )
    .fetch_one(pool)
    .await?;

    use sqlx::Row;
    Ok(RolesSummary {
        total_roles: row.get("total_roles"),
        total_permissions: row.get("total_permissions"),
        unique_resources: row.get("unique_resources"),
        write_count: row.get("write_count"),
        admin_count: row.get("admin_count"),
    })
}

pub async fn persist_new_role(pool: &Db, payload: &CreateRoleRequest) -> AppResult<Uuid> {
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
            return Err(AppError::Conflict(format!(
                "A role named '{}' already exists.",
                payload.name
            )));
        }
        Err(e) => return Err(e.into()),
    }

    for perm in &payload.permissions {
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
                return Err(AppError::Conflict(format!(
                    "Duplicate permission: '{}' with '{}' is already assigned to this role.",
                    resource_str, access_str
                )));
            }
            Err(e) => return Err(e.into()),
        }
    }

    tx.commit().await?;
    Ok(role_id)
}


pub async fn assign_role(
    admin: AdminClaims,
    State(pool): State<Db>,
    Form(form): Form<AssignRoleHtmlForm>,
) -> impl IntoResponse {
    let duration_secs = match form
        .duration_hours
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        Some(s) => match s.parse::<i64>() {
            Ok(h) if h > 0 => Some(h * 3600),
            _ => {
                let base = form.redirect.as_deref().unwrap_or("/assign");
                let sep = if base.contains('?') { '&' } else { '?' };
                return Redirect::to(&format!(
                    "{}{}error={}",
                    base,
                    sep,
                    query_param_encode("Invalid duration (hours).")
                ))
                .into_response();
            }
        },
        None => None,
    };

    let payload = AssignRoleRequest {
        email: form.email.trim().to_string(),
        role_name: form.role_name.trim().to_string(),
        duration_secs,
    };

    let res: AppResult<()> = async {
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

        let expires_at = payload
            .duration_secs
            .map(|secs| chrono::Utc::now() + chrono::Duration::seconds(secs));

        sqlx::query(
            "INSERT INTO role_assignments (user_id, role_id, assigned_by, expires_at)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (user_id, role_id)
             DO UPDATE SET expires_at = EXCLUDED.expires_at, assigned_at = NOW()",
        )
        .bind(user_id)
        .bind(role_id)
        .bind(admin.0.sub.parse::<Uuid>().ok())
        .bind(expires_at)
        .execute(&pool)
        .await?;
        Ok(())
    }
    .await;

    let base = form.redirect.as_deref().unwrap_or("/assign");
    let sep = if base.contains('?') { '&' } else { '?' };
    match res {
        Ok(()) => Redirect::to(&format!(
            "{}{}notice={}",
            base,
            sep,
            query_param_encode("Role assigned successfully.")
        ))
        .into_response(),
        Err(e) => {
            let sep_err = if base.contains('?') { '&' } else { '?' };
            Redirect::to(&format!(
                "{}{}error={}",
                base,
                sep_err,
                query_param_encode(&e.to_string())
            ))
            .into_response()
        }
    }
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
    headers: HeaderMap,
    admin: AdminClaims,
    State(pool): State<Db>,
    Path(user_id): Path<Uuid>,
    Form(payload): Form<DisableUserRequest>,
) -> impl IntoResponse {
    let actor_id = match Uuid::parse_str(&admin.0.sub) {
        Ok(v) => v,
        Err(_) => {
            if is_htmx(&headers) {
                return match users_htmx_html(&pool, &admin).await {
                    Ok(body) => Html(
                        body + &global_message::with_error("Unauthorized"),
                    )
                    .into_response(),
                    Err(_) => Redirect::to(&format!(
                        "/users?error={}",
                        query_param_encode("Unauthorized")
                    ))
                    .into_response(),
                };
            }
            return Redirect::to(&format!(
                "/users?error={}",
                query_param_encode("Unauthorized")
            ))
            .into_response();
        }
    };

    if actor_id == user_id {
        if is_htmx(&headers) {
            return match users_htmx_html(&pool, &admin).await {
                Ok(body) => Html(
                    body
                        + &global_message::with_error("You cannot disable your own account."),
                )
                .into_response(),
                Err(_) => Redirect::to(&format!(
                    "/users?error={}",
                    query_param_encode("You cannot disable your own account.")
                ))
                .into_response(),
            };
        }
        return Redirect::to(&format!(
            "/users?error={}",
            query_param_encode("You cannot disable your own account.")
        ))
        .into_response();
    }

    let res: AppResult<()> = async {
        let mut tx = pool.begin().await?;
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
        Ok(())
    }
    .await;

    match res {
        Ok(()) => {
            if is_htmx(&headers) {
                return match users_htmx_html(&pool, &admin).await {
                    Ok(body) => Html(
                        body + &global_message::with_success("User disabled."),
                    )
                    .into_response(),
                    Err(_) => Redirect::to("/users").into_response(),
                };
            }
            Redirect::to("/users").into_response()
        }
        Err(e) => {
            if is_htmx(&headers) {
                return match users_htmx_html(&pool, &admin).await {
                    Ok(body) => Html(
                        body + &global_message::with_error(&e.to_string()),
                    )
                    .into_response(),
                    Err(_) => Redirect::to(&format!(
                        "/users?error={}",
                        query_param_encode(&e.to_string())
                    ))
                    .into_response(),
                };
            }
            Redirect::to(&format!(
                "/users?error={}",
                query_param_encode(&e.to_string())
            ))
            .into_response()
        }
    }
}

pub async fn enable_user(
    headers: HeaderMap,
    admin: AdminClaims,
    State(pool): State<Db>,
    Path(user_id): Path<Uuid>,
) -> impl IntoResponse {
    let actor_id = match Uuid::parse_str(&admin.0.sub) {
        Ok(v) => v,
        Err(_) => {
            if is_htmx(&headers) {
                return match users_htmx_html(&pool, &admin).await {
                    Ok(body) => Html(
                        body + &global_message::with_error("Unauthorized"),
                    )
                    .into_response(),
                    Err(_) => Redirect::to(&format!(
                        "/users?error={}",
                        query_param_encode("Unauthorized")
                    ))
                    .into_response(),
                };
            }
            return Redirect::to(&format!(
                "/users?error={}",
                query_param_encode("Unauthorized")
            ))
            .into_response();
        }
    };

    let res: AppResult<()> = async {
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
        Ok(())
    }
    .await;

    match res {
        Ok(()) => {
            if is_htmx(&headers) {
                return match users_htmx_html(&pool, &admin).await {
                    Ok(body) => Html(
                        body + &global_message::with_success("User enabled."),
                    )
                    .into_response(),
                    Err(_) => Redirect::to("/users").into_response(),
                };
            }
            Redirect::to("/users").into_response()
        }
        Err(e) => {
            if is_htmx(&headers) {
                return match users_htmx_html(&pool, &admin).await {
                    Ok(body) => Html(
                        body + &global_message::with_error(&e.to_string()),
                    )
                    .into_response(),
                    Err(_) => Redirect::to(&format!(
                        "/users?error={}",
                        query_param_encode(&e.to_string())
                    ))
                    .into_response(),
                };
            }
            Redirect::to(&format!(
                "/users?error={}",
                query_param_encode(&e.to_string())
            ))
            .into_response()
        }
    }
}

fn redirect_for_invalid_role_form(headers: &HeaderMap) -> &'static str {
    match headers.get(REFERER).and_then(|v| v.to_str().ok()) {
        Some(referer) if referer.contains("/roles/new") => "/roles/new",
        _ => "/roles/quick",
    }
}

pub async fn create_role_form(
    admin: AdminClaims,
    State(pool): State<Db>,
    headers: HeaderMap,
    form: Result<Form<CreateRoleFormRequest>, FormRejection>,
) -> Response {
    let fallback = redirect_for_invalid_role_form(&headers);

    let form = match form {
        Ok(f) => f.0,
        Err(_) => {
            if is_quick_create_htmx(&headers, None) {
                let view = quick_create_shell(
                    &admin,
                    &empty_quick_create_form(),
                    Some("Invalid form submission. Please try again.".into()),
                    None,
                    None,
                    None,
                    true,
                );
                let html = view.render().unwrap_or_else(|e| format!("<!-- template error: {e} -->"));
                return Html(html).into_response();
            }
            if is_wizard_htmx(&headers, None) {
                let view = wizard_shell(
                    &admin,
                    &empty_wizard_form(),
                    Some("Invalid form submission. Please try again.".into()),
                    None,
                    None,
                    None,
                    true,
                );
                let html = view.render().unwrap_or_else(|e| format!("<!-- template error: {e} -->"));
                return Html(html).into_response();
            }
            return Redirect::to(&format!(
                "{}?error={}",
                fallback,
                query_param_encode("Invalid or incomplete form submission.")
            ))
            .into_response();
        }
    };

    let err_base = form
        .error_redirect
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("/roles/quick");

    let is_quick_htmx = is_quick_create_htmx(&headers, Some(&form));
    let is_wizard_htmx = is_wizard_htmx(&headers, Some(&form));

    let permissions = match validate_and_parse_create_role_form(&form) {
        Ok(p) => p,
        Err(errs) => {
            if is_quick_htmx {
                let view = quick_create_shell(
                    &admin,
                    &form,
                    None,
                    first_field_message(&errs, "name"),
                    first_field_message(&errs, "description"),
                    first_field_message(&errs, "resource"),
                    true,
                );
                let html = view.render().unwrap_or_else(|e| format!("<!-- template error: {e} -->"));
                return Html(html).into_response();
            }
            if is_wizard_htmx {
                let view = wizard_shell(
                    &admin,
                    &form,
                    None,
                    first_field_message(&errs, "name"),
                    first_field_message(&errs, "description"),
                    first_field_message(&errs, "resource"),
                    true,
                );
                let html = view.render().unwrap_or_else(|e| format!("<!-- template error: {e} -->"));
                return Html(html).into_response();
            }
            let msg = first_field_message(&errs, "name")
                .or_else(|| first_field_message(&errs, "description"))
                .or_else(|| first_field_message(&errs, "resource"))
                .unwrap_or_else(|| "Invalid form.".into());
            return Redirect::to(&format!(
                "{}?error={}",
                err_base,
                query_param_encode(&msg)
            ))
            .into_response();
        }
    };

    let req = CreateRoleRequest {
        name: form.name.trim().to_string(),
        description: form.description.trim().to_string(),
        permissions,
    };

    let ok_target = form
        .redirect
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("/roles?notice=created");

    match persist_new_role(&pool, &req).await {
        Ok(_) => {
            if is_quick_htmx || is_wizard_htmx {
                return hx_redirect_response(&ok_target);
            }
            Redirect::to(&ok_target).into_response()
        }
        Err(e) => {
            if is_quick_htmx {
                let view = quick_create_shell(
                    &admin,
                    &form,
                    None,
                    None,
                    None,
                    None,
                    true,
                );
                let mut html = view
                    .render()
                    .unwrap_or_else(|e| format!("<!-- template error: {e} -->"));
                html.push_str(&global_message::with_error(&e.to_string()));
                return Html(html).into_response();
            }
            if is_wizard_htmx {
                let view = wizard_shell(
                    &admin,
                    &form,
                    None,
                    None,
                    None,
                    None,
                    true,
                );
                let mut html = view
                    .render()
                    .unwrap_or_else(|e| format!("<!-- template error: {e} -->"));
                html.push_str(&global_message::with_error(&e.to_string()));
                return Html(html).into_response();
            }
            Redirect::to(&format!(
                "{}?error={}",
                err_base,
                query_param_encode(&e.to_string())
            ))
            .into_response()
        }
    }
}

pub async fn delete_role_submit(
    _admin: AdminClaims,
    State(pool): State<Db>,
    Path(role_id): Path<Uuid>,
) -> impl IntoResponse {
    let res: AppResult<()> = async {
        let result = sqlx::query("DELETE FROM roles WHERE role_id = $1")
            .bind(role_id)
            .execute(&pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(AppError::RoleNotFound);
        }
        Ok(())
    }
    .await;

    match res {
        Ok(()) => Redirect::to("/roles?notice=deleted").into_response(),
        Err(e) => Redirect::to(&format!(
            "/roles?error={}",
            query_param_encode(&e.to_string())
        ))
        .into_response(),
    }
}
