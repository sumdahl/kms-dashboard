use crate::app_state::AppState;
use crate::handlers::admin::{
    assign_role, create_role_form, delete_role, delete_role_htmx, delete_role_submit, disable_user,
    enable_user, get_role_detail, list_roles, list_users, permission_row, roles_list_htmx,
    roles_summary,
};
use axum::{
    http::StatusCode,
    routing::{get, post},
    Router,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/roles/create", post(create_role_form))
        .route("/roles/list", get(roles_list_htmx))
        .route("/roles", get(list_roles))
        .route("/roles/summary", get(roles_summary))
        .route("/roles/permission-row", get(permission_row))
        .route(
            "/roles/permission-row/remove",
            post(|| async { StatusCode::OK }),
        )
        .route("/roles/:role_id/htmx-delete", post(delete_role_htmx))
        .route("/roles/:role_id/delete", post(delete_role_submit))
        .route("/roles/:role_id", get(get_role_detail).delete(delete_role))
        .route("/assign", post(assign_role))
        .route("/users", get(list_users))
        .route("/users/disable/:user_id", post(disable_user))
        .route("/users/enable/:user_id", post(enable_user))
}
