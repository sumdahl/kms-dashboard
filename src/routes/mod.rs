use crate::app_state::AppState;
use crate::handlers::{actions, auth, pages, partials, password_reset};
use axum::{
    routing::{delete, get, post},
    Router,
};

pub fn app_router() -> Router<AppState> {
    Router::new()
        .merge(auth_routes())
        .merge(page_routes())
        .merge(partial_routes())
        .merge(ui_routes())
}

// ── Auth routes (no auth required) ───────────────────────────────────────────

fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/login", get(login_page).post(auth::login))
        .route("/signup", get(signup_page).post(auth::signup))
        .route(
            "/forgot-password",
            get(forgot_page).post(password_reset::forgot_password),
        )
        .route(
            "/reset-password",
            get(reset_page).post(password_reset::reset_password),
        )
        .route("/auth/logout", post(auth::logout))
}

// ── Page + Admin routes (auth required) ───────────────────────────────────────

fn page_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(pages::home))
        .route("/users", get(pages::users_page))
        .route("/roles", get(pages::roles_page).post(actions::create_role))
        .route("/roles/new", get(pages::create_role_wizard_page))
        .route("/roles/new/step1", get(actions::wizard_step1_back))
        .route("/roles/new/step2", post(actions::wizard_step2))
        .route("/roles/new/step3", post(actions::wizard_step3))
        .route("/roles/quick", get(pages::quick_create_role_page))
        .route("/roles/:role_id", delete(actions::delete_role))
        .route(
            "/assign",
            get(pages::assign_page).post(actions::assign_role),
        )
        .route("/search", get(pages::search_page))
        .route("/users/disable/:user_id", post(actions::disable_user))
        .route("/users/enable/:user_id", post(actions::enable_user))
}

// ── Partial routes (HTMX fragments) ──────────────────────────────────────────

fn partial_routes() -> Router<AppState> {
    Router::new()
        .route("/users/list", get(partials::users_list))
        .route("/users/:user_id/detail", get(partials::user_detail))
        .route(
            "/users/:user_id/disable-modal",
            get(partials::disable_modal),
        )
        .route("/users/stats", get(partials::users_stats))
        .route("/roles/list", get(partials::roles_list))
        .route("/roles/:role_id/detail", get(partials::role_detail_partial))
        .route("/roles/:role_id/delete-modal", get(partials::delete_modal))
        .route("/roles/stats", get(partials::roles_stats))
        .route("/roles/permission-row", post(partials::permission_row))
        .route("/roles/new/method", get(partials::create_method_modal))
        .route("/account-menu", get(partials::account_menu))
        .route("/me/roles", get(partials::my_roles))
        .route("/search/results", get(partials::search_results))
}

// ── UI utility routes ─────────────────────────────────────────────────────────

fn ui_routes() -> Router<AppState> {
    Router::new()
        .route("/ui/theme/toggle", post(actions::theme_toggle))
        .route("/ui/sidebar/pin", post(actions::sidebar_pin))
}

// ── Auth page GET handlers ────────────────────────────────────────────────────

use askama::Template;
use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use axum_extra::extract::CookieJar;
use serde::Deserialize;

#[derive(Template)]
#[template(path = "login.html")]
struct LoginPageTemplate {
    pub dark_mode: bool,
    pub form_email: String,
    pub global_error: Option<String>,
    pub email_error: String,
    pub password_error: String,
    pub account_disabled: bool,
}

async fn login_page(jar: CookieJar) -> impl IntoResponse {
    LoginPageTemplate {
        dark_mode: theme_from_jar(&jar),
        form_email: String::new(),
        global_error: None,
        email_error: String::new(),
        password_error: String::new(),
        account_disabled: false,
    }
}

#[derive(Template)]
#[template(path = "signup.html")]
struct SignupPageTemplate {
    pub dark_mode: bool,
    pub submitted: bool,
    pub form_full_name: String,
    pub form_email: String,
    pub global_error: Option<String>,
    pub name_error: String,
    pub email_error: String,
    pub password_error: String,
}

async fn signup_page(jar: CookieJar) -> impl IntoResponse {
    SignupPageTemplate {
        dark_mode: theme_from_jar(&jar),
        submitted: false,
        form_full_name: String::new(),
        form_email: String::new(),
        global_error: None,
        name_error: String::new(),
        email_error: String::new(),
        password_error: String::new(),
    }
}

#[derive(Template)]
#[template(path = "forgot_password.html")]
struct ForgotPageTemplate {
    pub dark_mode: bool,
    pub submitted: bool,
    pub form_email: String,
    pub global_error: Option<String>,
    pub email_error: String,
}

async fn forgot_page(jar: CookieJar) -> impl IntoResponse {
    ForgotPageTemplate {
        dark_mode: theme_from_jar(&jar),
        submitted: false,
        form_email: String::new(),
        global_error: None,
        email_error: String::new(),
    }
}

#[derive(Deserialize)]
struct ResetQuery {
    token: Option<String>,
}

#[derive(Template)]
#[template(path = "reset_password.html")]
struct ResetPageTemplate {
    pub dark_mode: bool,
    pub token_valid: bool,
    pub submitted: bool,
    pub token: String,
    pub global_error: Option<String>,
    pub password_error: String,
    pub confirm_error: String,
}

async fn reset_page(jar: CookieJar, Query(params): Query<ResetQuery>) -> impl IntoResponse {
    let token = params.token.unwrap_or_default();
    let token_valid = !token.is_empty();
    ResetPageTemplate {
        dark_mode: theme_from_jar(&jar),
        token_valid,
        submitted: false,
        token,
        global_error: None,
        password_error: String::new(),
        confirm_error: String::new(),
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn theme_from_jar(jar: &CookieJar) -> bool {
    jar.get("theme")
        .map(|c| c.value() == "dark")
        .unwrap_or(false)
}
