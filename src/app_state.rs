use crate::db::Db;
use axum::extract::FromRef;
use resend_rs::Resend;

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub resend: Resend,
    pub app_base_url: String,
}

impl FromRef<AppState> for Db {
    fn from_ref(state: &AppState) -> Self {
        state.db.clone()
    }
}
