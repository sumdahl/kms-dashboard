/// Role assignment handler.
use super::views::append_query_param;
use crate::app_state::AppState;
use crate::error::{AppError, AppResult};
use crate::middleware::auth::AdminClaims;
use crate::repositories;
use axum::{
    extract::{Form, State},
    response::{IntoResponse, Redirect},
};
use serde::Deserialize;
use uuid::Uuid;

// ── Request types ─────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct AssignRoleRequest {
    pub email: String,
    pub role_name: String,
    pub duration_secs: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct AssignRoleHtmlForm {
    pub email: String,
    pub role_name: String,
    pub duration_hours: Option<String>,
    #[serde(default)]
    pub redirect: Option<String>,
}

// ── Route handler ─────────────────────────────────────────────────────────────

pub async fn assign_role(
    admin: AdminClaims,
    State(state): State<AppState>,
    Form(form): Form<AssignRoleHtmlForm>,
) -> impl IntoResponse {
    let pool = state.db;
    let base = form.redirect.as_deref().unwrap_or("/assign");

    let duration_secs = match form
        .duration_hours
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        Some(s) => match s.parse::<i64>() {
            Ok(h) if h > 0 => Some(h * 3600),
            _ => {
                return Redirect::to(&append_query_param(
                    base,
                    "error",
                    "Invalid duration (hours).",
                ))
                .into_response();
            }
        },
        None => None,
    };

    let payload = AssignRoleRequest {
        email: form.email.trim().to_string(),
        role_name: form.role_name.trim().to_string(),
        duration_secs,
    };

    let res: AppResult<()> = async {
        let user_id = repositories::users::find_id_by_email(&pool, &payload.email)
            .await?
            .ok_or(AppError::UserNotFound)?;

        let role_id = repositories::roles::find_id_by_name(&pool, &payload.role_name)
            .await?
            .ok_or(AppError::RoleNotFound)?;

        let expires_at = payload
            .duration_secs
            .map(|secs| chrono::Utc::now() + chrono::Duration::seconds(secs));

        repositories::assignments::upsert_assignment(
            &pool,
            user_id,
            role_id,
            admin.0.sub.parse::<Uuid>().ok(),
            expires_at,
        )
        .await?;
        Ok(())
    }
    .await;

    match res {
        Ok(()) => Redirect::to(&append_query_param(base, "notice", "assigned")).into_response(),
        Err(e) => Redirect::to(&append_query_param(base, "error", &e.to_string())).into_response(),
    }
}
