pub mod admin;
pub mod api;
pub mod auth;
pub mod logout_partial;

use crate::app_state::AppState;
use crate::db::Db;
use crate::handlers::logout_partial::account_menu;
use crate::middleware::auth::require_admin_mw;
use crate::models::types::{AccessLevel, Resource};
use crate::models::{Claims, Role, RolePermission};
use axum::middleware;
use axum::{
    extract::{Form, Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    routing::{delete, get, post},
    Router,
};

use serde::Deserialize;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(home))
        .route("/login", get(login_page))
        .route("/signup", get(signup_page))
        .route("/roles", get(roles_page))
        .route("/roles/:name", get(role_detail_page))
        .route("/assign", get(assign_page))
        .route("/ui/sidebar/pin", post(sidebar_pin))
        .route("/ui/banner", delete(banner_dismiss))
        .route("/account-menu", get(account_menu))
        .nest("/auth", auth::router())
        .nest(
            "/admin",
            admin::router().layer(middleware::from_fn_with_state(
                state.clone(),
                require_admin_mw,
            )),
        )
        .nest("/api", api::router())
        .fallback(not_found_handler)
        .with_state(state)
}

#[derive(askama::Template)]
#[template(path = "login.html")]
struct LoginTemplate {}

async fn login_page() -> impl IntoResponse {
    LoginTemplate {}
}

#[derive(askama::Template)]
#[template(path = "signup.html")]
struct SignupTemplate {}

async fn signup_page() -> impl IntoResponse {
    SignupTemplate {}
}

#[derive(askama::Template)]
#[template(path = "dashboard/home.html")]
struct HomeTemplate {
    sidebar_pinned: bool,
    user_email: String,
    show_banner: bool,
    css_version: &'static str,
}

async fn home(claims: Option<Claims>) -> Response {
    match claims {
        None => Redirect::to("/login").into_response(),
        Some(c) => HomeTemplate {
            sidebar_pinned: true,
            user_email: c.email,
            show_banner: true,
            css_version: env!("CSS_VERSION"),
        }
        .into_response(),
    }
}

#[derive(askama::Template)]
#[template(path = "dashboard/roles.html")]
struct RolesTemplate {
    sidebar_pinned: bool,
    user_email: String,
    show_banner: bool,
    css_version: &'static str,
}

async fn roles_page(claims: Option<Claims>) -> Response {
    match claims {
        None => Redirect::to("/login").into_response(),
        Some(c) => RolesTemplate {
            sidebar_pinned: true,
            user_email: c.email,
            show_banner: false,
            css_version: env!("CSS_VERSION"),
        }
        .into_response(),
    }
}

#[derive(askama::Template)]
#[template(path = "dashboard/assign.html")]
struct AssignTemplate {
    sidebar_pinned: bool,
    user_email: String,
    show_banner: bool,
    css_version: &'static str,
}

async fn assign_page(claims: Option<Claims>) -> Response {
    match claims {
        None => Redirect::to("/login").into_response(),
        Some(c) => AssignTemplate {
            sidebar_pinned: true,
            user_email: c.email,
            show_banner: false,
            css_version: env!("CSS_VERSION"),
        }
        .into_response(),
    }
}

// ── Role Detail Page ─────────────────────────────────────────────────────

#[derive(askama::Template)]
#[template(path = "dashboard/role_detail.html")]
struct RoleDetailTemplate {
    sidebar_pinned: bool,
    user_email: String,
    show_banner: bool,
    css_version: &'static str,
    role_name: String,
    role_description: String,
    role_id_short: String,
    created_at_display: String,
    permissions_len: usize,
    permissions: Vec<(String, String)>,
    assignments_len: usize,
    assignments: Vec<(String, String, String, bool)>, // (email, assigned_display, expires_display, is_active)
}

async fn role_detail_page(
    claims: Option<Claims>,
    State(pool): State<Db>,
    Path(role_name): Path<String>,
) -> Response {
    let c = match claims {
        None => return Redirect::to("/login").into_response(),
        Some(c) => c,
    };

    // Fetch role
    let mut role = match sqlx::query_as::<_, Role>(
        "SELECT role_id, name, description, created_at FROM roles WHERE name = $1",
    )
    .bind(&role_name)
    .fetch_optional(&pool)
    .await
    {
        Ok(Some(r)) => r,
        _ => {
            return error_page(
                404,
                "Not Found",
                "The role you are looking for does not exist.",
            )
            .into_response()
        }
    };

    // Load permissions
    let perms =
        match sqlx::query("SELECT resource, access_level FROM role_permissions WHERE role_id = $1")
            .bind(role.role_id)
            .fetch_all(&pool)
            .await
        {
            Ok(rows) => rows,
            Err(_) => vec![],
        };

    use sqlx::Row;
    role.permissions = perms
        .into_iter()
        .map(|p| {
            let res_str: String = p.get("resource");
            let acc_str: String = p.get("access_level");
            let resource = serde_json::from_value(serde_json::Value::String(res_str.clone()))
                .unwrap_or(Resource::Orders);
            let access = serde_json::from_value(serde_json::Value::String(acc_str.clone()))
                .unwrap_or(AccessLevel::Read);
            RolePermission { resource, access }
        })
        .collect();

    let permissions: Vec<(String, String)> = role
        .permissions
        .iter()
        .map(|p| {
            let res = serde_json::to_value(&p.resource)
                .ok()
                .and_then(|v| v.as_str().map(String::from))
                .unwrap_or_default();
            let acc = serde_json::to_value(&p.access)
                .ok()
                .and_then(|v| v.as_str().map(String::from))
                .unwrap_or_default();
            (res, acc)
        })
        .collect();

    // Load assignments
    let assignment_rows = match sqlx::query(
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
    {
        Ok(rows) => rows,
        Err(_) => vec![],
    };

    let assignments: Vec<(String, String, String, bool)> = assignment_rows
        .into_iter()
        .map(|r| {
            let email: String = r.get("email");
            let assigned_at: chrono::DateTime<chrono::Utc> = r.get("assigned_at");
            let expires_at: Option<chrono::DateTime<chrono::Utc>> = r.get("expires_at");
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
        sidebar_pinned: true,
        user_email: c.email,
        show_banner: false,
        css_version: env!("CSS_VERSION"),
        role_name: role.name,
        role_description: role.description,
        role_id_short: role.role_id.to_string()[..8].to_string(),
        created_at_display: role.created_at.format("%b %d, %Y").to_string(),
        permissions_len: permissions.len(),
        permissions,
        assignments_len,
        assignments,
    }
    .into_response()
}

// ── Error pages ──────────────────────────────────────────────────────────

#[derive(askama::Template)]
#[template(path = "error.html")]
struct ErrorTemplate {
    code: u16,
    title: String,
    message: String,
}

fn error_page(code: u16, title: &str, message: &str) -> impl IntoResponse {
    (
        StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
        ErrorTemplate {
            code,
            title: title.to_string(),
            message: message.to_string(),
        },
    )
}

async fn not_found_handler() -> impl IntoResponse {
    error_page(
        404,
        "Page Not Found",
        "The page you are looking for does not exist or has been moved.",
    )
}

#[derive(Deserialize)]
pub struct SidebarPinForm {
    pub pinned: String,
}

async fn sidebar_pin(Form(_form): Form<SidebarPinForm>) -> Html<&'static str> {
    Html("")
}

async fn banner_dismiss() -> Html<&'static str> {
    Html("")
}
