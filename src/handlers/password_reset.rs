use askama::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse, Response},
    Form,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::auth::hashing::hash_password;
use crate::error::{AppError, AppResult};
use crate::resend_mailer::mailer::send_reset_email;

// ── Forgot Password ──────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct ForgotPasswordForm {
    pub email: String,
}

#[derive(Template)]
#[template(path = "forgot_password.html")]
struct ForgotPasswordTemplate {
    pub dark_mode: bool,
    pub submitted: bool,
    pub form_email: String,
    pub global_error: Option<String>,
    pub email_error: String,
}

pub async fn forgot_password(
    State(state): State<AppState>,
    Form(form): Form<ForgotPasswordForm>,
) -> AppResult<Response> {
    if form.email.trim().is_empty() {
        return Ok(forgot_form(form.email, None, "Email is required.".into()));
    }

    let user = sqlx::query!(
        "SELECT user_id FROM users WHERE email = $1",
        form.email.trim()
    )
    .fetch_optional(&state.db)
    .await?;

    if let Some(user) = user {
        let token = Uuid::new_v4();
        let expires_at = Utc::now() + chrono::Duration::minutes(15);

        sqlx::query!(
            "DELETE FROM password_reset_tokens WHERE user_id = $1 AND used_at IS NULL",
            user.user_id
        )
        .execute(&state.db)
        .await?;

        sqlx::query!(
            "INSERT INTO password_reset_tokens (user_id, token, expires_at) VALUES ($1, $2, $3)",
            user.user_id,
            token,
            expires_at
        )
        .execute(&state.db)
        .await?;

        send_reset_email(
            &state.resend,
            form.email.trim(),
            &token.to_string(),
            &state.app_base_url,
        )
        .await?;
    }

    // Always show success — don't leak whether email is registered
    let html = ForgotPasswordTemplate {
        dark_mode: false,
        submitted: true,
        form_email: String::new(),
        global_error: None,
        email_error: String::new(),
    }
    .render()
    .unwrap_or_default();

    Ok(Html(html).into_response())
}

fn forgot_form(email: String, global: Option<String>, email_error: String) -> Response {
    let html = ForgotPasswordTemplate {
        dark_mode: false,
        submitted: false,
        form_email: email,
        global_error: global,
        email_error,
    }
    .render()
    .unwrap_or_default();
    Html(html).into_response()
}

// ── Reset Password ───────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct ResetPasswordForm {
    pub token: String,
    pub new_password: String,
    pub confirm_password: String,
}

#[derive(Template)]
#[template(path = "reset_password.html")]
struct ResetPasswordTemplate {
    pub dark_mode: bool,
    pub token_valid: bool,
    pub submitted: bool,
    pub token: String,
    pub global_error: Option<String>,
    pub password_error: String,
    pub confirm_error: String,
}

pub struct PasswordResetToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
}

pub async fn reset_password(
    State(state): State<AppState>,
    Form(form): Form<ResetPasswordForm>,
) -> AppResult<Response> {
    // Parse token
    let token_uuid = match Uuid::parse_str(&form.token) {
        Ok(t) => t,
        Err(_) => {
            return Ok(reset_form(
                String::new(),
                false,
                Some("Invalid reset link.".into()),
                String::new(),
                String::new(),
            ))
        }
    };

    // Validate inputs
    let mut password_error = String::new();
    let mut confirm_error = String::new();

    if form.new_password.len() < 8 {
        password_error = "Password must be at least 8 characters.".into();
    }
    if form.new_password != form.confirm_password {
        confirm_error = "Passwords do not match.".into();
    }

    if !password_error.is_empty() || !confirm_error.is_empty() {
        return Ok(reset_form(
            form.token.clone(),
            true,
            None,
            password_error,
            confirm_error,
        ));
    }

    // Fetch token record
    let record = sqlx::query_as!(
        PasswordResetToken,
        "SELECT id, user_id, expires_at, used_at as \"used_at: _\"
         FROM password_reset_tokens WHERE token = $1",
        token_uuid
    )
    .fetch_optional(&state.db)
    .await?;

    let record = match record {
        Some(r) => r,
        None => {
            return Ok(reset_form(
                String::new(),
                false,
                Some("Invalid or expired reset link.".into()),
                String::new(),
                String::new(),
            ))
        }
    };

    if record.used_at.is_some() || Utc::now() > record.expires_at {
        return Ok(reset_form(
            String::new(),
            false,
            Some("This reset link has expired or already been used.".into()),
            String::new(),
            String::new(),
        ));
    }

    let hashed = hash_password(&form.new_password)?;

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

    // Success state
    let html = ResetPasswordTemplate {
        dark_mode: false,
        token_valid: true,
        submitted: true,
        token: String::new(),
        global_error: None,
        password_error: String::new(),
        confirm_error: String::new(),
    }
    .render()
    .unwrap_or_default();

    Ok(Html(html).into_response())
}

fn reset_form(
    token: String,
    token_valid: bool,
    global: Option<String>,
    password_error: String,
    confirm_error: String,
) -> Response {
    let html = ResetPasswordTemplate {
        dark_mode: false,
        token_valid,
        submitted: false,
        token,
        global_error: global,
        password_error,
        confirm_error,
    }
    .render()
    .unwrap_or_default();
    Html(html).into_response()
}
