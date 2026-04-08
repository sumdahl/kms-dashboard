use crate::app_state::AppState;
use crate::auth::blocklist::is_blocklisted;
use crate::auth::jwt::verify_jwt;
use crate::db::Db;
use crate::error::AppError;
use crate::models::Claims;
use axum::extract::State;
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts, Request},
    http::request::Parts,
    middleware::Next,
    response::Response,
};
use axum_extra::extract::CookieJar;
use tracing::error;
use uuid::Uuid;

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

        // 2. Verify JWT signature + expiry
        let claims = verify_jwt(&token)?;

        let pool = Db::from_ref(state);

        // 3. Check token blocklist (handles individual logouts)
        if is_blocklisted(&pool, &claims.jti).await? {
            return Err(AppError::Unauthorized);
        }

        // 4. ✅ Check user is still active + session version matches
        //    This is the account-disable enforcement — one indexed PK lookup
        let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;

        let row = sqlx::query(
            "SELECT is_active, session_version FROM users WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| {
            error!("Auth DB lookup failed: {}", e);
            AppError::Unauthorized
        })?
        .ok_or(AppError::Unauthorized)?;

        use sqlx::Row;
        let is_active: bool = row.get("is_active");
        let session_version: i32 = row.get("session_version");

        if !is_active {
            return Err(AppError::AccountDisabled);
        }

        if claims.sv != session_version {
            // Token is from an old session — silently treat as unauthorized
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
pub async fn require_admin_mw(
    State(state): axum::extract::State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let (mut parts, body) = req.into_parts();
    // Reuse the existing AdminClaims extractor — handles JWT + blocklist + is_admin
    AdminClaims::from_request_parts(&mut parts, &state).await?;
    req = Request::from_parts(parts, body);
    Ok(next.run(req).await)
}
