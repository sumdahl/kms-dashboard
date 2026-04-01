use axum::{routing::{get, post}, Router};
use crate::handlers::admin::{create_role, list_roles, assign_role, list_users};
use crate::app_state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/roles", get(list_roles).post(create_role))
        .route("/assign", post(assign_role))
        .route("/users", get(list_users))
}
