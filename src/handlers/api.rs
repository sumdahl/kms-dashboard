use crate::app_state::AppState;
use crate::error::AppResult;
use askama::Template;
use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

#[derive(Template)]
#[template(path = "partials/search_results.html")]
pub struct SearchResultsTemplate {
    pub results: Vec<SearchResult>,
    pub query: String,
}

#[derive(Serialize)]
pub struct SearchResult {
    pub title: String,
    pub description: String,
    pub url: String,
    pub category: &'static str,
}

pub async fn global_search(
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> AppResult<impl IntoResponse> {
    let pool = state.db;
    let query = params.q.trim();

    // The 3-character rule (backend safety)
    if query.len() < 3 {
        return Ok(SearchResultsTemplate {
            results: vec![],
            query: query.to_string(),
        });
    }

    let search_pattern = format!("%{}%", query);
    let mut results = Vec::new();

    // 1. Search Static Pages
    let static_pages = vec![
        ("Account Home", "Overview of your account and security", "/"),
        ("Manage Roles", "View and edit system roles", "/roles"),
        ("Assign Roles", "Grant permissions to users", "/assign"),
    ];

    for (title, desc, url) in static_pages {
        if title.to_lowercase().contains(&query.to_lowercase())
            || desc.to_lowercase().contains(&query.to_lowercase())
        {
            results.push(SearchResult {
                title: title.to_string(),
                description: desc.to_string(),
                url: url.to_string(),
                category: "Page",
            });
        }
    }

    // 2. Search Database: Roles
    let roles = sqlx::query!(
        "SELECT name, description FROM roles WHERE name ILIKE $1 OR description ILIKE $1 LIMIT 5",
        search_pattern
    )
    .fetch_all(&pool)
    .await?;

    for role in roles {
        results.push(SearchResult {
            title: role.name.clone(),
            description: role.description.clone(),
            url: format!("/roles/{}", role.name),
            category: "Role",
        });
    }

    // 3. Search Database: Users
    let users = sqlx::query!(
        "SELECT email, full_name FROM users WHERE email ILIKE $1 OR full_name ILIKE $1 LIMIT 5",
        search_pattern
    )
    .fetch_all(&pool)
    .await?;

    for user in users {
        results.push(SearchResult {
            title: user.email,
            description: user.full_name.clone(),
            url: "#".to_string(), // Placeholder since we don't have user detail pages yet
            category: "User",
        });
    }

    Ok(SearchResultsTemplate {
        results,
        query: query.to_string(),
    })
}
