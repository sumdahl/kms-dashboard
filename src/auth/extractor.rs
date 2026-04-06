use crate::auth::blocklist::is_token_blocklisted;
use crate::auth::jwt::verify_jwt;
use crate::db::Db;
use crate::error::AppError;
use crate::models::Claims;
use axum::{
    async_trait,
    extract::{FromRequestParts, State},
    http::request::Parts,
};

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync + AsRef<Db>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

        if !auth_header.starts_with("Bearer ") {
            return Err(AppError::Unauthorized);
        }

        let raw_token = &auth_header[7..];

        let claims = verify_jwt(raw_token)?;

        let pool = state.as_ref();
        if is_token_blocklisted(pool, raw_token).await? {
            return Err(AppError::TokenRevoked);
        }

        Ok(claims)
    }
}

pub struct AdminClaims(pub Claims);

#[async_trait]
impl<S> FromRequestParts<S> for AdminClaims
where
    S: Send + Sync + AsRef<Db>,
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
