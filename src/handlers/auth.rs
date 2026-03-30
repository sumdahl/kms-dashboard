use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use crate::db::Db;
use crate::error::{AppResult, AppError};
use crate::auth::hashing::verify_password;
use crate::auth::jwt::create_jwt;
use crate::models::User;

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

pub async fn login(
    State(pool): State<Db>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<Json<AuthResponse>> {
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = $1"
    )
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
