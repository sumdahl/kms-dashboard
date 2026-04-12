mod app_state;
mod auth;
mod config;
mod db;
mod error;
mod handlers;
mod middleware;
mod models;
mod resend_mailer;
mod routes;
mod startup;
mod ui;

use crate::config::Config;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = Config::from_env();

    let app = startup::init(&config)
        .await
        .expect("Failed to initialize application");

    startup::serve(app, config.port)
        .await
        .expect("Server failed");
}
