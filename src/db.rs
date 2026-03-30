use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use crate::error::AppResult;
use crate::auth::hashing::hash_password;

pub type Db = PgPool;

pub async fn init_db(database_url: &str) -> Db {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
        .expect("Failed to connect to Postgres")
}

pub async fn run_migrations(pool: &Db) -> AppResult<()> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|e| crate::error::AppError::Internal(format!("Migration error: {}", e)))?;
    Ok(())
}

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
            "INSERT INTO users (email, full_name, password_hash, is_admin) VALUES ($1, $2, $3, $4)"
        )
        .bind(admin_email)
        .bind("System Admin") // We must provide a name because the column is NOT NULL
        .bind(hashed)
        .bind(true)
        .execute(pool)
        .await?;
        
        println!("→ Seeded initial admin: {}", admin_email);
    }

    Ok(())
}
