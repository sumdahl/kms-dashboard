use axum::{
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;
use askama::Template;

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
    #[error("Account disabled: {0:?}")]
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

#[derive(Template)]
#[template(path = "partials/error_banner.html")]
struct ErrorBannerTemplate {
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        if let AppError::AccountDisabled(reason) = self {
            return disabled_response(reason);
        }

        let status = match &self {
            AppError::EmailTaken => StatusCode::CONFLICT,
            AppError::BadCredentials => StatusCode::UNAUTHORIZED,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::TokenExpired => StatusCode::UNAUTHORIZED,
            AppError::TokenRevoked => StatusCode::UNAUTHORIZED,
            AppError::NoPermission => StatusCode::FORBIDDEN,
            AppError::InsufficientAccess => StatusCode::FORBIDDEN,
            AppError::RoleNotFound => StatusCode::NOT_FOUND,
            AppError::UserNotFound => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::AccountDisabled(_) => unreachable!(),
        };

        (status, Json(json!({ "error": self.to_string() }))).into_response()
    }
}

impl AppError {
    /// Detects if the request is an HTMX request and returns a partial HTML banner,
    /// otherwise returns a standard JSON/Redirect response.
    pub fn smart_response(self, headers: &HeaderMap) -> Response {
        if headers.contains_key("HX-Request") {
            return self.htmx_response();
        }
        self.into_response()
    }

    pub fn htmx_response(self) -> Response {
        let status = match &self {
            AppError::EmailTaken => StatusCode::CONFLICT,
            AppError::BadCredentials => StatusCode::UNAUTHORIZED,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::TokenExpired => StatusCode::UNAUTHORIZED,
            AppError::TokenRevoked => StatusCode::UNAUTHORIZED,
            AppError::NoPermission => StatusCode::FORBIDDEN,
            AppError::InsufficientAccess => StatusCode::FORBIDDEN,
            AppError::RoleNotFound => StatusCode::NOT_FOUND,
            AppError::UserNotFound => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::AccountDisabled(_) => StatusCode::FORBIDDEN,
        };

        let message = if let AppError::AccountDisabled(Some(r)) = &self {
            format!("Account disabled: {}", r)
        } else {
            self.to_string()
        };

        let template = ErrorBannerTemplate { message };

        match template.render() {
            Ok(html) => (status, axum::response::Html(html)).into_response(),
            Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Error").into_response(),
        }
    }
}

fn disabled_response(reason: Option<String>) -> Response {
    let mut headers = HeaderMap::new();
    let url = match reason {
        Some(r) => format!("/login?reason=account_disabled&message={}", urlencoding::encode(&r)),
        None => "/login?reason=account_disabled".to_string(),
    };
    
    headers.insert(
        axum::http::header::LOCATION,
        HeaderValue::from_str(&url).unwrap(),
    );
    headers.insert(
        "HX-Redirect",
        HeaderValue::from_str(&url).unwrap(),
    );
    (StatusCode::FOUND, headers).into_response()
}
pub type AppResult<T> = Result<T, AppError>;
