use crate::auth::blocklist::blocklist_token;
use crate::auth::hashing::{hash_password, verify_password};
use crate::auth::jwt::create_jwt;
use crate::db::Db;
use crate::error::{AppError, AppResult};
use crate::models::{Claims, User};
use axum::response::Response;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

#[derive(Serialize)]
pub struct SigninResponse {
    pub token: String,
    pub user_id: String,
    pub message: String,
    pub is_admin: bool,
}

#[derive(Serialize)]
pub struct SignupResponse {
    pub user_id: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct LogoutResponse {
    pub message: String,
}

pub async fn login(
    State(pool): State<Db>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<Response> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(&payload.email)
        .fetch_optional(&pool)
        .await?
        .ok_or(AppError::BadCredentials)?;

    if !verify_password(&payload.password, &user.password_hash) {
        return Err(AppError::BadCredentials);
    }

    if !user.is_active {
        return Err(AppError::AccountDisabled(user.disabled_reason));
    }

    let token = create_jwt(
        &user.user_id.to_string(),
        &user.email,
        user.is_admin,
        user.session_version,
    )?;

    let cookie = Cookie::build(("token", token.clone()))
        .http_only(true)
        .same_site(SameSite::Lax)
        .path("/")
        .build();
    let jar = CookieJar::new().add(cookie);

    let body = Json(SigninResponse {
        token,
        user_id: user.user_id.to_string(),
        message: "Login successful".into(),
        is_admin: user.is_admin,
    });

    let mut res = (jar, body).into_response();
    res.headers_mut()
        .insert("HX-Redirect", "/".parse().unwrap());
    Ok(res)
}

pub async fn signup(
    State(pool): State<Db>,
    Json(payload): Json<SignupRequest>,
) -> AppResult<Response> {
    let exists = sqlx::query("SELECT user_id FROM users WHERE email = $1")
        .bind(&payload.email)
        .fetch_optional(&pool)
        .await?;

    if exists.is_some() {
        return Err(AppError::EmailTaken);
    }

    let hashed = hash_password(&payload.password)?;
    let user_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO users (user_id, email, full_name, password_hash) VALUES ($1, $2, $3, $4)",
    )
    .bind(user_id)
    .bind(&payload.email)
    .bind(&payload.full_name)
    .bind(hashed)
    .execute(&pool)
    .await?;

    let body = Json(SignupResponse {
        user_id: user_id.to_string(),
        message: "Account created".into(),
    });

    let mut res = (StatusCode::CREATED, body).into_response();
    res.headers_mut()
        .insert("HX-Redirect", "/login".parse().unwrap());
    Ok(res)
}

pub async fn logout(State(pool): State<Db>, claims: Claims, jar: CookieJar) -> AppResult<Response> {
    let expires_at = chrono::DateTime::<chrono::Utc>::from_timestamp(claims.exp as i64, 0)
        .ok_or_else(|| AppError::Internal("Invalid token expiry".into()))?;

    blocklist_token(&pool, &claims.jti, expires_at).await?;

    let jar = jar.remove(Cookie::build(("token", "")).path("/"));

    let body = Json(LogoutResponse {
        message: "Logged out".into(),
    });

    let mut res = (jar, body).into_response();
    res.headers_mut()
        .insert("HX-Redirect", "/login".parse().unwrap());
    Ok(res)
}
