use crate::app_state::AppState;
use crate::handlers::admin::{
    assign_role, create_role, delete_role, disable_user, enable_user, get_role_detail, list_roles,
    list_users, permission_row, roles_summary, wizard_step_1, wizard_step_2,
};
use axum::{
    routing::{get, post},
    Router,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/roles", get(list_roles).post(create_role))
        .route("/roles/summary", get(roles_summary))
        .route("/roles/permission-row", post(permission_row))
        .route("/roles/wizard/step-1", post(wizard_step_1))
        .route("/roles/wizard/step-2", post(wizard_step_2))
        .route("/roles/:role_id", get(get_role_detail).delete(delete_role))
        .route("/assign", post(assign_role))
        .route("/users", get(list_users))
        .route("/users/disable/:user_id", post(disable_user))
        .route("/users/enable/:user_id", post(enable_user))
}
