use axum::{
    extract::Json,
    response::{IntoResponse, Response},
    http::header,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::db::Db;
use crate::error::{AppResult, AppError};
use crate::auth::hashing::{verify_password, hash_password};
use crate::auth::jwt::create_jwt;
use crate::middleware::auth::{build_auth_cookie, clear_auth_cookie};
use crate::models::User;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub full_name: String,
    pub password: String,
}

#[derive(askama::Template)]
#[template(path = "auth/login.html")]
pub struct LoginTemplate {
    pub css_version: &'static str,
}

#[derive(askama::Template)]
#[template(path = "auth/signup.html")]
pub struct SignupTemplate {
    pub css_version: &'static str,
}

pub async fn login_page() -> impl IntoResponse {
    LoginTemplate {
        css_version: env!("CSS_VERSION"),
    }
}

pub async fn signup_page() -> impl IntoResponse {
    SignupTemplate {
        css_version: env!("CSS_VERSION"),
    }
}

pub async fn login(
    axum::extract::State(pool): axum::extract::State<Db>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<Response> {
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = $1"
    )
    .bind(&payload.email)
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::BadCredentials)?;

    if !verify_password(&payload.password, &user.password_hash) {
        let html = format!(
            r#"<div id="login-error" hx-swap-oob="true" class="mb-4 p-3 rounded-sm bg-status-red/10 border border-status-red/20 text-sm text-status-red">Invalid email or password</div>"#
        );
        return Ok(Response::builder()
            .status(401)
            .header(header::CONTENT_TYPE, "text/html")
            .body(axum::body::Body::from(html))
            .unwrap());
    }

    let token = create_jwt(&user.user_id.to_string(), &user.email, user.is_admin)?;
    let cookie = build_auth_cookie(&token);

    Ok(Response::builder()
        .status(302)
        .header(header::SET_COOKIE, cookie)
        .header("HX-Redirect", "/")
        .body(axum::body::Body::empty())
        .unwrap())
}

pub async fn signup(
    axum::extract::State(pool): axum::extract::State<Db>,
    Json(payload): Json<SignupRequest>,
) -> AppResult<Response> {
    // 1. Check if user already exists
    let exists = sqlx::query("SELECT user_id FROM users WHERE email = $1")
        .bind(&payload.email)
        .fetch_optional(&pool)
        .await?;

    if exists.is_some() {
        let html = format!(
            r#"<div id="signup-error" hx-swap-oob="true" class="mb-4 p-3 rounded-sm bg-status-red/10 border border-status-red/20 text-sm text-status-red">Email already registered</div>"#
        );
        return Ok(Response::builder()
            .status(409)
            .header(header::CONTENT_TYPE, "text/html")
            .body(axum::body::Body::from(html))
            .unwrap());
    }

    // 2. Hash password and insert
    let hashed = hash_password(&payload.password)?;
    let user_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO users (user_id, email, full_name, password_hash) VALUES ($1, $2, $3, $4)"
    )
    .bind(user_id)
    .bind(&payload.email)
    .bind(&payload.full_name)
    .bind(hashed)
    .execute(&pool)
    .await?;

    // 3. Auto-login: generate JWT and set cookie
    let token = create_jwt(&user_id.to_string(), &payload.email, false)?;
    let cookie = build_auth_cookie(&token);

    Ok(Response::builder()
        .status(302)
        .header(header::SET_COOKIE, cookie)
        .header("HX-Redirect", "/")
        .body(axum::body::Body::empty())
        .unwrap())
}

pub async fn logout() -> Response {
    Response::builder()
        .status(302)
        .header(header::SET_COOKIE, clear_auth_cookie())
        .header(header::LOCATION, "/auth/login")
        .body(axum::body::Body::empty())
        .unwrap()
}
