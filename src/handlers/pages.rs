use askama::Template;
use crate::render::render;
use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
};
use chrono::Utc;
use serde::Deserialize;

use axum::http::StatusCode;
use axum::response::Html;

use crate::db::Db;
use crate::models::types::{AccessLevel, Resource};
use crate::models::{Role, RolePermission};
use crate::page_context::PageContext;

// ── Home ─────────────────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "dashboard/home.html")]
struct HomeTemplate {
    pub dark_mode: bool,
    pub css_version: &'static str,
    pub sidebar_pinned: bool,
    pub user_email: String,
    pub user_initials: String,
    pub is_admin: bool,
    pub show_banner: bool,
}

pub async fn home(ctx: PageContext) -> Response {
    HomeTemplate {
        dark_mode: ctx.dark_mode,
        css_version: ctx.css_version,
        sidebar_pinned: ctx.sidebar_pinned,
        user_email: ctx.user_email,
        user_initials: ctx.user_initials,
        is_admin: ctx.is_admin,
        show_banner: false,
    }
    .into_response()
}

// ── Users ─────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct UsersQuery {
    pub q: Option<String>,
    pub filter: Option<String>,
}

#[derive(Template)]
#[template(path = "dashboard/users.html")]
struct UsersTemplate {
    pub dark_mode: bool,
    pub css_version: &'static str,
    pub sidebar_pinned: bool,
    pub user_email: String,
    pub user_initials: String,
    pub is_admin: bool,
    pub show_banner: bool,
    // page-specific
    pub query: String,
    pub filter: String,
}

pub async fn users_page(ctx: PageContext, Query(params): Query<UsersQuery>) -> Response {
    UsersTemplate {
        dark_mode: ctx.dark_mode,
        css_version: ctx.css_version,
        sidebar_pinned: ctx.sidebar_pinned,
        user_email: ctx.user_email,
        user_initials: ctx.user_initials,
        is_admin: ctx.is_admin,
        show_banner: false,
        query: params.q.unwrap_or_default(),
        filter: params.filter.unwrap_or_else(|| "all".into()),
    }
    .into_response()
}

// ── Roles ─────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct RolesQuery {
    pub search: Option<String>,
    pub filter: Option<String>,
    pub sort: Option<String>,
}

#[derive(Template)]
#[template(path = "dashboard/roles.html")]
struct RolesTemplate {
    pub dark_mode: bool,
    pub css_version: &'static str,
    pub sidebar_pinned: bool,
    pub user_email: String,
    pub user_initials: String,
    pub is_admin: bool,
    pub show_banner: bool,
    // page-specific
    pub query: String,
    pub filter: String,
    pub sort: String,
}

pub async fn roles_page(ctx: PageContext, Query(params): Query<RolesQuery>) -> Response {
    RolesTemplate {
        dark_mode: ctx.dark_mode,
        css_version: ctx.css_version,
        sidebar_pinned: ctx.sidebar_pinned,
        user_email: ctx.user_email,
        user_initials: ctx.user_initials,
        is_admin: ctx.is_admin,
        show_banner: false,
        query: params.search.unwrap_or_default(),
        filter: params.filter.unwrap_or_else(|| "all".into()),
        sort: params.sort.unwrap_or_else(|| "newest".into()),
    }
    .into_response()
}

// ── Assign ────────────────────────────────────────────────────────────────────

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
    // page-specific
    pub users: Vec<AssignUser>,
    pub roles: Vec<AssignRole>,
    pub form_email: String,
    pub form_role_name: String,
    pub form_duration_hours: String,
    pub global_error: Option<String>,
    pub success: Option<String>,
    pub email_error: String,
    pub role_error: String,
}

pub struct AssignUser {
    pub email: String,
    pub full_name: String,
}

pub struct AssignRole {
    pub name: String,
}

pub async fn assign_page(ctx: PageContext, State(pool): State<Db>) -> Response {
    let users = sqlx::query!(
        "SELECT email, full_name FROM users WHERE is_active = TRUE AND is_admin = FALSE ORDER BY full_name"
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|r| AssignUser {
        email: r.email,
        full_name: r.full_name,
    })
    .collect();

    let roles = sqlx::query!("SELECT name FROM roles ORDER BY name")
        .fetch_all(&pool)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|r| AssignRole { name: r.name })
        .collect();

    AssignTemplate {
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
        success: None,
        email_error: String::new(),
        role_error: String::new(),
    }
    .into_response()
}

// ── Create Role Wizard ────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "dashboard/create_role_wizard.html")]
struct CreateRoleWizardTemplate {
    pub dark_mode: bool,
    pub css_version: &'static str,
    pub sidebar_pinned: bool,
    pub user_email: String,
    pub user_initials: String,
    pub is_admin: bool,
    pub show_banner: bool,
    pub current_step: u8,
    // step 1
    pub form_name: String,
    pub form_description: String,
    pub name_error: String,
    pub error: Option<String>,
}

pub async fn create_role_wizard_page(ctx: PageContext) -> Response {
    CreateRoleWizardTemplate {
        dark_mode: ctx.dark_mode,
        css_version: ctx.css_version,
        sidebar_pinned: ctx.sidebar_pinned,
        user_email: ctx.user_email,
        user_initials: ctx.user_initials,
        is_admin: ctx.is_admin,
        show_banner: false,
        current_step: 1,
        form_name: String::new(),
        form_description: String::new(),
        name_error: String::new(),
        error: None,
    }
    .into_response()
}

// ── Quick Create Role ─────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "dashboard/quick_create_role.html")]
struct QuickCreateRoleTemplate {
    pub dark_mode: bool,
    pub css_version: &'static str,
    pub sidebar_pinned: bool,
    pub user_email: String,
    pub user_initials: String,
    pub is_admin: bool,
    pub show_banner: bool,
    // page-specific
    pub form_name: String,
    pub form_description: String,
    pub global_error: Option<String>,
    pub name_error: String,
    pub description_error: String,
    pub permissions_error: Option<String>,
    pub resources: Vec<&'static str>,
    pub access_levels: Vec<&'static str>,
}

pub async fn quick_create_role_page(ctx: PageContext) -> Response {
    QuickCreateRoleTemplate {
        dark_mode: ctx.dark_mode,
        css_version: ctx.css_version,
        sidebar_pinned: ctx.sidebar_pinned,
        user_email: ctx.user_email,
        user_initials: ctx.user_initials,
        is_admin: ctx.is_admin,
        show_banner: false,
        form_name: String::new(),
        form_description: String::new(),
        global_error: None,
        name_error: String::new(),
        description_error: String::new(),
        permissions_error: None,
        resources: vec!["orders", "customers", "reports", "inventory", "admin_panel"],
        access_levels: vec!["read", "write", "admin"],
    }
    .into_response()
}

// ── Role Detail Page ──────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "dashboard/role_detail.html")]
struct RoleDetailTemplate {
    pub dark_mode: bool,
    pub css_version: &'static str,
    pub sidebar_pinned: bool,
    pub user_email: String,
    pub user_initials: String,
    pub is_admin: bool,
    pub show_banner: bool,
    pub role_name: String,
    pub role_description: String,
    pub role_id_short: String,
    pub created_at_display: String,
    pub permissions_len: usize,
    pub permissions: Vec<(String, String)>,
    pub assignments_len: usize,
    pub assignments: Vec<(String, String, String, bool)>,
}

pub async fn role_detail_page(
    ctx: PageContext,
    State(pool): State<Db>,
    Path(role_name): Path<String>,
) -> Response {
    let mut role = match sqlx::query_as::<_, Role>(
        "SELECT role_id, name, description, created_at FROM roles WHERE name = $1",
    )
    .bind(&role_name)
    .fetch_optional(&pool)
    .await
    {
        Ok(Some(r)) => r,
        _ => {
            return (StatusCode::NOT_FOUND, Html("Not Found".to_string())).into_response();
        }
    };

    let perms =
        sqlx::query("SELECT resource, access_level FROM role_permissions WHERE role_id = $1")
            .bind(role.role_id)
            .fetch_all(&pool)
            .await
            .unwrap_or_default();

    use sqlx::Row;
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

    let permissions: Vec<(String, String)> = role
        .permissions
        .iter()
        .map(|p| (p.resource.to_string(), p.access.to_string()))
        .collect();

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
    .await
    .unwrap_or_default();

    let assignments: Vec<(String, String, String, bool)> = assignment_rows
        .into_iter()
        .map(|r| {
            let email: String = r.get("email");
            let assigned_at: chrono::DateTime<Utc> = r.get("assigned_at");
            let expires_at: Option<chrono::DateTime<Utc>> = r.get("expires_at");
            let is_active: bool = r.get("is_active");
            (
                email,
                assigned_at.format("%b %d, %Y").to_string(),
                expires_at
                    .map(|e| e.format("%b %d, %Y %H:%M").to_string())
                    .unwrap_or_default(),
                is_active,
            )
        })
        .collect();

    let assignments_len = assignments.len();

    RoleDetailTemplate {
        dark_mode: ctx.dark_mode,
        css_version: ctx.css_version,
        sidebar_pinned: ctx.sidebar_pinned,
        user_email: ctx.user_email,
        user_initials: ctx.user_initials,
        is_admin: ctx.is_admin,
        show_banner: false,
        role_name: role.name,
        role_description: role.description,
        role_id_short: role.role_id.to_string().chars().take(8).collect(),
        created_at_display: role.created_at.format("%b %d, %Y").to_string(),
        permissions_len: permissions.len(),
        permissions,
        assignments_len,
        assignments,
    }
    .into_response()
}

// ── Search ────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

#[derive(Template)]
#[template(path = "search.html")]
struct SearchTemplate {
    pub dark_mode: bool,
    pub css_version: &'static str,
    pub sidebar_pinned: bool,
    pub user_email: String,
    pub user_initials: String,
    pub is_admin: bool,
    pub show_banner: bool,
    pub query: String,
}

pub async fn search_page(ctx: PageContext, Query(params): Query<SearchQuery>) -> Response {
    SearchTemplate {
        dark_mode: ctx.dark_mode,
        css_version: ctx.css_version,
        sidebar_pinned: ctx.sidebar_pinned,
        user_email: ctx.user_email,
        user_initials: ctx.user_initials,
        is_admin: ctx.is_admin,
        show_banner: false,
        query: params.q.unwrap_or_default(),
    }
    .into_response()
}
