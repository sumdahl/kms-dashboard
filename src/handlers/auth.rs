use crate::auth::hashing::{hash_password, verify_password};
use crate::auth::jwt::create_jwt;
use crate::db::Db;
use crate::error::{AppError, AppResult};
use crate::models::User;
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct SigninResponse {
    pub status: String,
    pub message: String,
    pub token: String,
    pub user_id: String,
}

#[derive(Serialize)]
pub struct SignupResponse {
    pub status: String,
    pub user_id: String,
    pub token: String,
    pub message: String,
}

#[derive(Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub full_name: String, // Added this
    pub password: String,
}

pub async fn login(
    State(pool): State<Db>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<Json<SigninResponse>> {
    let user = sqlx::query_as::<_, User>(
       "SELECT * FROM users WHERE email = $1",
    )
    .bind(&payload.email)
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::BadCredentials)?;

    if !verify_password(&payload.password, &user.password_hash) {
        return Err(AppError::BadCredentials);
    }

    let token = create_jwt(&user.user_id.to_string(), &user.email, user.is_admin)?;

    Ok(Json(SigninResponse {
        message: "Login successful".to_string(),
        status: "success".to_string(),
        token,
        user_id: user.user_id.to_string(),
    }))
}

pub async fn signup(
    State(pool): State<Db>,
    Json(payload): Json<SignupRequest>,
) -> AppResult<Json<SignupResponse>> {
    // 1. Check if user already exists
    let exists = sqlx::query("SELECT user_id FROM users WHERE email = $1")
        .bind(&payload.email)
        .fetch_optional(&pool)
        .await?;

    if exists.is_some() {
        return Err(AppError::EmailTaken);
    }

    // 2. Hash password and insert
    let hashed = hash_password(&payload.password)?;
    let user_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO users (user_id, email, full_name, password_hash) VALUES ($1, $2, $3, $4)",
    )
    .bind(user_id)
    .bind(&payload.email)
    .bind(&payload.full_name) // Bind name
    .bind(hashed)
    .execute(&pool)
    .await?;

    let token = create_jwt(&user_id.to_string(), &payload.email, false)?;
    Ok(Json(SignupResponse {
        status: "success".to_string(),
        user_id: user_id.to_string(),
        token,
        message: "Signup successful".to_string(),
    }))
}
