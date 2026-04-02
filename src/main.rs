mod app_state;
mod auth;
mod config;
mod db;
mod error;
mod handlers;
mod middleware;
mod models;
mod routes;

use tower_http::services::ServeDir;
use tower_livereload::LiveReloadLayer;
use tracing::{error, info};

use crate::app_state::AppState;
use crate::config::Config;
use crate::db::{init_db, run_migrations, seed_admin};
use crate::routes::create_router;

#[tokio::main]
async fn main() {
    // 1. Load Config
    let config = Config::from_env();

    // 2. Initialize Database
    let pool = init_db(&config.database_url).await;
    run_migrations(&pool)
        .await
        .expect("Failed to run migrations");
    seed_admin(&pool).await.expect("Failed to seed admin");

    // 3. Clone pool for cleanup task BEFORE moving into state
    let cleanup_pool = pool.clone();

    // 4. Initialize App State
    let state = AppState { db: pool };

    // 5. Build Router
    let app = create_router(state)
        .nest_service("/static", ServeDir::new("static"))
        .nest_service("/nm", ServeDir::new("node_modules"))
        .nest_service("/public", ServeDir::new("public"))
        .layer(LiveReloadLayer::new());

    // 6. Start Server
    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind port");

    // 7. Spawn daily blocklist cleanup task
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(86_400));
        loop {
            interval.tick().await;
            match crate::auth::blocklist::purge_expired_tokens(&cleanup_pool).await {
                Ok(n) => info!("Blocklist cleanup: removed {n} expired rows"),
                Err(e) => error!("Blocklist cleanup failed: {e}"),
            }
        }
    });

    println!("→  Dashboard running at http://{}", addr);
    axum::serve(listener, app).await.expect("Server error");
}
