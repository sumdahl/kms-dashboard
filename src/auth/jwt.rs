use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use crate::models::auth::Claims;
use crate::error::{AppResult, AppError};
use chrono::{Utc, Duration};

pub fn create_jwt(user_id: &str, email: &str, is_admin: bool) -> AppResult<String> {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    
    let exp = Utc::now() + Duration::hours(24);
    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        is_admin,
        exp: exp.timestamp() as usize,
        iat: Utc::now().timestamp() as usize,
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
