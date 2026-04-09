use super::email_templates::reset_password_email;
use crate::error::{AppError, AppResult};
use resend_rs::types::CreateEmailBaseOptions;
use resend_rs::Resend;

pub async fn send_reset_email(
    resend: &Resend,
    to_email: &str,
    token: &str,
    base_url: &str,
) -> AppResult<()> {
    let reset_link = format!("{base_url}/reset-password?token={token}");
    let html = reset_password_email(&reset_link);

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
