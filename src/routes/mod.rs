pub mod admin;
pub mod api;
pub mod auth;
pub mod dashboard;
use crate::app_state::AppState;
use crate::middleware::auth::require_admin_mw;
use axum::middleware;
use axum::{
    extract::{Form, Query},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{delete, get, post},
    Router,
};
use std::collections::HashMap;

use serde::Deserialize;

use crate::handlers::auth::signup_page;
use crate::handlers::password_reset::{
    forgot_password_page, forgot_password_verify_page, reset_password_page,
};
use crate::ui::global_message;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .merge(dashboard::router())
        .route("/login", get(login_page))
        .route("/signup", get(signup_page))
        .route("/ui/sidebar/pin", post(sidebar_pin))
        .route("/ui/global-message", get(get_global_message_oob))
        .route("/ui/global-message/ping", delete(global_message_ping))
        .route("/forgot-password", get(forgot_password_page))
        .route("/forgot-password/verify", get(forgot_password_verify_page))
        .route("/reset-password", get(reset_password_page))
        //to simulate 500 server error.
        .route(
            "/test-panic",
            get(|| async {
                panic!("Test 500 error");
                #[allow(unreachable_code)]
                axum::http::StatusCode::INTERNAL_SERVER_ERROR
            }),
        )
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

async fn login_page(Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    use crate::handlers::auth::LoginView;
    use askama_axum::IntoResponse;

    let account_disabled = params
        .get("reason")
        .map(|r| r == "account_disabled")
        .unwrap_or(false);

    LoginView::page(account_disabled).into_response()
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

pub(crate) fn error_page(code: u16, title: &str, message: &str) -> impl IntoResponse {
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
    #[serde(rename = "pinned")]
    pub _pinned: String,
}

async fn sidebar_pin(Form(_form): Form<SidebarPinForm>) -> Html<&'static str> {
    Html("")
}

#[derive(Deserialize)]
struct GlobalMessageQuery {
    message: String,
    kind: Option<String>,
}

async fn get_global_message_oob(Query(q): Query<GlobalMessageQuery>) -> impl IntoResponse {
    let msg = q.message.trim();
    if msg.is_empty() || msg.len() > 500 {
        return StatusCode::BAD_REQUEST.into_response();
    }
    Html(global_message::from_query_kind(msg, q.kind.as_deref())).into_response()
}

async fn global_message_ping() -> StatusCode {
    StatusCode::NO_CONTENT
}
