use crate::error::{AppError, AppResult};
use crate::models::auth::Claims;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use uuid::Uuid;

pub fn create_jwt(
    user_id: &str,
    email: &str,
    is_admin: bool,
    session_version: i32,
) -> AppResult<String> {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    let exp = Utc::now() + Duration::hours(24);
    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        is_admin,
        sv: session_version,
        exp: exp.timestamp() as usize,
        iat: Utc::now().timestamp() as usize,
        jti: Uuid::new_v4().to_string(), // ← new
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("JWT error: {}", e)))
}

pub fn verify_jwt(token: &str) -> AppResult<Claims> {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| AppError::Unauthorized)
}
