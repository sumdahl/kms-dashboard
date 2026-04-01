mod auth;
mod db;
mod error;
mod models;
mod handlers;
mod middleware;
mod app_state;
mod config;
mod routes;

use tower_http::services::ServeDir;
use tower_livereload::LiveReloadLayer;

use crate::db::{init_db, run_migrations, seed_admin};
use crate::config::Config;
use crate::app_state::AppState;
use crate::routes::create_router;

#[tokio::main]
async fn main() {
    // 1. Load Config
    let config = Config::from_env();

    // 2. Initialize Database
    let pool = init_db(&config.database_url).await;
    run_migrations(&pool).await.expect("Failed to run migrations");
    seed_admin(&pool).await.expect("Failed to seed admin");

    // 3. Initialize App State
    let state = AppState { db: pool };

    // 4. Build Router
    let app = create_router(state)
        .nest_service("/static", ServeDir::new("static"))
        .nest_service("/nm", ServeDir::new("node_modules"))
        .layer(LiveReloadLayer::new());

    // 5. Start Server
    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind port");

    println!("→  Dashboard running at http://{}", addr);

    axum::serve(listener, app).await.expect("Server error");
}
