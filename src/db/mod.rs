pub mod seed;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::error::AppError;
use crate::error::AppResult;

pub type Db = PgPool;

pub async fn init_db(database_url: &str) -> Db {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
        .expect("Failed to connect to Postgres")
}

pub async fn run_migrations(pool: &Db) -> AppResult<()> {
    tracing::info!("Running database migrations...");

    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(migration_error)?;

    tracing::info!("✅ Migrations applied successfully");
    Ok(())
}

fn migration_error(e: sqlx::migrate::MigrateError) -> AppError {
    match e {
        sqlx::migrate::MigrateError::VersionMismatch(v) => {
            tracing::error!(
                "\n\n❌ MIGRATION CHECKSUM MISMATCH — version: {v}\n\
                 \n\
                 A migration file was edited after it was already applied.\n\
                 Never edit an applied migration — create a new one instead.\n\
                 \n\
                 Fix (local dev only):\n\
                 \n\
                 \t make db/reset\n\n"
            );
            AppError::Internal(format!(
                "Migration {v} was previously applied but has been modified"
            ))
        }
        sqlx::migrate::MigrateError::VersionMissing(v) => {
            tracing::error!(
                "\n\n❌ MIGRATION FILE MISSING — version: {v}\n\
                 \n\
                 The DB has record of migration {v} but the file no longer exists.\n\
                 Did someone delete a migration file?\n\n"
            );
            AppError::Internal(format!("Migration file for version {v} is missing"))
        }
        e => {
            tracing::error!("Migration failed: {e}");
            AppError::Internal(format!("Migration error: {e}"))
        }
    }
}
