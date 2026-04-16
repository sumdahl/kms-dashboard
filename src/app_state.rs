use crate::db::Db;
use crate::models::SessionConfig;
use axum::extract::FromRef;
use resend_rs::Resend;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub resend: Resend,
    pub app_base_url: String,
    pub session_config: Arc<RwLock<SessionConfig>>,
}

impl FromRef<AppState> for Db {
    fn from_ref(state: &AppState) -> Self {
        state.db.clone()
    }
}
