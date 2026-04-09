use axum::{
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
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

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        if let AppError::AccountDisabled(ref reason) = self {
            return disabled_response(reason.clone());
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
            AppError::AccountDisabled(_) => unreachable!("handled by early return above"),
        };

        (status, Json(json!({ "error": self.to_string() }))).into_response()
    }
}

fn disabled_response(reason: Option<String>) -> Response {
    let mut headers = HeaderMap::new();
    headers.insert(
        "HX-Redirect",
        HeaderValue::from_static("/login?reason=account_disabled"),
    );
    let message = match &reason {
        Some(r) => format!("Your account has been disabled. Reason: {}", r),
        None => "Your account has been disabled. Contact an administrator.".to_string(),
    };
    let body = Json(json!({ "error": message }));
    (StatusCode::FORBIDDEN, headers, body).into_response()
}
pub type AppResult<T> = Result<T, AppError>;
