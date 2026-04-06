use crate::app_state::AppState;
use crate::handlers::auth::{login, logout, signup};
use crate::handlers::password_reset::{forgot_password, reset_password};

use axum::{routing::post, Router};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/signup", post(signup))
        .route("/logout", post(logout))
        .route("/forgot-password", post(forgot_password))
        .route("/reset-password", post(reset_password))
}
