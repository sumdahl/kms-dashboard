use crate::app_state::AppState;
use crate::auth::field_validation::{
    email_field_error, password_field_error, reset_confirm_password_error,
};
#[allow(unused_imports)]
use crate::resend_mailer::mailer::send_reset_email;
use crate::{
    auth::hashing::hash_password,
    error::{AppError, AppResult},
};
use askama::Template;
use axum::{
    extract::{Form, Query, State},
    http::{HeaderName, StatusCode},
    response::{Html, IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ForgotPasswordForm {
    pub email: String,
}

#[derive(Template)]
#[template(path = "forgot_password.html")]
pub struct ForgotPasswordView {
    pub full_page: bool,
    pub oob_swap: bool,
    pub email: String,
    pub email_error: Option<String>,
    pub banner: Option<String>,
}

#[derive(Template)]
#[template(path = "forgot_password_verify.html")]
pub struct ForgotPasswordVerifyView {}

fn render_forgot(view: ForgotPasswordView) -> Html<String> {
    Html(
        view.render()
            .unwrap_or_else(|e| format!("<!-- template error: {e} -->")),
    )
}

pub async fn forgot_password_page() -> impl IntoResponse {
    let view = ForgotPasswordView {
        full_page: true,
        oob_swap: false,
        email: String::new(),
        email_error: None,
        banner: None,
    };
    (StatusCode::OK, render_forgot(view))
}

pub async fn forgot_password_verify_page() -> impl IntoResponse {
    let view = ForgotPasswordVerifyView {};
    let html = view
        .render()
        .unwrap_or_else(|e| format!("<!-- template error: {e} -->"));
    (StatusCode::OK, Html(html))
}

#[derive(Deserialize)]
pub struct ResetTokenQuery {
    pub token: Option<String>,
}

#[derive(Deserialize)]
pub struct ResetPasswordForm {
    pub token: String,
    pub new_password: String,
    pub confirm_password: String,
}

#[derive(Template)]
#[template(path = "reset_password.html")]
pub struct ResetPasswordView {
    pub full_page: bool,
    pub oob_swap: bool,
    pub token_valid: bool,
    pub token: String,
    pub success: bool,
    pub new_password_error: Option<String>,
    pub confirm_password_error: Option<String>,
    pub banner: Option<String>,
}

fn render_reset(view: ResetPasswordView) -> Html<String> {
    Html(
        view.render()
            .unwrap_or_else(|e| format!("<!-- template error: {e} -->")),
    )
}

async fn token_valid_and_value(state: &AppState, token_raw: &str) -> (bool, String) {
    match Uuid::parse_str(token_raw) {
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
                    (
                        valid,
                        if valid {
                            token_raw.to_string()
                        } else {
                            String::new()
                        },
                    )
                }
                _ => (false, String::new()),
            }
        }
    }
}

pub async fn reset_password_page(
    State(state): State<AppState>,
    Query(query): Query<ResetTokenQuery>,
) -> impl IntoResponse {
    let token_opt = query.token.as_deref();
    let (token_valid, token) = match token_opt {
        None => (false, String::new()),
        Some(t) => token_valid_and_value(&state, t).await,
    };

    let view = ResetPasswordView {
        full_page: true,
        oob_swap: false,
        token_valid,
        token,
        success: false,
        new_password_error: None,
        confirm_password_error: None,
        banner: None,
    };

    (StatusCode::OK, render_reset(view))
}

pub async fn reset_password(
    State(state): State<AppState>,
    form: Result<Form<ResetPasswordForm>, axum::extract::rejection::FormRejection>,
) -> Response {
    let form = match form {
        Ok(f) => f.0,
        Err(_) => {
            let view = ResetPasswordView {
                full_page: false,
                oob_swap: true,
                token_valid: false,
                token: String::new(),
                success: false,
                new_password_error: None,
                confirm_password_error: None,
                banner: Some("Invalid form submission. Open link from email again.".into()),
            };
            return (StatusCode::OK, render_reset(view)).into_response();
        }
    };

    let new_pw_err = password_field_error(&form.new_password);
    let confirm_err = reset_confirm_password_error(&form.new_password, &form.confirm_password);

    if new_pw_err.is_some() || confirm_err.is_some() {
        let (token_valid, token) = token_valid_and_value(&state, &form.token).await;
        let view = ResetPasswordView {
            full_page: false,
            oob_swap: true,
            token_valid,
            token,
            success: false,
            new_password_error: new_pw_err,
            confirm_password_error: confirm_err,
            banner: None,
        };
        return (StatusCode::OK, render_reset(view)).into_response();
    }

    let token_uuid = match Uuid::parse_str(&form.token) {
        Ok(u) => u,
        Err(_) => {
            let view = ResetPasswordView {
                full_page: false,
                oob_swap: true,
                token_valid: false,
                token: String::new(),
                success: false,
                new_password_error: None,
                confirm_password_error: None,
                banner: Some("Invalid reset link.".into()),
            };
            return (StatusCode::OK, render_reset(view)).into_response();
        }
    };

    match apply_password_reset(&state, token_uuid, &form.new_password).await {
        Ok(()) => {
            let view = ResetPasswordView {
                full_page: false,
                oob_swap: true,
                token_valid: false,
                token: String::new(),
                success: true,
                new_password_error: None,
                confirm_password_error: None,
                banner: None,
            };
            (StatusCode::OK, render_reset(view)).into_response()
        }
        Err(AppError::BadRequest(msg)) => {
            let view = ResetPasswordView {
                full_page: false,
                oob_swap: true,
                token_valid: false,
                token: String::new(),
                success: false,
                new_password_error: None,
                confirm_password_error: None,
                banner: Some(msg),
            };
            (StatusCode::OK, render_reset(view)).into_response()
        }
        Err(e) => {
            tracing::error!("reset password failed: {e}");
            let view = ResetPasswordView {
                full_page: false,
                oob_swap: true,
                token_valid: token_valid_and_value(&state, &form.token).await.0,
                token: form.token,
                success: false,
                new_password_error: None,
                confirm_password_error: None,
                banner: Some("Something went wrong. Please try again later.".into()),
            };
            (StatusCode::OK, render_reset(view)).into_response()
        }
    }
}

async fn apply_password_reset(
    state: &AppState,
    token: Uuid,
    new_password: &str,
) -> Result<(), AppError> {
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

    if Utc::now() > record.expires_at {
        return Err(AppError::BadRequest("Reset token has expired".into()));
    }

    let hashed = hash_password(new_password)?;

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
    form: Result<Form<ForgotPasswordForm>, axum::extract::rejection::FormRejection>,
) -> AppResult<Response> {
    let form = match form {
        Ok(f) => f.0,
        Err(_) => {
            let view = ForgotPasswordView {
                full_page: false,
                oob_swap: true,
                email: String::new(),
                email_error: None,
                banner: Some("Invalid form submission. Please try again.".into()),
            };
            return Ok((StatusCode::OK, render_forgot(view)).into_response());
        }
    };

    let email = form.email.trim().to_string();
    if let Some(msg) = email_field_error(&email) {
        let view = ForgotPasswordView {
            full_page: false,
            oob_swap: true,
            email,
            email_error: Some(msg),
            banner: None,
        };
        return Ok((StatusCode::OK, render_forgot(view)).into_response());
    }

    let user = match sqlx::query!("SELECT user_id FROM users WHERE email = $1", email)
        .fetch_optional(&state.db)
        .await
    {
        Ok(u) => u,
        Err(e) => {
            tracing::error!("forgot-password db: {e}");
            let view = ForgotPasswordView {
                full_page: false,
                oob_swap: true,
                email,
                email_error: None,
                banner: Some("Something went wrong. Please try again later.".into()),
            };
            return Ok((StatusCode::OK, render_forgot(view)).into_response());
        }
    };

    if user.is_some() {
        tracing::info!("User found for forgot password: {}", email);
        // Re-enable when ready: create reset token + send email.
        let user = user.unwrap();
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

        if let Err(e) = send_reset_email(
            &state.resend,
            &email,
            &token.to_string(),
            &state.app_base_url,
        )
        .await
        {
            tracing::error!("Failed to send reset email to {}: {}", email, e);
        }
    } else {
        tracing::info!("No user found for forgot password: {}", email);
    }

    let mut res = StatusCode::NO_CONTENT.into_response();
    res.headers_mut().insert(
        HeaderName::from_static("hx-redirect"),
        "/forgot-password/verify".parse().unwrap(),
    );
    Ok(res)
}
