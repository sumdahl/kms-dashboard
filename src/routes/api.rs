use crate::app_state::AppState;
use crate::handlers::api::global_search;
use crate::handlers::dashboard::{inventory_status, my_roles};
use axum::{routing::get, Router};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/inventory", get(inventory_status))
        .route("/me/roles", get(my_roles))
        .route("/search", get(global_search))
}
