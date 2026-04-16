use crate::db::Db;
use crate::error::{AppError, AppResult};
use crate::models::{AppSetting, SessionConfig, AuthStrategy};
use sqlx::Row;
use uuid::Uuid;

pub async fn load_all_settings(pool: &Db) -> AppResult<Vec<AppSetting>> {
    let settings = sqlx::query_as!(
        AppSetting,
        r#"
        SELECT key, value, description, updated_at, updated_by
        FROM app_settings
        ORDER BY key
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(settings)
}

pub async fn get_session_config(pool: &Db) -> AppResult<SessionConfig> {
    load_session_config(pool).await
}

pub async fn load_session_config(pool: &Db) -> AppResult<SessionConfig> {
    let rows = sqlx::query("SELECT key, value FROM app_settings")
        .fetch_all(pool)
        .await?;

    let mut config = SessionConfig {
        auth_strategy: AuthStrategy::Hybrid,
        jwt_access_ttl_minutes: 15,
        session_refresh_ttl_hours: 168,
        max_concurrent_sessions: 5,
        logout_on_browser_close: false,
        force_logout_on_password_change: true,
        ip_restriction_enabled: false,
        remember_me_extension_hours: 72,
    };

    for row in rows {
        let key: String = row.get("key");
        let value: String = row.get("value");

        match key.as_str() {
            "auth_strategy" => config.auth_strategy = AuthStrategy::from(value),
            "jwt_access_ttl_minutes" => config.jwt_access_ttl_minutes = value.parse().unwrap_or(15),
            "session_refresh_ttl_hours" => config.session_refresh_ttl_hours = value.parse().unwrap_or(168),
            "max_concurrent_sessions" => config.max_concurrent_sessions = value.parse().unwrap_or(5),
            "logout_on_browser_close" => config.logout_on_browser_close = value.parse().unwrap_or(false),
            "force_logout_on_password_change" => config.force_logout_on_password_change = value.parse().unwrap_or(true),
            "ip_restriction_enabled" => config.ip_restriction_enabled = value.parse().unwrap_or(false),
            "remember_me_extension_hours" => config.remember_me_extension_hours = value.parse().unwrap_or(72),
            _ => {}
        }
    }

    Ok(config)
}

pub async fn update_setting(pool: &Db, key: &str, value: &str, updated_by: Uuid) -> AppResult<()> {
    sqlx::query!(
        r#"
        INSERT INTO app_settings (key, value, updated_at, updated_by)
        VALUES ($1, $2, NOW(), $3)
        ON CONFLICT (key) DO UPDATE
        SET value = $2, updated_at = NOW(), updated_by = $3
        "#,
        key,
        value,
        updated_by
    )
    .execute(pool)
    .await?;

    Ok(())
}
