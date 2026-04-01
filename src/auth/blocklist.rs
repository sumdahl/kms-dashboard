use crate::error::AppResult;
use sqlx::PgPool;

pub async fn is_blocklisted(pool: &PgPool, jti: &str) -> AppResult<bool> {
    let row = sqlx::query!(
        "SELECT jti FROM token_blocklist WHERE jti = $1 AND expires_at > NOW()",
        jti
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.is_some())
}

pub async fn blocklist_token(
    pool: &PgPool,
    jti: &str,
    expires_at: chrono::DateTime<chrono::Utc>,
) -> AppResult<()> {
    sqlx::query!(
        "INSERT INTO token_blocklist (jti, expires_at) VALUES ($1, $2)",
        jti,
        expires_at
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn purge_expired_tokens(pool: &PgPool) -> sqlx::Result<u64> {
    let result = sqlx::query!("DELETE FROM token_blocklist WHERE expires_at < NOW()")
        .execute(pool)
        .await?;

    Ok(result.rows_affected())
}
