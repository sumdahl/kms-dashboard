use askama::Template;
use askama_axum::IntoResponse;
use axum::{
    extract::Form,
    response::Html,
    routing::{delete, get, post},
    Router,
};
use serde::Deserialize;
use tower_http::services::ServeDir;

#[derive(Template)]
#[template(path = "dashboard/home.html")]
struct HomeTemplate {
    sidebar_pinned: bool,
    user_email: String,
    show_banner: bool,
}

async fn home() -> impl axum::response::IntoResponse {
    HomeTemplate {
        sidebar_pinned: false,
        user_email: "user@example.com".to_string(),
        show_banner: true,
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

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(home))
        .route("/ui/sidebar/pin", post(sidebar_pin))
        .route("/ui/banner", delete(banner_dismiss))
        .nest_service("/static", ServeDir::new("static"))
        .nest_service("/nm", ServeDir::new("node_modules"));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind port 3000");

    println!("→  Dashboard running at http://localhost:3000");

    axum::serve(listener, app).await.expect("Server error");
}
