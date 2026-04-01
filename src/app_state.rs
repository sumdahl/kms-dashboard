use crate::db::Db;
use axum::extract::FromRef;

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
}

// This allows Axum to automatically extract the DB pool 
// from our AppState when a handler or extractor asks for it.
impl FromRef<AppState> for Db {
    fn from_ref(state: &AppState) -> Self {
        state.db.clone()
    }
}
