use crate::auth::blocklist::is_token_blocklisted;
use crate::auth::jwt::verify_jwt;
use crate::db::Db;
use crate::error::AppError;
use crate::models::Claims;
use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::CookieJar;

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync + AsRef<Db>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Read JWT from HTTP-only cookie
        let jar = CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::Unauthorized)?;

        let token = jar
            .get("token")
            .map(|c| c.value().to_string())
            .ok_or(AppError::Unauthorized)?;

        let claims = verify_jwt(&token)?;

        let pool = state.as_ref();
        if is_token_blocklisted(pool, &token).await? {
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
