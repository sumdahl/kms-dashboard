use crate::app_state::AppState;
use crate::error::AppResult;
use crate::middleware::AdminClaims;
use crate::models::SessionConfig;
use crate::repositories::settings;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::Form;
use serde::Deserialize;
use askama::Template;

#[derive(Template)]
#[template(path = "admin/settings.html")]
pub struct AdminSettingsPage {
    pub config: SessionConfig,
    pub message: Option<String>,
    pub error: Option<String>,
    pub css_version: String,
    pub sidebar_pinned: bool,
    pub user_email: String,
    pub nav_active: &'static str,
    pub is_admin: bool,
}

pub async fn get_settings(
    State(state): State<AppState>,
    _admin: AdminClaims,
) -> AppResult<impl IntoResponse> {
    let config = settings::get_session_config(&state.db).await?;
    let view = AdminSettingsPage {
        config,
        message: None,
        error: None,
        css_version: "1".to_string(),
        sidebar_pinned: false,
        user_email: "admin@local".to_string(),
        nav_active: "settings",
        is_admin: true,
    };
    let html = view.render()?;
    Ok((StatusCode::OK, Html(html)))
}

#[derive(Debug, Deserialize)]
pub struct SettingsUpdateForm {
    pub auth_strategy: String,
    pub jwt_access_ttl_minutes: i32,
    pub session_refresh_ttl_hours: i32,
    pub max_concurrent_sessions: i32,
    pub logout_on_browser_close: Option<String>,
    pub force_logout_on_password_change: Option<String>,
    pub ip_restriction_enabled: Option<String>,
    pub remember_me_extension_hours: i32,
}

pub async fn post_settings(
    State(state): State<AppState>,
    admin: AdminClaims,
    Form(form): Form<SettingsUpdateForm>,
) -> AppResult<impl IntoResponse> {
    let user_id = admin.0.sub.parse()?;

    settings::update_setting(&state.db, "auth_strategy", &form.auth_strategy, user_id).await?;
    settings::update_setting(&state.db, "jwt_access_ttl_minutes", &form.jwt_access_ttl_minutes.to_string(), user_id).await?;
    settings::update_setting(&state.db, "session_refresh_ttl_hours", &form.session_refresh_ttl_hours.to_string(), user_id).await?;
    settings::update_setting(&state.db, "max_concurrent_sessions", &form.max_concurrent_sessions.to_string(), user_id).await?;
    settings::update_setting(&state.db, "logout_on_browser_close", &form.logout_on_browser_close.is_some().to_string(), user_id).await?;
    settings::update_setting(&state.db, "force_logout_on_password_change", &form.force_logout_on_password_change.is_some().to_string(), user_id).await?;
    settings::update_setting(&state.db, "ip_restriction_enabled", &form.ip_restriction_enabled.is_some().to_string(), user_id).await?;
    settings::update_setting(&state.db, "remember_me_extension_hours", &form.remember_me_extension_hours.to_string(), user_id).await?;

    let config = settings::get_session_config(&state.db).await?;
    let view = AdminSettingsPage {
        config,
        message: Some("Settings saved successfully".to_string()),
        error: None,
        css_version: "1".to_string(),
        sidebar_pinned: false,
        user_email: "admin@local".to_string(),
        nav_active: "settings",
        is_admin: true,
    };
    let html = view.render()?;
    Ok((StatusCode::OK, Html(html)))
}
