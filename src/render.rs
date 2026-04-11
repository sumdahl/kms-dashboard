use askama::Template;
use axum::response::{Html, IntoResponse, Response};

pub fn render<T: Template>(t: T) -> Response {
    Html(
        t.render()
            .unwrap_or_else(|_| String::from("<h1>Template render error</h1>")),
    )
    .into_response()
}
