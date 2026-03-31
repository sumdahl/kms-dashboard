use axum::{
    async_trait,
    extract::FromRequestParts,
    http::request::Parts,
    response::{IntoResponse, Response},
};
use crate::error::AppError;
use crate::auth::jwt::verify_jwt;
use crate::models::Claims;

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // 1. Try Authorization header first (API compatibility)
        let token = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .map(String::from)
            // 2. Fallback to cookie
            .or_else(|| {
                parts
                    .headers
                    .get(axum::http::header::COOKIE)
                    .and_then(|v| v.to_str().ok())
                    .and_then(|cookies| {
                        cookies.split(';')
                            .find_map(|c| {
                                let c = c.trim();
                                c.strip_prefix("token=").map(String::from)
                            })
                    })
            })
            .ok_or(AppError::Unauthorized)?;

        let claims = verify_jwt(&token)?;
        Ok(claims)
    }
}

/// Page-aware claims extractor — redirects to login instead of returning JSON
pub struct PageClaims(pub Claims);

#[async_trait]
impl<S> FromRequestParts<S> for PageClaims
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Claims::from_request_parts(parts, state)
            .await
            .map(PageClaims)
            .map_err(|_| {
                Response::builder()
                    .status(302)
                    .header("Location", "/auth/login")
                    .body(axum::body::Body::empty())
                    .unwrap()
            })
    }
}

/// Helper extractor that specifically requires the user to be an admin
pub struct AdminClaims(pub Claims);

#[async_trait]
impl<S> FromRequestParts<S> for AdminClaims
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let claims = Claims::from_request_parts(parts, state).await?;

        if !claims.is_admin {
            return Err(AppError::InsufficientAccess);
        }

        Ok(AdminClaims(claims))
    }
}

/// Build a Set-Cookie header value for auth token
pub fn build_auth_cookie(token: &str) -> String {
    format!(
        "token={}; HttpOnly; SameSite=Lax; Path=/; Max-Age={}",
        token,
        60 * 60 * 24  // 24 hours
    )
}

/// Build a Set-Cookie header value that clears the auth token
pub fn clear_auth_cookie() -> &'static str {
    "token=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0"
}
