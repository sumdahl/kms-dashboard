use axum::Json;
use crate::error::AppResult;
use crate::middleware::rbac::Permissions;
use crate::models::types::{AccessLevel, Resource};

pub async fn inventory_status(
    perms: Permissions, // Our new RBAC lock
) -> AppResult<Json<serde_json::Value>> {
    // Check if user has "inventory" with at least "read" access
    perms.require(Resource::Inventory, AccessLevel::Read)?;

    Ok(Json(serde_json::json!({
        "status": "online",
        "items_count": 150,
        "message": "You have active access to inventory data."
    })))
}
