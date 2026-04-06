use super::Db;
use crate::auth::hashing::hash_password;
use crate::error::AppResult;

pub async fn seed_admin(pool: &Db) -> AppResult<()> {
    let admin_email = "admin@example.com";
    let admin_password = "admin_password";

    let exists = sqlx::query("SELECT user_id FROM users WHERE email = $1")
        .bind(admin_email)
        .fetch_optional(pool)
        .await?;

    if exists.is_none() {
        let hashed = hash_password(admin_password)?;

        sqlx::query(
            "INSERT INTO users (email, full_name, password_hash, is_admin)
             VALUES ($1, $2, $3, $4)",
        )
        .bind(admin_email)
        .bind("System Admin")
        .bind(hashed)
        .bind(true)
        .execute(pool)
        .await?;

        tracing::info!("→ Seeded initial admin: {}", admin_email);
    }

    Ok(())
}
