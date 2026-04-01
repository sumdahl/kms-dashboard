use axum::{routing::post, Router};
use crate::handlers::auth::{login, signup};
use crate::app_state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/signup", post(signup))
}
