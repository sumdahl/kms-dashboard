use crate::error::AppError;
use crate::models::Claims;
use askama::Template;
use askama_axum::IntoResponse;

#[derive(Template)]
#[template(path = "partials/account_menu.html")]
pub struct AccountMenuTemplate {
    pub user_email: String,
}

pub async fn account_menu(claims: Claims) -> Result<impl IntoResponse, AppError> {
    Ok(AccountMenuTemplate {
        user_email: claims.email,
    })
}
