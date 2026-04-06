use crate::db::Db;
use axum::extract::FromRef;
use resend_rs::Resend;

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub resend: Resend,
}

// This allows Axum to automatically extract the DB pool
// from our AppState when a handler or extractor asks for it.
impl FromRef<AppState> for Db {
    fn from_ref(state: &AppState) -> Self {
        state.db.clone()
    }
}
