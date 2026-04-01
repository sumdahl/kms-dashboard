mod auth;
mod db;
mod error;
mod models;
mod handlers;
mod middleware;

use tower_http::services::ServeDir;
use tower_livereload::LiveReloadLayer;

use crate::db::{init_db, run_migrations, seed_admin};
<<<<<<< HEAD
use crate::handlers::admin::{create_role, list_roles, assign_role};
use crate::handlers::auth::{login, signup};
use crate::handlers::dashboard::inventory_status;

#[derive(askama::Template)]
#[template(path = "dashboard/home.html")]
struct HomeTemplate {
    sidebar_pinned: bool,
    user_email: String,
    show_banner: bool,
    css_version: &'static str, // <-- Add this
    is_admin: bool,
}

async fn home() -> impl axum::response::IntoResponse {
    HomeTemplate {
        sidebar_pinned: false,
        user_email: "admin@example.com".to_string(),
        show_banner: true,
        // env!() pulls the value set in build.rs at compile time
        css_version: env!("CSS_VERSION"),
        is_admin: true,
    }
}

#[derive(Deserialize)]
struct SidebarPinForm {
    pinned: String,
}

async fn sidebar_pin(Form(_form): Form<SidebarPinForm>) -> Html<&'static str> {
    Html("")
}

async fn banner_dismiss() -> Html<&'static str> {
    Html("")
}
=======
use crate::config::Config;
use crate::app_state::AppState;
use crate::routes::create_router;
>>>>>>> feat/overall_flow

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
