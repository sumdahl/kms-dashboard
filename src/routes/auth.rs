use crate::app_state::AppState;
use crate::handlers::auth::{login, login_page, logout, signup, signup_page};
use crate::handlers::password_reset::{
    forgot_password, forgot_password_page, reset_password, reset_password_page,
};

use axum::routing::{get, post};
use axum::Router;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", get(login_page).post(login))
        .route("/signup", get(signup_page).post(signup))
        .route("/logout", post(logout))
        .route("/forgot-password", get(forgot_password_page).post(forgot_password))
        .route("/reset-password", get(reset_password_page).post(reset_password))
}
