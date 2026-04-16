/// Assign page route handler.
use crate::app_state::AppState;
use crate::handlers::admin::{
    fetch_all_role_names, fetch_user_summaries, UserSummary,
};
use crate::models::Claims;
use crate::ui::global_message;
use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::{IntoResponse, Redirect, Response},
};

// ── Query params ──────────────────────────────────────────────────────────────

#[derive(serde::Deserialize, Default)]
pub struct AssignPageQuery {
    pub skip_onboarding: Option<bool>,
    pub error: Option<String>,
    pub notice: Option<String>,
    pub role: Option<String>,
}

// ── Templates ─────────────────────────────────────────────────────────────────

#[derive(askama::Template)]
#[template(path = "dashboard/assign.html")]
struct AssignTemplate {
    sidebar_pinned: bool,
    user_email: String,
    css_version: &'static str,
    is_admin: bool,
    nav_active: &'static str,
    pub users: Vec<UserSummary>,
    pub roles: Vec<String>,
    pub pre_role: String,
    pub global_message_oob_html: Option<String>,
    pub global_message_row_html: Option<String>,
    pub history_replace_url: Option<String>,
    pub assign_redirect: String,
}

impl AssignTemplate {
    fn role_selected(&self, role_name: &str) -> bool {
        self.pre_role == role_name
    }
}

#[derive(askama::Template)]
#[template(path = "dashboard/assign_partial.html")]
#[allow(dead_code)]
struct AssignPartialTemplate {
    pub sidebar_pinned: bool,
    pub user_email: String,
    pub css_version: &'static str,
    pub is_admin: bool,
    pub nav_active: &'static str,
    pub users: Vec<UserSummary>,
    pub roles: Vec<String>,
    pub pre_role: String,
    pub global_message_oob_html: Option<String>,
    pub history_replace_url: Option<String>,
    pub assign_redirect: String,
}

impl AssignPartialTemplate {
    fn role_selected(&self, role_name: &str) -> bool {
        self.pre_role == role_name
    }
}

#[derive(askama::Template)]
#[template(path = "dashboard/onboarding.html")]
#[allow(dead_code)]
struct OnboardingTemplate {
    sidebar_pinned: bool,
    user_email: String,
    css_version: &'static str,
    is_admin: bool,
    current_step: u8,
    nav_active: &'static str,
    pub users: Vec<UserSummary>,
    pub roles: Vec<String>,
    pub pre_role: String,
    pub global_message_oob_html: Option<String>,
    pub global_message_row_html: Option<String>,
    pub history_replace_url: Option<String>,
    pub assign_redirect: String,
}

impl OnboardingTemplate {
    fn role_selected(&self, role_name: &str) -> bool {
        self.pre_role == role_name
    }
}

#[derive(askama::Template)]
#[template(path = "dashboard/onboarding_partial.html")]
#[allow(dead_code)]
struct OnboardingPartialTemplate {
    pub sidebar_pinned: bool,
    pub user_email: String,
    pub css_version: &'static str,
    pub is_admin: bool,
    pub current_step: u8,
    pub nav_active: &'static str,
    pub users: Vec<UserSummary>,
    pub roles: Vec<String>,
    pub pre_role: String,
    pub global_message_oob_html: Option<String>,
    pub history_replace_url: Option<String>,
    pub assign_redirect: String,
}

impl OnboardingPartialTemplate {
    fn role_selected(&self, role_name: &str) -> bool {
        self.pre_role == role_name
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

pub fn assign_message_parts(params: &AssignPageQuery) -> Option<(&'static str, String)> {
    match params.notice.as_deref() {
        Some("assigned") => Some(("success", "Role assigned successfully.".to_string())),
        _ => params
            .error
            .as_deref()
            .map(str::trim)
            .filter(|e| !e.is_empty())
            .map(|e| ("error", e.to_string())),
    }
}

#[cfg(test)]
mod assign_tests {
    use super::{assign_message_parts, AssignPageQuery};

    #[test]
    fn assign_message_parts_maps_success_notice() {
        let params = AssignPageQuery {
            skip_onboarding: None,
            error: None,
            notice: Some("assigned".into()),
            role: None,
        };
        assert_eq!(
            assign_message_parts(&params),
            Some(("success", "Role assigned successfully.".to_string()))
        );
    }

    #[test]
    fn assign_message_parts_maps_errors() {
        let params = AssignPageQuery {
            skip_onboarding: None,
            error: Some("User not found".into()),
            notice: None,
            role: None,
        };
        assert_eq!(
            assign_message_parts(&params),
            Some(("error", "User not found".to_string()))
        );
    }
}

// ── Handler ───────────────────────────────────────────────────────────────────

pub async fn assign_page(
    headers: HeaderMap,
    claims: Option<Claims>,
    State(state): State<AppState>,
    Query(params): Query<AssignPageQuery>,
) -> Response {
    let pool = state.db;
    match claims {
        None => Redirect::to("/login").into_response(),
        Some(c) if !c.is_admin => Redirect::to("/").into_response(),
        Some(c) => {
            let users = fetch_user_summaries(&pool).await.unwrap_or_default();
            let roles = fetch_all_role_names(&pool).await.unwrap_or_default();
            let pre_role = params.role.clone().unwrap_or_default();
            let htmx = super::is_htmx_partial(&headers);
            let message = assign_message_parts(&params);
            let global_message_oob_html = message
                .as_ref()
                .map(|(kind, message)| global_message::from_query_kind(message, Some(kind)));
            let global_message_row_html = message
                .as_ref()
                .map(|(kind, message)| global_message::row_for_kind(message, Some(kind)));
            let should_clean_url = message.is_some() || params.skip_onboarding.is_some();
            let history_replace_url = should_clean_url.then(|| "/assign".to_string());

            if htmx {
                let mut res = AssignPartialTemplate {
                    sidebar_pinned: true,
                    user_email: c.email.clone(),
                    css_version: env!("CSS_VERSION"),
                    is_admin: c.is_admin,
                    nav_active: "assign",
                    users: users.clone(),
                    roles: roles.clone(),
                    pre_role: pre_role.clone(),
                    global_message_oob_html: global_message_oob_html.clone(),
                    history_replace_url: history_replace_url.clone(),
                    assign_redirect: "/assign".to_string(),
                }
                .into_response();

                if should_clean_url {
                    let hv = axum::http::HeaderValue::from_static("/assign");
                    res.headers_mut().insert("HX-Replace-Url", hv);
                }
                res
            } else {
                AssignTemplate {
                    sidebar_pinned: true,
                    user_email: c.email,
                    css_version: env!("CSS_VERSION"),
                    is_admin: c.is_admin,
                    nav_active: "assign",
                    users,
                    roles,
                    pre_role,
                    global_message_oob_html: None,
                    global_message_row_html,
                    history_replace_url,
                    assign_redirect: "/assign".to_string(),
                }
                .into_response()
            }
        }
    }
}
