use crate::db::Db;
use crate::error::AppResult;
use crate::models::UserSession;
use chrono::{Duration, Utc};
use uuid::Uuid;

pub async fn create_session(
    pool: &Db,
    user_id: Uuid,
    refresh_token: Option<String>,
    ip_address: Option<String>,
    user_agent: Option<String>,
    ttl_hours: i64,
) -> AppResult<UserSession> {
    let now = Utc::now();
    let expires_at = now + Duration::hours(ttl_hours);

    let session = sqlx::query_as!(
        UserSession,
        r#"
        INSERT INTO user_sessions (user_id, refresh_token, ip_address, user_agent, created_at, expires_at, last_activity)
        VALUES ($1, $2, $3::text::inet, $4, $5, $6, $5)
        RETURNING session_id, user_id, refresh_token, ip_address::text as "ip_address: _", user_agent, data, created_at, expires_at, last_activity
        "#,
        user_id,
        refresh_token,
        ip_address,
        user_agent,
        now,
        expires_at
    )
    .fetch_one(pool)
    .await?;

    Ok(session)
}

pub async fn get_session_by_refresh_token(
    pool: &Db,
    refresh_token: &str,
) -> AppResult<Option<UserSession>> {
    let session = sqlx::query_as!(UserSession,r#"
    SELECT session_id, user_id, refresh_token, ip_address::text as "ip_address: _", user_agent, data, created_at, expires_at, last_activity
    FROM user_sessions
    WHERE refresh_token = $1 AND expires_at > NOW()
    "#, refresh_token)
        .fetch_optional(pool)
        .await?;

    Ok(session)
}

pub async fn update_session_activity(pool: &Db, session_id: Uuid) -> AppResult<()> {
    sqlx::query!(
        r#"
        UPDATE user_sessions
        SET last_activity = NOW()
        WHERE session_id = $1
        "#,
        session_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_session(pool: &Db, session_id: Uuid) -> AppResult<()> {
    sqlx::query!(
        r#"
        DELETE FROM user_sessions
        WHERE session_id = $1
        "#,
        session_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_all_user_sessions(pool: &Db, user_id: Uuid) -> AppResult<()> {
    sqlx::query!(
        r#"
        DELETE FROM user_sessions
        WHERE user_id = $1
        "#,
        user_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn count_user_sessions(pool: &Db, user_id: Uuid) -> AppResult<i64> {
    let count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) FROM user_sessions
        WHERE user_id = $1 AND expires_at > NOW()
        "#,
        user_id
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0);

    Ok(count)
}

pub async fn cleanup_expired_sessions(pool: &Db) -> AppResult<()> {
    sqlx::query!("DELETE FROM user_sessions WHERE expires_at < NOW()")
        .execute(pool)
        .await?;

    Ok(())
}
