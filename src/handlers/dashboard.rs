use axum::{response::IntoResponse, Json};
use crate::error::AppResult;
use crate::middleware::auth::PageClaims;
use crate::middleware::rbac::Permissions;
use crate::models::types::{AccessLevel, Resource};

#[derive(askama::Template)]
#[template(path = "dashboard/home.html")]
pub struct HomeTemplate {
    pub sidebar_pinned: bool,
    pub user_email: String,
    pub is_admin: bool,
    pub show_banner: bool,
    pub css_version: &'static str,
}

pub async fn dashboard_page(
    PageClaims(claims): PageClaims,
) -> impl IntoResponse {
    HomeTemplate {
        sidebar_pinned: false,
        user_email: claims.email,
        is_admin: claims.is_admin,
        show_banner: true,
        css_version: env!("CSS_VERSION"),
    }
}

pub async fn inventory_status(
    perms: Permissions,
) -> AppResult<Json<serde_json::Value>> {
    perms.require(Resource::Inventory, AccessLevel::Read)?;

    Ok(Json(serde_json::json!({
        "status": "online",
        "items_count": 150,
        "message": "You have active access to inventory data."
    })))
}
