use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::CookieJar;

use crate::app_state::AppState;
use crate::error::AppError;
use crate::models::Claims;

/// Injected into every page handler that renders layout.html.
/// Carries all variables the layout needs.
#[derive(Debug, Clone)]
pub struct PageContext {
    pub dark_mode: bool,
    pub css_version: &'static str,
    pub sidebar_pinned: bool,
    pub user_email: String,
    pub user_initials: String,
    pub is_admin: bool,
    pub show_banner: bool,
}

#[async_trait]
impl FromRequestParts<AppState> for PageContext {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Requires valid auth
        let claims = Claims::from_request_parts(parts, state).await?;

        let jar = CookieJar::from_request_parts(parts, state)
            .await
            .unwrap_or_default();

        let dark_mode = jar
            .get("theme")
            .map(|c| c.value() == "dark")
            .unwrap_or(false);

        let sidebar_pinned = jar
            .get("sidebar_pinned")
            .map(|c| c.value() == "true")
            .unwrap_or(false);

        let user_email = claims.email.clone();
        let user_initials = first_initial(&user_email);
        let is_admin = claims.is_admin;

        Ok(PageContext {
            dark_mode,
            css_version: env!("CSS_VERSION"),
            sidebar_pinned,
            user_email,
            user_initials,
            is_admin,
            show_banner: false,
        })
    }
}

fn first_initial(email: &str) -> String {
    email
        .chars()
        .next()
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_else(|| "?".to_string())
}
