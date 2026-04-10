pub mod admin;
pub mod api;
pub mod auth;
pub mod logout_partial;

use crate::app_state::AppState;
use crate::handlers::dashboard::{
    assign_page, banner_dismiss, create_role_wizard_page, home, quick_create_role_page,
    role_detail_page, roles_page, sidebar_pin, users_page,
};
use crate::handlers::auth::{login, login_page, signup, signup_page};
use crate::handlers::logout_partial::account_menu;
use crate::handlers::password_reset::{forgot_password_page, reset_password_page};
use crate::middleware::auth::require_admin_mw;
use axum::middleware;
use axum::{
    routing::{delete, get, post},
    Router,
};

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(home))
        .route("/login", get(login_page).post(login))
        .route("/signup", get(signup_page).post(signup))
        .route("/roles", get(roles_page))
        .route("/users", get(users_page))
        .route("/roles/new", get(create_role_wizard_page))
        .route("/roles/quick", get(quick_create_role_page))
        .route("/roles/:name", get(role_detail_page))
        .route("/assign", get(assign_page))
        .route("/ui/sidebar/pin", post(sidebar_pin))
        .route("/ui/banner", delete(banner_dismiss))
        .route("/forgot-password", get(forgot_password_page))
        .route("/reset-password", get(reset_password_page))
        .route(
            "/test-panic",
            get(|| async {
                panic!("Test 500 error");
                #[allow(unreachable_code)]
                axum::http::StatusCode::INTERNAL_SERVER_ERROR
            }),
        )
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

// ── Error pages ──────────────────────────────────────────────────────────

#[derive(askama::Template)]
#[template(path = "error.html")]
struct ErrorTemplate {
    code: u16,
    title: String,
    message: String,
}

pub fn error_page_response(
    code: u16,
    title: &str,
    message: &str,
) -> axum::http::Response<axum::body::Body> {
    use askama::Template;
    use axum::http::{header, StatusCode};

    let template = ErrorTemplate {
        code,
        title: title.to_string(),
        message: message.to_string(),
    };

    let body = template.render().unwrap_or_else(|_| {
        format!(
            "<html><body><h1>{} {}</h1><p>{}</p></body></html>",
            code, title, message
        )
    });

    axum::http::Response::builder()
        .status(StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR))
        .header(header::CONTENT_TYPE, "text/html")
        .body(axum::body::Body::from(body))
        .unwrap()
}

fn error_page(code: u16, title: &str, message: &str) -> axum::http::Response<axum::body::Body> {
    error_page_response(code, title, message)
}

async fn not_found_handler() -> axum::http::Response<axum::body::Body> {
    error_page(
        404,
        "Page Not Found",
        "The page you are looking for does not exist or has been moved.",
    )
}
