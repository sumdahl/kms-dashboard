use crate::error::{AppError, AppResult};
use resend_rs::types::CreateEmailBaseOptions;
use resend_rs::Resend;

pub async fn send_reset_email(resend: &Resend, to_email: &str, token: &str) -> AppResult<()> {
    let base_url =
        std::env::var("APP_BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
    let reset_link = format!("{base_url}/reset-password?token={token}");

    let html = format!(
        r#"
        <p>Hi,</p>
        <p>We received a request to reset your password.</p>
        <p>
          <a href="{reset_link}" style="padding:10px 20px;background:#4f46e5;color:white;border-radius:6px;text-decoration:none;">
            Reset Password
          </a>
        </p>
        <p>This link expires in <strong>15 minutes</strong>.</p>
        <p>If you did not request this, you can safely ignore this email.</p>
        "#
    );

    let email = CreateEmailBaseOptions::new(
        "KMS <noreply@sumirandahal.com.np>",
        [to_email],
        "Reset your KMS password",
    )
    .with_html(&html);

    resend
        .emails
        .send(email)
        .await
        .map_err(|e| AppError::Internal(format!("Resend error: {e}")))?;

    Ok(())
}
