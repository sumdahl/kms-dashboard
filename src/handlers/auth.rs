use crate::auth::blocklist::blocklist_token;
use crate::auth::hashing::{hash_password, verify_password};
use crate::auth::jwt::create_jwt;
use crate::db::Db;
use crate::error::{AppError, AppResult};
use crate::models::{Claims, User};
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user_id: String,
}

#[derive(Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub full_name: String,
    pub password: String,
}

pub async fn login(
    State(pool): State<Db>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<Json<AuthResponse>> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(&payload.email)
        .fetch_optional(&pool)
        .await?
        .ok_or(AppError::BadCredentials)?;

    if !verify_password(&payload.password, &user.password_hash) {
        return Err(AppError::BadCredentials);
    }

    let token = create_jwt(&user.user_id.to_string(), &user.email, user.is_admin)?;
    Ok(Json(AuthResponse {
        token,
        user_id: user.user_id.to_string(),
    }))
}

pub async fn signup(
    State(pool): State<Db>,
    Json(payload): Json<SignupRequest>,
) -> AppResult<Json<serde_json::Value>> {
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

    Ok(Json(serde_json::json!({
        "status": "success",
        "user_id": user_id
    })))
}

pub async fn logout(State(pool): State<Db>, claims: Claims) -> AppResult<Json<serde_json::Value>> {
    let expires_at = chrono::DateTime::<chrono::Utc>::from_timestamp(claims.exp as i64, 0)
        .ok_or_else(|| AppError::Internal("Invalid token expiry".into()))?;

    blocklist_token(&pool, &claims.jti, expires_at).await?;

    Ok(Json(serde_json::json!({ "status": "logged out" })))
}
