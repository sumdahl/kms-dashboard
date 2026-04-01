pub mod admin;
pub mod api;
pub mod auth;
pub mod logout_partial;

use crate::app_state::AppState;
use crate::handlers::logout_partial::account_menu;
use crate::models::Claims;
use axum::{
    extract::Form,
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
        .route("/assign", get(assign_page))
        .route("/ui/sidebar/pin", post(sidebar_pin))
        .route("/ui/banner", delete(banner_dismiss))
        .route("/account-menu", get(account_menu))
        .nest("/auth", auth::router())
        .nest("/admin", admin::router())
        .nest("/api", api::router())
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
            sidebar_pinned: false,
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
            sidebar_pinned: false,
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
            sidebar_pinned: false,
            user_email: c.email,
            show_banner: false,
            css_version: env!("CSS_VERSION"),
        }
        .into_response(),
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
