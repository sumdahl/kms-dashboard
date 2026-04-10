use askama::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse, Response},
    Form,
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::blocklist::blocklist_token;
use crate::auth::hashing::{hash_password, verify_password};
use crate::auth::jwt::create_jwt;
use crate::db::Db;
use crate::error::{AppError, AppResult};
use crate::models::{Claims, User};

// ── Login ────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {
    pub dark_mode: bool,
    pub form_email: String,
    pub global_error: Option<String>,
    pub email_error: String,
    pub password_error: String,
    pub account_disabled: bool,
}

pub async fn login(
    State(pool): State<Db>,
    jar: CookieJar,
    Form(form): Form<LoginForm>,
) -> AppResult<Response> {
    // Validate email
    if form.email.trim().is_empty() {
        return Ok(login_error(
            form.email.clone(),
            None,
            "Email is required.",
            "",
        ));
    }
    if form.password.is_empty() {
        return Ok(login_error(
            form.email.clone(),
            None,
            "",
            "Password is required.",
        ));
    }

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(form.email.trim())
        .fetch_optional(&pool)
        .await?
        .ok_or(AppError::BadCredentials);

    let user = match user {
        Ok(u) => u,
        Err(_) => {
            return Ok(login_error(
                form.email.clone(),
                Some("Invalid email or password.".into()),
                "",
                "",
            ))
        }
    };

    if !verify_password(&form.password, &user.password_hash) {
        return Ok(login_error(
            form.email.clone(),
            Some("Invalid email or password.".into()),
            "",
            "",
        ));
    }

    if !user.is_active {
        let html = LoginTemplate {
            dark_mode: false,
            form_email: form.email.clone(),
            global_error: Some("Your account has been disabled.".into()),
            email_error: String::new(),
            password_error: String::new(),
            account_disabled: true,
        }
        .render()
        .unwrap_or_default();
        return Ok(Html(html).into_response());
    }

    let token = create_jwt(
        &user.user_id.to_string(),
        &user.email,
        user.is_admin,
        user.session_version,
    )?;

    let cookie = Cookie::build(("token", token))
        .http_only(true)
        .same_site(SameSite::Lax)
        .path("/")
        .build();

    let mut response = axum::http::Response::builder()
        .status(axum::http::StatusCode::OK)
        .header("HX-Redirect", "/")
        .body(axum::body::Body::empty())
        .unwrap();

    let jar = jar.add(cookie);
    // Merge jar into response headers
    let jar_response = jar.into_response();
    for (key, val) in jar_response.headers() {
        response.headers_mut().insert(key, val.clone());
    }

    Ok(response)
}

fn login_error(
    email: String,
    global: Option<String>,
    email_err: &str,
    password_err: &str,
) -> Response {
    let html = LoginTemplate {
        dark_mode: false,
        form_email: email,
        global_error: global,
        email_error: email_err.to_string(),
        password_error: password_err.to_string(),
        account_disabled: false,
    }
    .render()
    .unwrap_or_default();
    Html(html).into_response()
}

// ── Signup ───────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct SignupForm {
    pub full_name: String,
    pub email: String,
    pub password: String,
}

#[derive(Template)]
#[template(path = "signup.html")]
struct SignupTemplate {
    pub dark_mode: bool,
    pub submitted: bool,
    pub form_full_name: String,
    pub form_email: String,
    pub global_error: Option<String>,
    pub name_error: String,
    pub email_error: String,
    pub password_error: String,
}

pub async fn signup(State(pool): State<Db>, Form(form): Form<SignupForm>) -> AppResult<Response> {
    let mut name_error = String::new();
    let mut email_error = String::new();
    let mut password_error = String::new();

    if form.full_name.trim().is_empty() {
        name_error = "Full name is required.".into();
    }
    if form.email.trim().is_empty() {
        email_error = "Email is required.".into();
    }
    if form.password.len() < 8 {
        password_error = "Password must be at least 8 characters.".into();
    }

    if !name_error.is_empty() || !email_error.is_empty() || !password_error.is_empty() {
        return Ok(signup_form(
            form.full_name,
            form.email,
            None,
            name_error,
            email_error,
            password_error,
        ));
    }

    // Check duplicate email
    let exists = sqlx::query("SELECT user_id FROM users WHERE email = $1")
        .bind(form.email.trim())
        .fetch_optional(&pool)
        .await?;

    if exists.is_some() {
        return Ok(signup_form(
            form.full_name,
            form.email,
            Some("An account with this email already exists.".into()),
            String::new(),
            String::new(),
            String::new(),
        ));
    }

    let hashed = hash_password(&form.password)?;
    let user_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO users (user_id, email, full_name, password_hash) VALUES ($1, $2, $3, $4)",
    )
    .bind(user_id)
    .bind(form.email.trim())
    .bind(form.full_name.trim())
    .bind(hashed)
    .execute(&pool)
    .await?;

    // Success state — HTMX swaps form with success message
    let html = SignupTemplate {
        dark_mode: false,
        submitted: true,
        form_full_name: String::new(),
        form_email: String::new(),
        global_error: None,
        name_error: String::new(),
        email_error: String::new(),
        password_error: String::new(),
    }
    .render()
    .unwrap_or_default();

    Ok(Html(html).into_response())
}

fn signup_form(
    full_name: String,
    email: String,
    global: Option<String>,
    name_error: String,
    email_error: String,
    password_error: String,
) -> Response {
    let html = SignupTemplate {
        dark_mode: false,
        submitted: false,
        form_full_name: full_name,
        form_email: email,
        global_error: global,
        name_error,
        email_error,
        password_error,
    }
    .render()
    .unwrap_or_default();
    Html(html).into_response()
}

// ── Logout ───────────────────────────────────────────────────────────────────

pub async fn logout(State(pool): State<Db>, claims: Claims, jar: CookieJar) -> AppResult<Response> {
    let expires_at = chrono::DateTime::<chrono::Utc>::from_timestamp(claims.exp as i64, 0)
        .ok_or_else(|| AppError::Internal("Invalid token expiry".into()))?;

    blocklist_token(&pool, &claims.jti, expires_at).await?;

    let jar = jar.remove(Cookie::build(("token", "")).path("/"));

    let mut response = axum::http::Response::builder()
        .status(axum::http::StatusCode::OK)
        .header("HX-Redirect", "/login")
        .body(axum::body::Body::empty())
        .unwrap();

    let jar_response = jar.into_response();
    for (key, val) in jar_response.headers() {
        response.headers_mut().insert(key, val.clone());
    }

    Ok(response)
}
