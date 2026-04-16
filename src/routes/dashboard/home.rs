/// Home page route handler.
use crate::app_state::AppState;
use crate::handlers::dashboard::{load_my_roles, MyRole};
use crate::models::Claims;
use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::{IntoResponse, Redirect, Response},
};

#[derive(serde::Deserialize)]
pub struct HomeParams {
    pub skip_onboarding: Option<bool>,
}

#[derive(askama::Template)]
#[template(path = "dashboard/home.html")]
struct HomeTemplate {
    sidebar_pinned: bool,
    user_email: String,
    css_version: &'static str,
    is_admin: bool,
    pub nav_active: &'static str,
    pub my_roles: Vec<MyRole>,
}

#[derive(askama::Template)]
#[template(path = "dashboard/home_partial.html")]
#[allow(dead_code)]
struct HomePartialTemplate {
    pub sidebar_pinned: bool,
    pub user_email: String,
    pub css_version: &'static str,
    pub is_admin: bool,
    pub nav_active: &'static str,
    pub my_roles: Vec<MyRole>,
}

pub async fn home(
    headers: HeaderMap,
    claims: Option<Claims>,
    State(state): State<AppState>,
    Query(_params): Query<HomeParams>,
) -> Response {
    match claims {
        None => Redirect::to("/login").into_response(),
        Some(c) => {
            let my_roles = load_my_roles(&c, &state.db).await.unwrap_or_default();
            if super::is_htmx_partial(&headers) {
                HomePartialTemplate {
                    sidebar_pinned: true,
                    user_email: c.email,
                    css_version: env!("CSS_VERSION"),
                    is_admin: c.is_admin,
                    nav_active: "home",
                    my_roles,
                }
                .into_response()
            } else {
                HomeTemplate {
                    sidebar_pinned: true,
                    user_email: c.email,
                    css_version: env!("CSS_VERSION"),
                    is_admin: c.is_admin,
                    nav_active: "home",
                    my_roles,
                }
                .into_response()
            }
        }
    }
}
