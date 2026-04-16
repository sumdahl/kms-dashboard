//! Temporary artificial latency for manual testing (HTMX button indicators).
//! Only **POST, PATCH, PUT, DELETE** — GET/HEAD/etc. unchanged.
//! Remove `dev_delay` module + `from_fn` layer in `startup.rs` when done.

use std::time::Duration;

use axum::http::Method;
use axum::{extract::Request, middleware::Next, response::Response};

const DELAY_MS: u64 = 1500;

fn should_delay(method: &Method) -> bool {
    matches!(
        *method,
        Method::POST | Method::PATCH | Method::PUT | Method::DELETE
    )
}

pub async fn artificial_delay(req: Request, next: Next) -> Response {
    if should_delay(req.method()) {
        tokio::time::sleep(Duration::from_millis(DELAY_MS)).await;
    }
    next.run(req).await
}
