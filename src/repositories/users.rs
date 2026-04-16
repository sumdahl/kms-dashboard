use crate::db::Db;
use crate::error::AppResult;
use serde::Serialize;
use uuid::Uuid;

// ── Data types ────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, sqlx::FromRow, Clone)]
pub struct UserSummary {
    pub user_id: Uuid,
    pub email: String,
    pub full_name: String,
    pub is_admin: bool,
    pub is_active: bool,
    pub disabled_reason: Option<String>,
}

// ── Query functions ───────────────────────────────────────────────────────────

/// All non-admin users ordered by creation date descending.
pub async fn fetch_user_summaries(pool: &Db) -> AppResult<Vec<UserSummary>> {
    sqlx::query_as::<_, UserSummary>(
        "SELECT user_id, email, full_name, is_admin, is_active, disabled_reason
         FROM users
         WHERE is_admin = FALSE
         ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

/// Count of admin users.
pub async fn count_admins(pool: &Db) -> AppResult<i64> {
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*)::bigint FROM users WHERE is_admin = TRUE")
        .fetch_one(pool)
        .await
        .map_err(Into::into)
}

/// Find user_id by email address.
pub async fn find_id_by_email(pool: &Db, email: &str) -> AppResult<Option<Uuid>> {
    use sqlx::Row;
    let row = sqlx::query("SELECT user_id FROM users WHERE email = $1")
        .bind(email)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|r| r.get("user_id")))
}
