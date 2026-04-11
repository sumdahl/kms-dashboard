use askama::Template;
use crate::render::render;
use axum::{
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
    Form,
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use chrono::Utc;
use serde::Deserialize;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::db::Db;
use crate::error::{AppError, AppResult};
use crate::handlers::partials::{
    initials_pub, render_user_row, DisableModalTemplate, DisableModalUser, UserRow,
    UsersStatsTemplate, WizardStep1Template,
};
use crate::models::Claims;
use crate::page_context::PageContext;

// ── Theme Toggle ──────────────────────────────────────────────────────────────

pub async fn theme_toggle(jar: CookieJar) -> impl IntoResponse {
    let current = jar
        .get("theme")
        .map(|c| c.value().to_string())
        .unwrap_or_else(|| "light".to_string());

    let new_theme = if current == "dark" { "light" } else { "dark" };

    let cookie = Cookie::build(("theme", new_theme.to_string()))
        .path("/")
        .same_site(SameSite::Lax)
        .build();

    (StatusCode::OK, jar.add(cookie))
}

// ── Sidebar Pin ───────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct SidebarPinForm {
    pub pinned: String,
}

pub async fn sidebar_pin(jar: CookieJar, Form(form): Form<SidebarPinForm>) -> impl IntoResponse {
    let cookie = Cookie::build(("sidebar_pinned", form.pinned))
        .path("/")
        .same_site(SameSite::Lax)
        .build();

    (StatusCode::OK, jar.add(cookie))
}

// ── Disable User ──────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct DisableUserForm {
    pub reason: Option<String>,
}

pub async fn disable_user(
    State(pool): State<Db>,
    claims: Claims,
    Path(user_id): Path<Uuid>,
    Form(form): Form<DisableUserForm>,
) -> AppResult<Response> {
    let actor_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;

    if actor_id == user_id {
        return Ok(disable_modal_error(
            &pool,
            user_id,
            "You cannot disable your own account.".into(),
        )
        .await);
    }

    let reason = form.reason.filter(|r| !r.trim().is_empty());

    let mut tx = pool.begin().await?;

    let updated = sqlx::query(
        r#"UPDATE users
           SET is_active = FALSE,
               session_version = session_version + 1,
               disabled_at = NOW(),
               disabled_by = $1,
               disabled_reason = $2
           WHERE user_id = $3 AND is_active = TRUE
           RETURNING user_id"#,
    )
    .bind(actor_id)
    .bind(&reason)
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await?;

    if updated.is_none() {
        tx.rollback().await?;
        return Ok(disable_modal_error(
            &pool,
            user_id,
            "User not found or already disabled.".into(),
        )
        .await);
    }

    sqlx::query(
        "INSERT INTO user_audit_log (target_user_id, actor_id, action, reason)
         VALUES ($1, $2, 'disabled', $3)",
    )
    .bind(user_id)
    .bind(actor_id)
    .bind(&reason)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let r = sqlx::query!(
        "SELECT user_id, email, full_name, is_active, is_admin, disabled_reason
         FROM users WHERE user_id = $1",
        user_id
    )
    .fetch_one(&pool)
    .await?;

    let user = UserRow {
        user_id: r.user_id.to_string(),
        initials: initials_pub(&r.full_name),
        full_name: r.full_name,
        email: r.email,
        is_active: r.is_active,
        is_admin: r.is_admin,
        disabled_reason: r.disabled_reason,
    };

    let row_html = render_user_row(user);
    let stats = build_stats_oob(&pool).await;
    Ok(Html(format!("{}{}", row_html, stats)).into_response())
}

pub async fn disable_modal_error(pool: &Db, user_id: Uuid, error: String) -> Response {
    let r = sqlx::query!(
        "SELECT user_id, email FROM users WHERE user_id = $1",
        user_id
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    match r {
        Some(r) => DisableModalTemplate {
            user: DisableModalUser {
                user_id: r.user_id.to_string(),
                email: r.email,
            },
            error: Some(error),
            reason_error: String::new(),
        }
        .into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

// ── Enable User ───────────────────────────────────────────────────────────────

pub async fn enable_user(
    State(pool): State<Db>,
    claims: Claims,
    Path(user_id): Path<Uuid>,
) -> AppResult<Response> {
    let actor_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;

    let mut tx = pool.begin().await?;

    let updated = sqlx::query(
        r#"UPDATE users
           SET is_active = TRUE,
               disabled_at = NULL,
               disabled_by = NULL,
               disabled_reason = NULL
           WHERE user_id = $1 AND is_active = FALSE
           RETURNING user_id"#,
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
        "INSERT INTO user_audit_log (target_user_id, actor_id, action)
         VALUES ($1, $2, 'enabled')",
    )
    .bind(user_id)
    .bind(actor_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let r = sqlx::query!(
        "SELECT user_id, email, full_name, is_active, is_admin, disabled_reason
         FROM users WHERE user_id = $1",
        user_id
    )
    .fetch_one(&pool)
    .await?;

    let user = UserRow {
        user_id: r.user_id.to_string(),
        initials: initials_pub(&r.full_name),
        full_name: r.full_name,
        email: r.email,
        is_active: r.is_active,
        is_admin: r.is_admin,
        disabled_reason: r.disabled_reason,
    };

    let row_html = render_user_row(user);
    let stats = build_stats_oob(&pool).await;
    Ok(Html(format!("{}{}", row_html, stats)).into_response())
}

// ── Delete Role ───────────────────────────────────────────────────────────────

pub async fn delete_role(State(pool): State<Db>, Path(role_id): Path<Uuid>) -> AppResult<Response> {
    let result = sqlx::query("DELETE FROM roles WHERE role_id = $1")
        .bind(role_id)
        .execute(&pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::RoleNotFound);
    }

    Ok((StatusCode::OK, Html(String::new())).into_response())
}

// ── Assign Role ───────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct AssignRoleForm {
    pub email: String,
    pub role_name: String,
    pub duration_hours: Option<String>,
}

#[derive(Template)]
#[template(path = "dashboard/assign.html")]
struct AssignTemplate {
    pub dark_mode: bool,
    pub css_version: &'static str,
    pub sidebar_pinned: bool,
    pub user_email: String,
    pub user_initials: String,
    pub is_admin: bool,
    pub show_banner: bool,
    pub users: Vec<crate::handlers::pages::AssignUser>,
    pub roles: Vec<crate::handlers::pages::AssignRole>,
    pub form_email: String,
    pub form_role_name: String,
    pub form_duration_hours: String,
    pub global_error: Option<String>,
    pub success: Option<String>,
    pub email_error: String,
    pub role_error: String,
}

pub async fn assign_role(
    ctx: PageContext,
    State(pool): State<Db>,
    claims: Claims,
    Form(form): Form<AssignRoleForm>,
) -> AppResult<Response> {
    let mut email_error = String::new();
    let mut role_error = String::new();

    if form.email.trim().is_empty() {
        email_error = "Please select a user.".into();
    }
    if form.role_name.trim().is_empty() {
        role_error = "Please select a role.".into();
    }

    let users = load_assign_users(&pool).await;
    let roles = load_assign_roles(&pool).await;

    if !email_error.is_empty() || !role_error.is_empty() {
        return Ok(AssignTemplate {
            dark_mode: ctx.dark_mode,
            css_version: ctx.css_version,
            sidebar_pinned: ctx.sidebar_pinned,
            user_email: ctx.user_email,
            user_initials: ctx.user_initials,
            is_admin: ctx.is_admin,
            show_banner: false,
            users,
            roles,
            form_email: form.email,
            form_role_name: form.role_name,
            form_duration_hours: String::new(),
            global_error: None,
            success: None,
            email_error,
            role_error,
        }
        .into_response());
    }

    let duration_secs = form
        .duration_hours
        .as_deref()
        .filter(|s| !s.is_empty())
        .and_then(|s| s.parse::<i64>().ok())
        .map(|h| h * 3600);

    use sqlx::Row;

    let user_row = sqlx::query("SELECT user_id FROM users WHERE email = $1")
        .bind(form.email.trim())
        .fetch_optional(&pool)
        .await?
        .ok_or(AppError::UserNotFound)?;
    let user_id: Uuid = user_row.get("user_id");

    let role_row = sqlx::query("SELECT role_id FROM roles WHERE name = $1")
        .bind(form.role_name.trim())
        .fetch_optional(&pool)
        .await?
        .ok_or(AppError::RoleNotFound)?;
    let role_id: Uuid = role_row.get("role_id");

    let actor_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;
    let expires_at = duration_secs.map(|secs| Utc::now() + chrono::Duration::seconds(secs));

    sqlx::query(
        r#"INSERT INTO role_assignments (user_id, role_id, assigned_by, expires_at)
           VALUES ($1, $2, $3, $4)
           ON CONFLICT (user_id, role_id)
           DO UPDATE SET expires_at = EXCLUDED.expires_at, assigned_at = NOW()"#,
    )
    .bind(user_id)
    .bind(role_id)
    .bind(actor_id)
    .bind(expires_at)
    .execute(&pool)
    .await?;

    Ok(AssignTemplate {
        dark_mode: ctx.dark_mode,
        css_version: ctx.css_version,
        sidebar_pinned: ctx.sidebar_pinned,
        user_email: ctx.user_email,
        user_initials: ctx.user_initials,
        is_admin: ctx.is_admin,
        show_banner: false,
        users,
        roles,
        form_email: String::new(),
        form_role_name: String::new(),
        form_duration_hours: String::new(),
        global_error: None,
        success: Some(format!(
            "Role '{}' assigned to {} successfully.",
            form.role_name.trim(),
            form.email.trim()
        )),
        email_error: String::new(),
        role_error: String::new(),
    }
    .into_response())
}

// ── Create Role ───────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CreateRoleForm {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "resource[]")]
    pub resources: Vec<String>,
    #[serde(rename = "access[]")]
    pub accesses: Vec<String>,
}

pub async fn create_role(
    State(pool): State<Db>,
    Form(form): Form<CreateRoleForm>,
) -> AppResult<Response> {
    let name = form.name.trim().to_string();
    let description = form.description.unwrap_or_default();

    if name.is_empty() {
        let mut headers = HeaderMap::new();
        headers.insert("HX-Redirect", HeaderValue::from_static("/roles/quick"));
        return Ok((StatusCode::SEE_OTHER, headers).into_response());
    }

    let exists = sqlx::query("SELECT role_id FROM roles WHERE name = $1")
        .bind(&name)
        .fetch_optional(&pool)
        .await?;

    if exists.is_some() {
        let mut headers = HeaderMap::new();
        headers.insert("HX-Redirect", HeaderValue::from_static("/roles/quick"));
        return Ok((StatusCode::SEE_OTHER, headers).into_response());
    }

    let role_id = Uuid::new_v4();
    let mut tx = pool.begin().await?;

    sqlx::query("INSERT INTO roles (role_id, name, description) VALUES ($1, $2, $3)")
        .bind(role_id)
        .bind(&name)
        .bind(&description)
        .execute(&mut *tx)
        .await?;

    for (resource, access) in form.resources.iter().zip(form.accesses.iter()) {
        sqlx::query(
            "INSERT INTO role_permissions (role_id, resource, access_level)
             VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
        )
        .bind(role_id)
        .bind(resource)
        .bind(access)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    let mut headers = HeaderMap::new();
    headers.insert("HX-Redirect", HeaderValue::from_static("/roles"));
    Ok((StatusCode::OK, headers).into_response())
}

// ── Wizard Step 2 ─────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct WizardStep1Form {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Template)]
#[template(path = "partials/wizard/step2.html")]
struct WizardStep2Template {
    pub form_name: String,
    pub form_description: String,
    pub permissions_error: Option<String>,
    pub resources: Vec<&'static str>,
    pub access_levels: Vec<&'static str>,
}

pub async fn wizard_step2(Form(form): Form<WizardStep1Form>) -> Response {
    if form.name.trim().is_empty() {
        return WizardStep1Template {
            form_name: form.name,
            form_description: form.description.unwrap_or_default(),
            name_error: "Role name is required.".into(),
            error: None,
        }
        .into_response();
    }

    WizardStep2Template {
        form_name: form.name,
        form_description: form.description.unwrap_or_default(),
        permissions_error: None,
        resources: vec!["orders", "customers", "reports", "inventory", "admin_panel"],
        access_levels: vec!["read", "write", "admin"],
    }
    .into_response()
}

// ── Wizard Step 3 ─────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct WizardStep2Form {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "resource[]")]
    pub resources: Vec<String>,
    #[serde(rename = "access[]")]
    pub accesses: Vec<String>,
}

pub struct WizardPerm {
    pub resource: String,
    pub access: String,
}

#[derive(Template)]
#[template(path = "partials/wizard/step3.html")]
struct WizardStep3Template {
    pub form_name: String,
    pub form_description: String,
    pub form_permissions: Vec<WizardPerm>,
    pub error: Option<String>,
}

pub async fn wizard_step3(Form(form): Form<WizardStep2Form>) -> Response {
    if form.resources.is_empty() {
        return WizardStep2Template {
            form_name: form.name,
            form_description: form.description.unwrap_or_default(),
            permissions_error: Some("At least one permission is required.".into()),
            resources: vec!["orders", "customers", "reports", "inventory", "admin_panel"],
            access_levels: vec!["read", "write", "admin"],
        }
        .into_response();
    }

    let form_permissions = form
        .resources
        .into_iter()
        .zip(form.accesses.into_iter())
        .map(|(resource, access)| WizardPerm { resource, access })
        .collect();

    WizardStep3Template {
        form_name: form.name,
        form_description: form.description.unwrap_or_default(),
        form_permissions,
        error: None,
    }
    .into_response()
}

// ── Wizard back to step1 ──────────────────────────────────────────────────────

pub async fn wizard_step1_back() -> impl IntoResponse {
    WizardStep1Template {
        form_name: String::new(),
        form_description: String::new(),
        name_error: String::new(),
        error: None,
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

pub async fn build_stats_oob(pool: &Db) -> String {
    let row = sqlx::query!(
        r#"SELECT
            COUNT(*) FILTER (WHERE is_admin = FALSE) AS "total!",
            COUNT(*) FILTER (WHERE is_admin = FALSE AND is_active = TRUE) AS "active!",
            COUNT(*) FILTER (WHERE is_admin = FALSE AND is_active = FALSE) AS "disabled!",
            COUNT(*) FILTER (WHERE is_admin = TRUE) AS "admins!"
           FROM users"#
    )
    .fetch_one(pool)
    .await;

    match row {
        Ok(r) => {
            let html = UsersStatsTemplate {
                total: r.total,
                active: r.active,
                disabled: r.disabled,
                admins: r.admins,
                summary_date: Utc::now().format("%B %d").to_string(),
            }
            .render()
            .unwrap_or_default();
            format!(r#"<div id="users-stats" hx-swap-oob="true">{}</div>"#, html)
        }
        Err(_) => String::new(),
    }
}

pub async fn load_assign_users(pool: &Db) -> Vec<crate::handlers::pages::AssignUser> {
    sqlx::query!(
        "SELECT email, full_name FROM users WHERE is_active = TRUE AND is_admin = FALSE ORDER BY full_name"
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|r| crate::handlers::pages::AssignUser {
        email: r.email,
        full_name: r.full_name,
    })
    .collect()
}

pub async fn load_assign_roles(pool: &Db) -> Vec<crate::handlers::pages::AssignRole> {
    sqlx::query!("SELECT name FROM roles ORDER BY name")
        .fetch_all(pool)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|r| crate::handlers::pages::AssignRole { name: r.name })
        .collect()
}
