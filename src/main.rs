mod app_state;
mod auth;
mod config;
mod db;
mod email_templates;
mod error;
mod handlers;
mod mailer;
mod middleware;
mod models;
mod routes;
mod startup;

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
