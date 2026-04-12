use crate::auth::blocklist::blocklist_token;
use crate::auth::hashing::{hash_password, verify_password};
use crate::auth::jwt::create_jwt;
use crate::db::Db;
use crate::error::{AppError, AppResult};
use crate::models::{Claims, User};
use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::{Form, Query, State};
use axum::http::StatusCode;
use axum::response::{Html, Response};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use serde::Deserialize;
use std::collections::HashMap;
use uuid::Uuid;

// --- Templates ---

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    pub email: String,
    pub email_error: String,
    pub password_error: String,
    pub global_error: String,
    pub account_disabled: bool,
}

#[derive(Template)]
#[template(path = "signup.html")]
pub struct SignupTemplate {
    pub email: String,
    pub full_name: String,
    pub email_error: String,
    pub full_name_error: String,
    pub password_error: String,
    pub global_error: String,
}

// --- Requests ---

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub full_name: String,
    pub password: String,
}

// --- Handlers ---

pub async fn login_page(Query(params): Query<HashMap<String, String>>) -> Response {
    let template = LoginTemplate {
        email: String::new(),
        email_error: String::new(),
        password_error: String::new(),
        global_error: String::new(),
        account_disabled: params
            .get("reason")
            .map(|r| r == "account_disabled")
            .unwrap_or(false),
    };
    template.into_response()
}

pub async fn signup_page() -> Response {
    let template = SignupTemplate {
        email: String::new(),
        full_name: String::new(),
        email_error: String::new(),
        full_name_error: String::new(),
        password_error: String::new(),
        global_error: String::new(),
    };
    template.into_response()
}

pub async fn login(
    State(pool): State<Db>,
    jar: CookieJar,
    Form(payload): Form<LoginRequest>,
) -> Response {
    let mut template = LoginTemplate {
        email: payload.email.clone(),
        email_error: String::new(),
        password_error: String::new(),
        global_error: String::new(),
        account_disabled: false,
    };

    let mut has_error = false;
    if payload.email.is_empty() {
        template.email_error = "Email is required".into();
        has_error = true;
    }
    if payload.password.is_empty() {
        template.password_error = "Password is required".into();
        has_error = true;
    }

    if has_error {
        return (StatusCode::UNPROCESSABLE_ENTITY, template).into_response();
    }

    let user = match sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(&payload.email)
        .fetch_optional(&pool)
        .await
    {
        Ok(Some(u)) => u,
        Ok(None) => {
            template.global_error = "Invalid email or password".into();
            return (StatusCode::UNAUTHORIZED, template).into_response();
        }
        Err(_) => {
            template.global_error = "An internal error occurred".into();
            return (StatusCode::INTERNAL_SERVER_ERROR, template).into_response();
        }
    };

    if !verify_password(&payload.password, &user.password_hash) {
        template.global_error = "Invalid email or password".into();
        return (StatusCode::UNAUTHORIZED, template).into_response();
    }

    if !user.is_active {
        template.account_disabled = true;
        return (StatusCode::FORBIDDEN, template).into_response();
    }

    let token = match create_jwt(
        &user.user_id.to_string(),
        &user.email,
        user.is_admin,
        user.session_version,
    ) {
        Ok(t) => t,
        Err(_) => {
            template.global_error = "Failed to create session".into();
            return (StatusCode::INTERNAL_SERVER_ERROR, template).into_response();
        }
    };

    let cookie = Cookie::build(("token", token))
        .http_only(true)
        .same_site(SameSite::Lax)
        .path("/")
        .build();
    
    let mut res = (jar.add(cookie), Html("")).into_response();
    res.headers_mut().insert("HX-Redirect", "/".parse().unwrap());
    res
}

pub async fn signup(
    State(pool): State<Db>,
    Form(payload): Form<SignupRequest>,
) -> Response {
    let mut template = SignupTemplate {
        email: payload.email.clone(),
        full_name: payload.full_name.clone(),
        email_error: String::new(),
        full_name_error: String::new(),
        password_error: String::new(),
        global_error: String::new(),
    };

    let mut has_error = false;
    if payload.email.is_empty() {
        template.email_error = "Email is required".into();
        has_error = true;
    } else if !payload.email.contains('@') {
        template.email_error = "Invalid email address".into();
        has_error = true;
    }

    if payload.full_name.is_empty() {
        template.full_name_error = "Full name is required".into();
        has_error = true;
    }

    if payload.password.len() < 6 {
        template.password_error = "Password must be at least 6 characters".into();
        has_error = true;
    }

    if has_error {
        return (StatusCode::UNPROCESSABLE_ENTITY, template).into_response();
    }

    let exists = match sqlx::query("SELECT user_id FROM users WHERE email = $1")
        .bind(&payload.email)
        .fetch_optional(&pool)
        .await
    {
        Ok(e) => e,
        Err(_) => {
            template.global_error = "An internal error occurred".into();
            return (StatusCode::INTERNAL_SERVER_ERROR, template).into_response();
        }
    };

    if exists.is_some() {
        template.email_error = "Email already registered".into();
        return (StatusCode::CONFLICT, template).into_response();
    }

    let hashed = match hash_password(&payload.password) {
        Ok(h) => h,
        Err(_) => {
            template.global_error = "Failed to secure password".into();
            return (StatusCode::INTERNAL_SERVER_ERROR, template).into_response();
        }
    };

    if let Err(_) = sqlx::query(
        "INSERT INTO users (user_id, email, full_name, password_hash) VALUES ($1, $2, $3, $4)",
    )
    .bind(Uuid::new_v4())
    .bind(&payload.email)
    .bind(&payload.full_name)
    .bind(hashed)
    .execute(&pool)
    .await
    {
        template.global_error = "Failed to save user".into();
        return (StatusCode::INTERNAL_SERVER_ERROR, template).into_response();
    }

    let mut res = (StatusCode::CREATED, Html("")).into_response();
    res.headers_mut().insert("HX-Redirect", "/login".parse().unwrap());
    res
}

pub async fn logout(State(pool): State<Db>, claims: Claims, jar: CookieJar) -> AppResult<Response> {
    let expires_at = chrono::DateTime::<chrono::Utc>::from_timestamp(claims.exp as i64, 0)
        .ok_or_else(|| AppError::Internal("Invalid token expiry".into()))?;

    blocklist_token(&pool, &claims.jti, expires_at).await?;

    let jar = jar.remove(Cookie::build(("token", "")).path("/"));
    let mut res = (jar, Html("")).into_response();
    res.headers_mut().insert("HX-Redirect", "/login".parse().unwrap());
    Ok(res)
}
