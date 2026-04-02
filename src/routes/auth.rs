use crate::app_state::AppState;
use crate::handlers::auth::{login, logout, signup};
use axum::{routing::post, Router};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/signup", post(signup))
        .route("/logout", post(logout))
}
