use axum::{
    async_trait,
    extract::FromRequestParts,
    http::request::Parts,
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
        // 1. Get the Authorization header
        let auth_header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

        // 2. Check if it starts with "Bearer "
        if !auth_header.starts_with("Bearer ") {
            return Err(AppError::Unauthorized);
        }

        // 3. Extract and verify token
        let token = &auth_header[7..];
        let claims = verify_jwt(token)?;

        Ok(claims)
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
