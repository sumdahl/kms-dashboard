/// Users page route handler.
use crate::app_state::AppState;
use crate::models::Claims;
use crate::repositories;
use crate::repositories::users::UserSummary;
use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::{IntoResponse, Redirect, Response},
};
use chrono::Utc;

#[derive(serde::Deserialize, Default)]
pub struct UsersListQuery {
    pub flash_kind: Option<String>,
    pub flash_msg: Option<String>,
    pub error: Option<String>,
}

// ── Avatar helpers (same algorithm as admin handlers) ─────────────────────────

fn user_initials(full_name: &str) -> String {
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

fn user_avatar_style(email: &str) -> String {
    let mut hash: i64 = 0;
    for b in email.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(b as i64);
    }
    let hue = hash.rem_euclid(360);
    format!(
        "background:hsl({hue},65%,28%);color:hsl({hue},75%,90%);box-shadow:inset 0 0 0 1px rgba(0,0,0,0.08)"
    )
}

// ── Templates ─────────────────────────────────────────────────────────────────

#[derive(askama::Template)]
#[template(path = "dashboard/users.html")]
struct UsersTemplate {
    pub sidebar_pinned: bool,
    pub user_email: String,
    pub css_version: &'static str,
    pub is_admin: bool,
    pub nav_active: &'static str,
    pub users: Vec<UserSummary>,
    pub total_users: usize,
    pub active_users: usize,
    pub disabled_users: usize,
    pub admin_users: usize,
    pub summary_date: String,
}

impl UsersTemplate {
    fn initials(full_name: &str) -> String {
        user_initials(full_name)
    }
    fn avatar_style(email: &str) -> String {
        user_avatar_style(email)
    }
}

#[derive(askama::Template)]
#[template(path = "dashboard/users_partial.html")]
#[allow(dead_code)]
struct UsersPartialTemplate {
    pub sidebar_pinned: bool,
    pub user_email: String,
    pub css_version: &'static str,
    pub is_admin: bool,
    pub nav_active: &'static str,
    pub users: Vec<UserSummary>,
    pub total_users: usize,
    pub active_users: usize,
    pub disabled_users: usize,
    pub admin_users: usize,
    pub summary_date: String,
}

impl UsersPartialTemplate {
    fn initials(full_name: &str) -> String {
        user_initials(full_name)
    }
    fn avatar_style(email: &str) -> String {
        user_avatar_style(email)
    }
}

// ── Handler ───────────────────────────────────────────────────────────────────

pub async fn users_page(
    headers: HeaderMap,
    State(state): State<AppState>,
    Query(_q): Query<UsersListQuery>,
    claims: Option<Claims>,
) -> Response {
    let pool = state.db;
    match claims {
        None => Redirect::to("/login").into_response(),
        Some(c) if !c.is_admin => Redirect::to("/").into_response(),
        Some(c) => {
            let users = repositories::users::fetch_user_summaries(&pool)
                .await
                .unwrap_or_default();
            let admin_users = repositories::users::count_admins(&pool).await.unwrap_or(0) as usize;
            let total_users = users.len();
            let active_users = users.iter().filter(|u| u.is_active).count();
            let disabled_users = total_users.saturating_sub(active_users);
            let summary_date = Utc::now().format("%b %d").to_string();
            if super::is_htmx_partial(&headers) {
                UsersPartialTemplate {
                    sidebar_pinned: true,
                    user_email: c.email,
                    css_version: env!("CSS_VERSION"),
                    is_admin: c.is_admin,
                    nav_active: "users",
                    users,
                    total_users,
                    active_users,
                    disabled_users,
                    admin_users,
                    summary_date,
                }
                .into_response()
            } else {
                UsersTemplate {
                    sidebar_pinned: true,
                    user_email: c.email,
                    css_version: env!("CSS_VERSION"),
                    is_admin: c.is_admin,
                    nav_active: "users",
                    users,
                    total_users,
                    active_users,
                    disabled_users,
                    admin_users,
                    summary_date,
                }
                .into_response()
            }
        }
    }
}
