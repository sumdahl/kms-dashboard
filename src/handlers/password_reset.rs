use crate::app_state::AppState;
use crate::resend_mailer::mailer::send_reset_email;
use crate::{
    auth::hashing::hash_password,
    error::{AppError, AppResult},
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
}

#[derive(Serialize)]
pub struct MessageResponse {
    pub message: String,
}

// Struct to help SQLx infer types for the password reset record
pub struct PasswordResetToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
}

pub async fn forgot_password(
    State(state): State<AppState>,
    Json(payload): Json<ForgotPasswordRequest>,
) -> AppResult<impl IntoResponse> {
    let user = sqlx::query!("SELECT user_id FROM users WHERE email = $1", payload.email)
        .fetch_optional(&state.db)
        .await?;

    if let Some(user) = user {
        let token = Uuid::new_v4();
        let expires_at = Utc::now() + chrono::Duration::minutes(15);

        sqlx::query!(
            "DELETE FROM password_reset_tokens
             WHERE user_id = $1 AND used_at IS NULL",
            user.user_id
        )
        .execute(&state.db)
        .await?;

        sqlx::query!(
            "INSERT INTO password_reset_tokens (user_id, token, expires_at)
             VALUES ($1, $2, $3)",
            user.user_id,
            token,
            expires_at
        )
        .execute(&state.db)
        .await?;

        send_reset_email(&state.resend, &payload.email, &token.to_string()).await?;
    }

    Ok((
        StatusCode::OK,
        Json(MessageResponse {
            message: "If that email is registered, a reset link has been sent.".into(),
        }),
    ))
}

pub async fn reset_password(
    State(state): State<AppState>,
    Json(payload): Json<ResetPasswordRequest>,
) -> AppResult<impl IntoResponse> {
    let token = Uuid::parse_str(&payload.token)
        .map_err(|_| AppError::BadRequest("Invalid token format".into()))?;

    // Use query_as! to map to our struct and solve the type inference issue
    let record = sqlx::query_as!(
        PasswordResetToken,
        "SELECT id, user_id, expires_at, used_at as \"used_at: _\"
         FROM password_reset_tokens
         WHERE token = $1",
        token
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::BadRequest(
        "Invalid or expired reset token".into(),
    ))?;

    if record.used_at.is_some() {
        return Err(AppError::BadRequest(
            "Reset token has already been used".into(),
        ));
    }

    // Expired?
    if Utc::now() > record.expires_at {
        return Err(AppError::BadRequest("Reset token has expired".into()));
    }

    let hashed = hash_password(&payload.new_password)?;

    let mut tx = state.db.begin().await?;

    sqlx::query!(
        "UPDATE users SET password_hash = $1 WHERE user_id = $2",
        hashed,
        record.user_id
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        "UPDATE password_reset_tokens SET used_at = $1 WHERE id = $2",
        Utc::now(),
        record.id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok((
        StatusCode::OK,
        Json(MessageResponse {
            message: "Password reset successful. You can now log in.".into(),
        }),
    ))
}
