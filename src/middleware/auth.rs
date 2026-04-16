use crate::app_state::AppState;
use crate::auth::blocklist::is_blocklisted;
use crate::auth::jwt::verify_jwt;
use crate::db::Db;
use crate::error::AppError;
use crate::models::Claims;
use crate::repositories::settings::get_session_config;
use axum::extract::State;
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts, Request},
    http::request::Parts,
    middleware::Next,
    response::Response,
};
use axum_extra::extract::CookieJar;
use sqlx::PgPool;
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
        let jar = CookieJar::from_headers(&parts.headers);
        let pool = Db::from_ref(state);
        let config = get_session_config(&pool).await?;

        let token = jar
            .get("token")
            .map(|c| c.value().to_owned())
            .ok_or(AppError::Unauthorized)?;

        let claims = verify_jwt(&token)?;

        if is_blocklisted(&pool, &claims.jti).await? {
            return Err(AppError::Unauthorized);
        }

        let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;

        let row = sqlx::query(
            "SELECT is_active, disabled_reason, session_version FROM users WHERE user_id = $1",
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
        let disabled_reason: Option<String> = row.get("disabled_reason");
        let session_version: i32 = row.get("session_version");

        if !is_active {
            return Err(AppError::AccountDisabled(disabled_reason));
        }

        if claims.sv != session_version {
            return Err(AppError::Unauthorized);
        }

        use crate::models::AuthStrategy;
        match config.auth_strategy {
            AuthStrategy::Hybrid | AuthStrategy::Session => {
                let session_id = jar.get("sid")
                    .map(|c| c.value())
                    .ok_or(AppError::Unauthorized)?;
                let session_id = Uuid::parse_str(session_id).map_err(|_| AppError::Unauthorized)?;

                let session_exists = sqlx::query_scalar::<_, bool>(
                    "SELECT EXISTS(SELECT 1 FROM user_sessions WHERE session_id = $1 AND user_id = $2 AND expires_at > NOW())"
                )
                .bind(session_id)
                .bind(user_id)
                .fetch_one(&pool)
                .await?;

                if !session_exists {
                    return Err(AppError::Unauthorized);
                }
            }
            AuthStrategy::Jwt => {}
        }

        Ok(claims)
    }
}

#[derive(Clone)]
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
