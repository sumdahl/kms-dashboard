pub mod assign;
pub mod home;
pub mod roles;
pub mod users;

use crate::app_state::AppState;
use axum::{http::HeaderMap, routing::get, Router};

/// True when request comes from HTMX navigation (not a history restore).
pub fn is_htmx_partial(headers: &HeaderMap) -> bool {
    headers
        .get("hx-request")
        .and_then(|v| v.to_str().ok())
        .map(|v| v == "true")
        .unwrap_or(false)
        && !headers.contains_key("hx-history-restore-request")
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(home::home))
        .route("/roles", get(roles::roles_page))
        .route("/users", get(users::users_page))
        .route("/roles/new", get(roles::create_role_wizard_page))
        .route("/roles/quick", get(roles::quick_create_role_page))
        .route("/roles/:role_id", get(roles::role_detail_page))
        .route("/assign", get(assign::assign_page))
}
