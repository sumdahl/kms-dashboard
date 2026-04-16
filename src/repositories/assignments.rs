use crate::db::Db;
use crate::error::AppResult;
use chrono::{DateTime, Utc};
use uuid::Uuid;

// ── Data types ────────────────────────────────────────────────────────────────

pub struct AssignmentWithUser {
    pub email: String,
    pub assigned_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

// ── Query functions ───────────────────────────────────────────────────────────

/// Insert or update a role assignment.
pub async fn upsert_assignment(
    pool: &Db,
    user_id: Uuid,
    role_id: Uuid,
    assigned_by: Option<Uuid>,
    expires_at: Option<DateTime<Utc>>,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO role_assignments (user_id, role_id, assigned_by, expires_at)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (user_id, role_id)
         DO UPDATE SET expires_at = EXCLUDED.expires_at, assigned_at = NOW()",
    )
    .bind(user_id)
    .bind(role_id)
    .bind(assigned_by)
    .bind(expires_at)
    .execute(pool)
    .await?;
    Ok(())
}

/// All assignments for a role with joined user emails, ordered by assigned_at desc.
pub async fn find_by_role_with_users(
    pool: &Db,
    role_id: Uuid,
) -> AppResult<Vec<AssignmentWithUser>> {
    use sqlx::Row;
    let rows = sqlx::query(
        "SELECT u.email, ra.assigned_at, ra.expires_at,
                CASE WHEN ra.expires_at IS NULL OR ra.expires_at > NOW() THEN true ELSE false END AS is_active
         FROM role_assignments ra
         JOIN users u ON u.user_id = ra.user_id
         WHERE ra.role_id = $1
         ORDER BY ra.assigned_at DESC",
    )
    .bind(role_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| AssignmentWithUser {
            email: r.get("email"),
            assigned_at: r.get("assigned_at"),
            expires_at: r.get("expires_at"),
            is_active: r.get("is_active"),
        })
        .collect())
}
