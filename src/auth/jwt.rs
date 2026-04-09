use crate::error::{AppError, AppResult};
use crate::models::auth::Claims;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::sync::OnceLock;
use uuid::Uuid;

static JWT_SECRET: OnceLock<String> = OnceLock::new();

pub fn init_jwt_secret(secret: String) {
    JWT_SECRET
        .set(secret)
        .expect("JWT secret already initialized");
}

fn secret() -> &'static str {
    JWT_SECRET
        .get()
        .expect("JWT secret not initialized — call init_jwt_secret at startup")
}

pub fn create_jwt(
    user_id: &str,
    email: &str,
    is_admin: bool,
    session_version: i32,
) -> AppResult<String> {
    let exp = Utc::now() + Duration::hours(24);
    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        is_admin,
        sv: session_version,
        exp: exp.timestamp() as usize,
        iat: Utc::now().timestamp() as usize,
        jti: Uuid::new_v4().to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret().as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("JWT error: {}", e)))
}

pub fn verify_jwt(token: &str) -> AppResult<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret().as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| AppError::Unauthorized)
}
