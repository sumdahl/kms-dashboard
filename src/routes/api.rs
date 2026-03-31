use axum::{routing::get, Router};
use crate::handlers::dashboard::inventory_status;
use crate::app_state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/inventory", get(inventory_status))
}
