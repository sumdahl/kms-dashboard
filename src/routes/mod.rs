pub mod auth;
pub mod admin;
pub mod api;

use axum::{
    extract::Form,
    response::Html,
    routing::{delete, get, post},
    Router,
};
use serde::Deserialize;
use crate::app_state::AppState;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Web / UI Routes
        .route("/", get(home))
        .route("/login", get(login_page))
        .route("/signup", get(signup_page))
        .route("/roles", get(roles_page))
        .route("/assign", get(assign_page))
        .route("/ui/sidebar/pin", post(sidebar_pin))
        .route("/ui/banner", delete(banner_dismiss))

        // API Routes
        .nest("/auth", auth::router())
        .nest("/admin", admin::router())
        .nest("/api", api::router())

        .with_state(state)
}

// ── Web Handlers ──

#[derive(askama::Template)]
#[template(path = "login.html")]
struct LoginTemplate {}

async fn login_page() -> impl axum::response::IntoResponse {
    LoginTemplate {}
}

#[derive(askama::Template)]
#[template(path = "signup.html")]
struct SignupTemplate {}

async fn signup_page() -> impl axum::response::IntoResponse {
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

async fn home() -> impl axum::response::IntoResponse {
    HomeTemplate {
        sidebar_pinned: false,
        user_email: String::new(),
        show_banner: true,
        css_version: env!("CSS_VERSION"),
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

async fn roles_page() -> impl axum::response::IntoResponse {
    RolesTemplate {
        sidebar_pinned: false,
        user_email: String::new(),
        show_banner: false,
        css_version: env!("CSS_VERSION"),
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

async fn assign_page() -> impl axum::response::IntoResponse {
    AssignTemplate {
        sidebar_pinned: false,
        user_email: String::new(),
        show_banner: false,
        css_version: env!("CSS_VERSION"),
    }
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
