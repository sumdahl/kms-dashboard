use crate::app_state::AppState;
use crate::resend_mailer::mailer::send_reset_email;
use crate::auth::hashing::hash_password;
use crate::error::AppResult;
use askama::Template;
use askama_axum::IntoResponse;
use axum::{
    extract::{Form, Query, State},
    http::StatusCode,
    response::Redirect,
};
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

// --- Templates ---

#[derive(Template)]
#[template(path = "forgot_password.html")]
pub struct ForgotPasswordTemplate {
    pub email: String,
    pub error: String,
    pub success: bool,
}

#[derive(Deserialize)]
pub struct ResetTokenQuery {
    pub token: Option<String>,
}

#[derive(Template)]
#[template(path = "reset_password.html")]
pub struct ResetPasswordTemplate {
    pub token_valid: bool,
    pub token: String,
    pub error: String,
    pub success: bool,
}

// --- Handlers ---

pub async fn forgot_password_page() -> impl IntoResponse {
    ForgotPasswordTemplate {
        email: String::new(),
        error: String::new(),
        success: false,
    }
}

pub async fn reset_password_page(
    State(state): State<AppState>,
    Query(query): Query<ResetTokenQuery>,
) -> impl IntoResponse {
    let (token_valid, token) = match query.token {
        None => (false, String::new()),
        Some(ref t) => match Uuid::parse_str(t) {
            Err(_) => (false, String::new()),
            Ok(token_uuid) => {
                let result = sqlx::query!(
                    "SELECT expires_at, used_at FROM password_reset_tokens WHERE token = $1",
                    token_uuid
                )
                .fetch_optional(&state.db)
                .await;

                match result {
                    Ok(Some(record)) => {
                        let valid = record.used_at.is_none() && Utc::now() < record.expires_at;
                        (valid, if valid { t.clone() } else { String::new() })
                    }
                    _ => (false, String::new()),
                }
            }
        },
    };

    ResetPasswordTemplate {
        token_valid,
        token,
        error: String::new(),
        success: false,
    }
}

pub async fn forgot_password(
    State(state): State<AppState>,
    Form(payload): Form<ForgotPasswordRequest>,
) -> impl IntoResponse {
    let user = match sqlx::query!("SELECT user_id FROM users WHERE email = $1", payload.email)
        .fetch_optional(&state.db)
        .await {
            Ok(u) => u,
            Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, ForgotPasswordTemplate {
                email: payload.email,
                error: "An internal error occurred".into(),
                success: false,
            }).into_response(),
        };

    if let Some(user) = user {
        let token = Uuid::new_v4();
        let expires_at = Utc::now() + chrono::Duration::minutes(15);

        // Delete existing tokens
        if let Err(e) = sqlx::query!(
            "DELETE FROM password_reset_tokens WHERE user_id = $1 AND used_at IS NULL",
            user.user_id
        )
        .execute(&state.db)
        .await {
            return (StatusCode::INTERNAL_SERVER_ERROR, ForgotPasswordTemplate {
                email: payload.email,
                error: format!("Database error: {}", e),
                success: false,
            }).into_response();
        }

        // Insert new token
        if let Err(e) = sqlx::query!(
            "INSERT INTO password_reset_tokens (user_id, token, expires_at) VALUES ($1, $2, $3)",
            user.user_id,
            token,
            expires_at
        )
        .execute(&state.db)
        .await {
            return (StatusCode::INTERNAL_SERVER_ERROR, ForgotPasswordTemplate {
                email: payload.email,
                error: format!("Database error: {}", e),
                success: false,
            }).into_response();
        }

        // Send email
        if let Err(e) = send_reset_email(
            &state.resend,
            &payload.email,
            &token.to_string(),
            &state.app_base_url,
        ).await {
            return (StatusCode::INTERNAL_SERVER_ERROR, ForgotPasswordTemplate {
                email: payload.email,
                error: format!("Email error: {}", e),
                success: false,
            }).into_response();
        }
    }

    (StatusCode::OK, ForgotPasswordTemplate {
        email: payload.email,
        error: String::new(),
        success: true,
    }).into_response()
}

pub async fn reset_password(
    State(state): State<AppState>,
    Form(payload): Form<ResetPasswordRequest>,
) -> impl IntoResponse {
    let token_uuid = match Uuid::parse_str(&payload.token) {
        Ok(u) => u,
        Err(_) => return (StatusCode::BAD_REQUEST, ResetPasswordTemplate {
            token_valid: false,
            token: payload.token,
            error: "Invalid token format".into(),
            success: false,
        }).into_response(),
    };

    let record = match sqlx::query_as!(
        PasswordResetToken,
        "SELECT id, user_id, expires_at, used_at as \"used_at: _\"
         FROM password_reset_tokens
         WHERE token = $1",
        token_uuid
    )
    .fetch_optional(&state.db)
    .await {
        Ok(Some(r)) => r,
        Ok(None) => return (StatusCode::BAD_REQUEST, ResetPasswordTemplate {
            token_valid: false,
            token: payload.token,
            error: "Invalid or expired reset token".into(),
            success: false,
        }).into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, ResetPasswordTemplate {
            token_valid: false,
            token: payload.token,
            error: "Internal error".into(),
            success: false,
        }).into_response(),
    };

    if record.used_at.is_some() {
        return (StatusCode::BAD_REQUEST, ResetPasswordTemplate {
            token_valid: false,
            token: payload.token,
            error: "Reset token has already been used".into(),
            success: false,
        }).into_response();
    }

    if Utc::now() > record.expires_at {
        return (StatusCode::BAD_REQUEST, ResetPasswordTemplate {
            token_valid: false,
            token: payload.token,
            error: "Reset token has expired".into(),
            success: false,
        }).into_response();
    }

    let hashed = match hash_password(&payload.new_password) {
        Ok(h) => h,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, ResetPasswordTemplate {
            token_valid: true,
            token: payload.token,
            error: "Failed to process password".into(),
            success: false,
        }).into_response(),
    };

    let res: AppResult<()> = async {
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
        Ok(())
    }.await;

    if let Err(e) = res {
        return (StatusCode::INTERNAL_SERVER_ERROR, ResetPasswordTemplate {
            token_valid: true,
            token: payload.token,
            error: format!("Database error: {}", e),
            success: false,
        }).into_response();
    }

    // Success - redirect to login
    let mut headers = axum::http::HeaderMap::new();
    headers.insert("HX-Redirect", "/login".parse().unwrap());
    (StatusCode::OK, headers, Redirect::to("/login")).into_response()
}
