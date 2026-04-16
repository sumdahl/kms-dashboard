/// User management view types and HTMX handlers.
use super::views::{is_htmx, query_param_encode};
use crate::app_state::AppState;
use crate::db::Db;
use crate::error::{AppError, AppResult};
use crate::middleware::auth::AdminClaims;
use crate::repositories;
use crate::repositories::users::UserSummary;
use crate::ui::global_message;
use askama::Template;
use axum::{
    extract::{Form, Path, State},
    http::HeaderMap,
    response::{Html, IntoResponse, Redirect},
};
use chrono::Utc;
use serde::Deserialize;
use uuid::Uuid;

// ── Avatar / initials helpers ─────────────────────────────────────────────────

fn users_htmx_initials(full_name: &str) -> String {
    let t = full_name.trim();
    if t.is_empty() {
        return "?".to_string();
    }
    let parts: Vec<&str> = t.split_whitespace().collect();
    if parts.len() == 1 {
        parts[0].chars().take(2).collect::<String>().to_uppercase()
    } else {
        let a = parts[0].chars().next().unwrap_or('?');
        let b = parts[1].chars().next().unwrap_or('?');
        format!("{}{}", a, b).to_uppercase()
    }
}

fn users_htmx_avatar_style(email: &str) -> String {
    let mut hash: i64 = 0;
    for b in email.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(b as i64);
    }
    let hue = hash.rem_euclid(360);
    format!(
        "background:hsl({hue},65%,28%);color:hsl({hue},75%,90%);box-shadow:inset 0 0 0 1px rgba(0,0,0,0.08)"
    )
}

// ── Fragment template ─────────────────────────────────────────────────────────

#[derive(askama::Template)]
#[template(path = "dashboard/users_partial.html")]
#[allow(dead_code)]
struct UsersHtmxFragment {
    sidebar_pinned: bool,
    user_email: String,
    css_version: &'static str,
    is_admin: bool,
    nav_active: &'static str,
    users: Vec<UserSummary>,
    total_users: usize,
    active_users: usize,
    disabled_users: usize,
    admin_users: usize,
    summary_date: String,
}

impl UsersHtmxFragment {
    fn initials(full_name: &str) -> String {
        users_htmx_initials(full_name)
    }
    fn avatar_style(email: &str) -> String {
        users_htmx_avatar_style(email)
    }
}

pub async fn users_htmx_html(pool: &Db, admin: &AdminClaims) -> Result<String, AppError> {
    let users = repositories::users::fetch_user_summaries(pool).await?;
    let admin_users = repositories::users::count_admins(pool).await? as usize;
    let total_users = users.len();
    let active_users = users.iter().filter(|u| u.is_active).count();
    let disabled_users = total_users.saturating_sub(active_users);
    let summary_date = Utc::now().format("%b %d").to_string();
    UsersHtmxFragment {
        sidebar_pinned: true,
        user_email: admin.0.email.clone(),
        css_version: env!("CSS_VERSION"),
        is_admin: admin.0.is_admin,
        nav_active: "users",
        users,
        total_users,
        active_users,
        disabled_users,
        admin_users,
        summary_date,
    }
    .render()
    .map_err(|e| AppError::Internal(e.to_string()))
}

// ── Request types ─────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct DisableUserRequest {
    pub reason: Option<String>,
}

// ── Route handlers ────────────────────────────────────────────────────────────

pub async fn disable_user(
    headers: HeaderMap,
    admin: AdminClaims,
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Form(payload): Form<DisableUserRequest>,
) -> impl IntoResponse {
    let pool = state.db;
    let actor_id = match Uuid::parse_str(&admin.0.sub) {
        Ok(v) => v,
        Err(_) => {
            if is_htmx(&headers) {
                return match users_htmx_html(&pool, &admin).await {
                    Ok(body) => {
                        Html(body + &global_message::with_error("Unauthorized")).into_response()
                    }
                    Err(_) => Redirect::to(&format!(
                        "/users?error={}",
                        query_param_encode("Unauthorized")
                    ))
                    .into_response(),
                };
            }
            return Redirect::to(&format!(
                "/users?error={}",
                query_param_encode("Unauthorized")
            ))
            .into_response();
        }
    };

    if actor_id == user_id {
        if is_htmx(&headers) {
            return match users_htmx_html(&pool, &admin).await {
                Ok(body) => {
                    Html(body + &global_message::with_error("You cannot disable your own account."))
                        .into_response()
                }
                Err(_) => Redirect::to(&format!(
                    "/users?error={}",
                    query_param_encode("You cannot disable your own account.")
                ))
                .into_response(),
            };
        }
        return Redirect::to(&format!(
            "/users?error={}",
            query_param_encode("You cannot disable your own account.")
        ))
        .into_response();
    }

    let res: AppResult<()> = async {
        let mut tx = pool.begin().await?;
        let updated = sqlx::query(
            r#"
        UPDATE users
        SET
            is_active       = FALSE,
            session_version = session_version + 1,
            disabled_at     = NOW(),
            disabled_by     = $1,
            disabled_reason = $2
        WHERE user_id = $3
          AND is_active = TRUE
        RETURNING user_id
        "#,
        )
        .bind(actor_id)
        .bind(&payload.reason)
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await?;

        if updated.is_none() {
            tx.rollback().await?;
            return Err(AppError::BadRequest(
                "User not found or already disabled.".into(),
            ));
        }

        sqlx::query(
            r#"
        INSERT INTO user_audit_log (target_user_id, actor_id, action, reason)
        VALUES ($1, $2, 'disabled', $3)
        "#,
        )
        .bind(user_id)
        .bind(actor_id)
        .bind(&payload.reason)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }
    .await;

    match res {
        Ok(()) => {
            if is_htmx(&headers) {
                return match users_htmx_html(&pool, &admin).await {
                    Ok(body) => {
                        Html(body + &global_message::with_success("User disabled.")).into_response()
                    }
                    Err(_) => Redirect::to("/users").into_response(),
                };
            }
            Redirect::to("/users").into_response()
        }
        Err(e) => {
            if is_htmx(&headers) {
                return match users_htmx_html(&pool, &admin).await {
                    Ok(body) => {
                        Html(body + &global_message::with_error(&e.to_string())).into_response()
                    }
                    Err(_) => Redirect::to(&format!(
                        "/users?error={}",
                        query_param_encode(&e.to_string())
                    ))
                    .into_response(),
                };
            }
            Redirect::to(&format!(
                "/users?error={}",
                query_param_encode(&e.to_string())
            ))
            .into_response()
        }
    }
}

pub async fn enable_user(
    headers: HeaderMap,
    admin: AdminClaims,
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> impl IntoResponse {
    let pool = state.db;
    let actor_id = match Uuid::parse_str(&admin.0.sub) {
        Ok(v) => v,
        Err(_) => {
            if is_htmx(&headers) {
                return match users_htmx_html(&pool, &admin).await {
                    Ok(body) => {
                        Html(body + &global_message::with_error("Unauthorized")).into_response()
                    }
                    Err(_) => Redirect::to(&format!(
                        "/users?error={}",
                        query_param_encode("Unauthorized")
                    ))
                    .into_response(),
                };
            }
            return Redirect::to(&format!(
                "/users?error={}",
                query_param_encode("Unauthorized")
            ))
            .into_response();
        }
    };

    let res: AppResult<()> = async {
        let mut tx = pool.begin().await?;

        let updated = sqlx::query(
            r#"
        UPDATE users
        SET
            is_active       = TRUE,
            disabled_at     = NULL,
            disabled_by     = NULL,
            disabled_reason = NULL
        WHERE user_id = $1
          AND is_active = FALSE
        RETURNING user_id
        "#,
        )
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await?;

        if updated.is_none() {
            tx.rollback().await?;
            return Err(AppError::BadRequest(
                "User not found or already active.".into(),
            ));
        }

        sqlx::query(
            "INSERT INTO user_audit_log (target_user_id, actor_id, action) VALUES ($1, $2, 'enabled')",
        )
        .bind(user_id)
        .bind(actor_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }
    .await;

    match res {
        Ok(()) => {
            if is_htmx(&headers) {
                return match users_htmx_html(&pool, &admin).await {
                    Ok(body) => {
                        Html(body + &global_message::with_success("User enabled.")).into_response()
                    }
                    Err(_) => Redirect::to("/users").into_response(),
                };
            }
            Redirect::to("/users").into_response()
        }
        Err(e) => {
            if is_htmx(&headers) {
                return match users_htmx_html(&pool, &admin).await {
                    Ok(body) => {
                        Html(body + &global_message::with_error(&e.to_string())).into_response()
                    }
                    Err(_) => Redirect::to(&format!(
                        "/users?error={}",
                        query_param_encode(&e.to_string())
                    ))
                    .into_response(),
                };
            }
            Redirect::to(&format!(
                "/users?error={}",
                query_param_encode(&e.to_string())
            ))
            .into_response()
        }
    }
}
