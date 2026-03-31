mod auth;
mod db;
mod error;
mod models;
mod handlers;
mod middleware;

use axum::{
    extract::Json,
    response::Html,
    routing::{delete, get, post},
    Router,
};
use serde::Deserialize;
use tower_http::services::ServeDir;
use tower_livereload::LiveReloadLayer;

use crate::db::{init_db, run_migrations, seed_admin};
use crate::handlers::admin::{
    list_roles, create_role, assign_role,
    roles_page, assignments_page,
    roles_list, create_role_htmx, delete_role,
    assignments_list, assign_role_htmx, revoke_assignment,
};
use crate::handlers::auth::{login, signup, login_page, signup_page, logout};
use crate::handlers::dashboard::{dashboard_page, inventory_status};

#[derive(Deserialize)]
struct SidebarPinForm {
    pinned: String,
}

async fn sidebar_pin(Json(_body): Json<SidebarPinForm>) -> Html<&'static str> {
    Html("")
}

async fn banner_dismiss() -> Html<&'static str> {
    Html("")
}

#[tokio::main]
async fn main() {
    // 1. Load environment variables
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // 2. Initialize Database & Seed Admin
    let pool = init_db(&database_url).await;
    run_migrations(&pool).await.expect("Failed to run migrations");
    seed_admin(&pool).await.expect("Failed to seed admin");

    // 3. Build Router
    let app = Router::new()
        // Page routes
        .route("/", get(dashboard_page))
        .route("/auth/login", get(login_page).post(login))
        .route("/auth/signup", get(signup_page).post(signup))
        .route("/auth/logout", get(logout).post(logout))

        // Admin page routes
        .route("/admin/roles", get(roles_page))
        .route("/admin/assignments", get(assignments_page))

        // Admin HTMX partial routes
        .route("/admin/roles/list", get(roles_list))
        .route("/admin/roles/create", post(create_role_htmx))
        .route("/admin/roles/{role_id}", delete(delete_role))
        .route("/admin/assignments/list", get(assignments_list))
        .route("/admin/assignments/assign", post(assign_role_htmx))
        .route("/admin/assignments/{assignment_id}", delete(revoke_assignment))

        // API routes (JSON, unchanged)
        .route("/api/inventory", get(inventory_status))
        .route("/api/admin/roles", get(list_roles).post(create_role))
        .route("/api/admin/assign", post(assign_role))

        // UI interaction routes
        .route("/ui/sidebar/pin", post(sidebar_pin))
        .route("/ui/banner", delete(banner_dismiss))
        .nest_service("/static", ServeDir::new("static"))
        .nest_service("/nm", ServeDir::new("node_modules"))
        .layer(LiveReloadLayer::new())
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind port 3000");

    println!("→  Dashboard running at http://localhost:3000");

    axum::serve(listener, app).await.expect("Server error");
}
