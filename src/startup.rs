use axum::Router;
use resend_rs::Resend;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::services::ServeDir;
use tower_livereload::LiveReloadLayer;
use tracing::{error, info};

use crate::app_state::AppState;
use crate::auth::blocklist::purge_expired_tokens;
use crate::config::Config;
use crate::db;
use crate::error::AppResult;
use crate::routes::{create_router, error_page_response};

fn internal_error_response(
    _panic_info: Box<dyn std::any::Any + Send>,
) -> axum::http::Response<axum::body::Body> {
    error_page_response(
        500,
        "Internal Server Error",
        "An unexpected error occurred while processing your request. Please try again later.",
    )
}

pub async fn init(config: &Config) -> AppResult<Router> {
    // 1. Database
    let pool = db::init_db(&config.database_url).await;
    db::run_migrations(&pool).await?;
    db::seed::seed_admin(&pool).await?;

    // 2. Clone for background task before pool moves into state
    let cleanup_pool = pool.clone();

    // 3. App state
    let state = AppState {
        db: pool,
        resend: Resend::default(),
    };

    // 4. Router
    let app = create_router(state)
        .nest_service("/static", ServeDir::new("static"))
        .nest_service("/nm", ServeDir::new("node_modules"))
        .nest_service("/public", ServeDir::new("public"))
        .layer(CatchPanicLayer::custom(internal_error_response))
        .layer(LiveReloadLayer::new());

    // 5. Background cleanup task
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(86_400));
        loop {
            interval.tick().await;
            match purge_expired_tokens(&cleanup_pool).await {
                Ok(n) => info!("Blocklist cleanup: removed {n} expired rows"),
                Err(e) => error!("Blocklist cleanup failed: {e}"),
            }
        }
    });

    Ok(app)
}

pub async fn serve(app: Router, port: u16) -> AppResult<()> {
    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind port");

    println!("→  Dashboard running at http://{addr}");
    axum::serve(listener, app).await.expect("Server error");

    Ok(())
}
