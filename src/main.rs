mod app_state;
mod auth;
mod config;
mod db;
mod error;
mod handlers;
mod middleware;
<<<<<<< Updated upstream
=======
mod models;
mod routes;
>>>>>>> Stashed changes

use axum::{
    extract::Form,
    response::Html,
    routing::{delete, get, post},
    Router,
};
use serde::Deserialize;
use tower_http::services::ServeDir;
use tower_livereload::LiveReloadLayer;

<<<<<<< Updated upstream
use crate::db::{init_db, run_migrations, seed_admin};
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
use crate::app_state::AppState;
use crate::config::Config;
use crate::db::{init_db, run_migrations, seed_admin};
use crate::routes::create_router;
>>>>>>> Stashed changes

#[tokio::main]
async fn main() {
    // 1. Load environment variables
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

<<<<<<< Updated upstream
    // 2. Initialize Database & Seed Admin
    let pool = init_db(&database_url).await;
    run_migrations(&pool).await.expect("Failed to run migrations");
=======
    // 2. Initialize Database
    let pool = init_db(&config.database_url).await;
    run_migrations(&pool)
        .await
        .expect("Failed to run migrations");
>>>>>>> Stashed changes
    seed_admin(&pool).await.expect("Failed to seed admin");

    // 3. Build Router
    let app = Router::new()
        .route("/", get(home))
        .route("/auth/login", post(login))
        .route("/auth/signup", post(signup))
        .route("/admin/roles", get(list_roles).post(create_role))
        .route("/admin/assign", post(assign_role))
        .route("/api/inventory", get(inventory_status))
        .route("/ui/sidebar/pin", post(sidebar_pin))
        .route("/ui/banner", delete(banner_dismiss))
        .nest_service("/static", ServeDir::new("static"))
        .nest_service("/nm", ServeDir::new("node_modules"))
        .layer(LiveReloadLayer::new())
        .with_state(pool); // Share DB pool with handlers

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind port 3000");

    println!("→  Dashboard running at http://localhost:3000");

    axum::serve(listener, app).await.expect("Server error");
}
