use crate::app_state::AppState;
use crate::handlers::admin::{
    assign_role, create_role, delete_role, get_role_detail, list_roles, list_users, roles_summary,
};
use axum::{
    routing::{delete, get, post},
    Router,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/roles", get(list_roles).post(create_role))
        .route("/roles/summary", get(roles_summary))
        .route("/roles/{name}", get(get_role_detail).delete(delete_role))
        .route("/assign", post(assign_role))
        .route("/users", get(list_users))
}
