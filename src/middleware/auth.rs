use crate::auth::blocklist::is_blocklisted;
use crate::auth::jwt::verify_jwt;
use crate::db::Db;
use crate::error::AppError;
use crate::models::Claims;
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use axum_extra::extract::CookieJar;

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    Db: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // 1. Read token from cookie
        let jar = CookieJar::from_headers(&parts.headers);
        let token = jar
            .get("token")
            .map(|c| c.value().to_owned())
            .ok_or(AppError::Unauthorized)?;

        // 2. Verify JWT
        let claims = verify_jwt(&token)?;

        // 3. Check blocklist
        let pool = Db::from_ref(state);
        if is_blocklisted(&pool, &claims.jti).await? {
            return Err(AppError::Unauthorized);
        }

        Ok(claims)
    }
}

pub struct AdminClaims(pub Claims);

#[async_trait]
impl<S> FromRequestParts<S> for AdminClaims
where
    Db: FromRef<S>,
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
