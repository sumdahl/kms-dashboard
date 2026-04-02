use crate::app_state::AppState;
use crate::handlers::logout_partial::account_menu;
use axum::{routing::get, Router};

pub fn router() -> Router<AppState> {
    Router::new().route("/account-menu", get(account_menu))
}
