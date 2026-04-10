use crate::app_state::AppState;
use crate::handlers::auth::{login, login_page, logout, signup, signup_page};
use crate::handlers::password_reset::{forgot_password, reset_password};

use axum::routing::{get, post};
use axum::Router;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", get(login_page).post(login))
        .route("/signup", get(signup_page).post(signup))
        .route("/logout", post(logout))
        .route("/forgot-password", post(forgot_password))
        .route("/reset-password", post(reset_password))
}
