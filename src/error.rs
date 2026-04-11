use askama::Template;
use axum::{
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Email already registered")]
    EmailTaken,
    #[error("Invalid email or password")]
    BadCredentials,
    #[error("Missing or invalid token")]
    Unauthorized,
    #[error("Token expired")]
    TokenExpired,
    #[error("Token has been revoked")]
    TokenRevoked,
    #[error("No active assignment for this resource")]
    NoPermission,
    #[error("Insufficient access level")]
    InsufficientAccess,
    #[error("Role not found")]
    RoleNotFound,
    #[error("User not found")]
    UserNotFound,
    #[error("{0}")]
    Conflict(String),
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Account disabled")]
    AccountDisabled(Option<String>),
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}

// ── Error page template ──────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "error.html")]
struct ErrorPageTemplate {
    pub dark_mode: bool,
    pub code: u16,
    pub title: String,
    pub message: String,
}

fn render_error_page(status: StatusCode, title: &str, message: &str) -> Response {
    let html = ErrorPageTemplate {
        dark_mode: false,
        code: status.as_u16(),
        title: title.to_string(),
        message: message.to_string(),
    };
    let body = html.render().unwrap_or_else(|_| {
        format!(
            "<html><body><h1>{}</h1><p>{}</p></body></html>",
            title, message
        )
    });
    (status, Html(body)).into_response()
}

// ── Auth redirect (Option A: both headers) ───────────────────────────────────

fn login_redirect(path: &'static str) -> Response {
    let mut headers = HeaderMap::new();
    // Browser redirect
    headers.insert(axum::http::header::LOCATION, HeaderValue::from_static(path));
    // HTMX full-page navigation
    headers.insert("HX-Redirect", HeaderValue::from_static(path));
    (StatusCode::FOUND, headers).into_response()
}

// ── IntoResponse ─────────────────────────────────────────────────────────────

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            // Auth errors → redirect to login (both browser and HTMX)
            AppError::Unauthorized | AppError::TokenExpired | AppError::TokenRevoked => {
                login_redirect("/login")
            }

            AppError::AccountDisabled(_) => login_redirect("/login?reason=account_disabled"),

            // Forbidden → 403 page
            AppError::NoPermission | AppError::InsufficientAccess => render_error_page(
                StatusCode::FORBIDDEN,
                "Access Denied",
                "You don't have permission to access this resource.",
            ),

            // Not found → 404 page
            AppError::RoleNotFound | AppError::UserNotFound => render_error_page(
                StatusCode::NOT_FOUND,
                "Not Found",
                "The resource you're looking for doesn't exist.",
            ),

            AppError::EmailTaken => render_error_page(
                StatusCode::CONFLICT,
                "Email Already Registered",
                "An account with this email already exists.",
            ),

            AppError::Conflict(ref msg) => render_error_page(StatusCode::CONFLICT, "Conflict", msg),

            AppError::BadCredentials => render_error_page(
                StatusCode::UNAUTHORIZED,
                "Invalid Credentials",
                "The email or password you entered is incorrect.",
            ),

            AppError::BadRequest(ref msg) => {
                render_error_page(StatusCode::BAD_REQUEST, "Bad Request", msg)
            }

            AppError::Internal(ref msg) => render_error_page(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal Server Error",
                msg,
            ),
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;
