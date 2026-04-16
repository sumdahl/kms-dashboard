use crate::app_state::AppState;
use crate::auth::blocklist::blocklist_token;
use crate::auth::dto::{first_field_message, LoginRequest, SignupRequest};
use crate::auth::hashing::{hash_password, verify_password};
use crate::auth::jwt::create_jwt;
use crate::error::{AppError, AppResult};
use crate::models::{Claims, User};

use askama::Template;
use axum::response::Response;
use axum::{
    extract::{Form, State},
    http::{HeaderName, StatusCode},
    response::{Html, IntoResponse},
    Json,
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use serde::Serialize;
use uuid::Uuid;
use validator::Validate;

#[derive(Serialize)]
pub struct LogoutResponse {
    pub message: String,
}

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginView {
    pub full_page: bool,
    pub oob_swap: bool,
    pub account_disabled: bool,
    pub email: String,
    pub email_error: Option<String>,
    pub password_error: Option<String>,
    pub banner: Option<String>,
}

impl LoginView {
    pub fn page(account_disabled: bool) -> Self {
        Self {
            full_page: true,
            oob_swap: false,
            account_disabled,
            email: String::new(),
            email_error: None,
            password_error: None,
            banner: None,
        }
    }
}

#[derive(Template)]
#[template(path = "signup.html")]
pub struct SignupView {
    pub full_page: bool,
    pub oob_swap: bool,
    pub full_name: String,
    pub email: String,
    pub full_name_error: Option<String>,
    pub email_error: Option<String>,
    pub password_error: Option<String>,
    pub banner: Option<String>,
}

impl SignupView {
    pub fn page() -> Self {
        Self {
            full_page: true,
            oob_swap: false,
            full_name: String::new(),
            email: String::new(),
            full_name_error: None,
            email_error: None,
            password_error: None,
            banner: None,
        }
    }
}

pub async fn signup_page() -> impl IntoResponse {
    let view = SignupView::page();
    let html = view
        .render()
        .unwrap_or_else(|e| format!("<!-- template error: {e} -->"));
    (StatusCode::OK, Html(html))
}

pub async fn login(
    State(state): State<AppState>,
    form: Result<Form<LoginRequest>, axum::extract::rejection::FormRejection>,
) -> axum::response::Response {
    let pool = state.db;
    use axum::response::Html;

    fn render_form(view: LoginView) -> Html<String> {
        Html(
            view.render()
                .unwrap_or_else(|e| format!("<!-- template error: {e} -->")),
        )
    }

    let payload = match form {
        Ok(f) => f.0,
        Err(_) => {
            let view = LoginView {
                full_page: false,
                oob_swap: true,
                account_disabled: false,
                email: String::new(),
                email_error: None,
                password_error: None,
                banner: Some("Enter a valid email and password.".into()),
            };
            return (StatusCode::OK, render_form(view)).into_response();
        }
    };

    if let Err(ref errs) = payload.validate() {
        let email = payload.email.trim().to_string();
        let view = LoginView {
            full_page: false,
            oob_swap: true,
            account_disabled: false,
            email,
            email_error: first_field_message(errs, "email"),
            password_error: first_field_message(errs, "password"),
            banner: None,
        };
        return (StatusCode::OK, render_form(view)).into_response();
    }

    let email = payload.email.trim().to_string();
    let password = payload.password;

        let attempt: Result<(User, String, Uuid), AppError> = async {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(&email)
            .fetch_optional(&pool)
            .await?
            .ok_or(AppError::BadCredentials)?;

        if !verify_password(&password, &user.password_hash) {
            return Err(AppError::BadCredentials);
        }

        if !user.is_active {
            return Err(AppError::AccountDisabled(user.disabled_reason));
        }

        use crate::repositories::settings::get_session_config;
        let config = get_session_config(&pool).await?;
        let token = create_jwt(
            &user.user_id.to_string(),
            &user.email,
            user.is_admin,
            user.session_version,
            config.jwt_access_ttl_minutes as i64,
        )?;

        use chrono::Utc;
        let session_id = Uuid::new_v4();
        let expires_at = Utc::now() + chrono::Duration::hours(config.session_refresh_ttl_hours as i64);

        sqlx::query!(
            "INSERT INTO user_sessions (session_id, user_id, expires_at, last_activity)
             VALUES ($1, $2, $3, NOW())",
            session_id,
            user.user_id,
            expires_at,
        )
        .execute(&pool)
        .await?;

        Ok((user, token, session_id))
    }
    .await;

    match attempt {
        Ok((_user, token, session_id)) => {
            let token_cookie = Cookie::build(("token", token))
                .http_only(true)
                .same_site(SameSite::Lax)
                .path("/")
                .build();

            let sid_cookie = Cookie::build(("sid", session_id.to_string()))
                .http_only(true)
                .same_site(SameSite::Lax)
                .path("/")
                .build();

            let jar = CookieJar::new().add(token_cookie).add(sid_cookie);
            let mut res = jar.into_response();
            *res.status_mut() = StatusCode::NO_CONTENT;
            res.headers_mut()
                .insert(HeaderName::from_static("hx-redirect"), "/".parse().unwrap());
            res
        }
        Err(AppError::BadCredentials) => {
            let view = LoginView {
                full_page: false,
                oob_swap: true,
                account_disabled: false,
                email,
                email_error: None,
                password_error: None,
                banner: Some(AppError::BadCredentials.to_string()),
            };
            (StatusCode::OK, render_form(view)).into_response()
        }
        Err(AppError::AccountDisabled(reason)) => {
            let msg = match &reason {
                Some(r) => format!("Your account has been disabled. Reason: {r}"),
                None => "Your account has been disabled. Contact an administrator.".into(),
            };
            let view = LoginView {
                full_page: false,
                oob_swap: true,
                account_disabled: false,
                email,
                email_error: None,
                password_error: None,
                banner: Some(msg),
            };
            (StatusCode::OK, render_form(view)).into_response()
        }
        Err(e) => {
            tracing::error!("login failed: {e}");
            let view = LoginView {
                full_page: false,
                oob_swap: true,
                account_disabled: false,
                email,
                email_error: None,
                password_error: None,
                banner: Some("Something went wrong. Please try again later.".into()),
            };
            (StatusCode::OK, render_form(view)).into_response()
        }
    }
}

fn render_signup(view: SignupView) -> Html<String> {
    Html(
        view.render()
            .unwrap_or_else(|e| format!("<!-- template error: {e} -->")),
    )
}

pub async fn signup(
    State(state): State<AppState>,
    form: Result<Form<SignupRequest>, axum::extract::rejection::FormRejection>,
) -> Response {
    let pool = state.db;
    let payload = match form {
        Ok(f) => f.0,
        Err(_) => {
            let view = SignupView {
                full_page: false,
                oob_swap: true,
                full_name: String::new(),
                email: String::new(),
                full_name_error: None,
                email_error: None,
                password_error: None,
                banner: Some("Invalid form submission. Please try again.".into()),
            };
            return (StatusCode::OK, render_signup(view)).into_response();
        }
    };

    if let Err(ref errs) = payload.validate() {
        let full_name = payload.full_name.trim().to_string();
        let email = payload.email.trim().to_string();
        let view = SignupView {
            full_page: false,
            oob_swap: true,
            full_name,
            email,
            full_name_error: first_field_message(errs, "full_name"),
            email_error: first_field_message(errs, "email"),
            password_error: first_field_message(errs, "password"),
            banner: None,
        };
        return (StatusCode::OK, render_signup(view)).into_response();
    }

    let full_name = payload.full_name.trim().to_string();
    let email = payload.email.trim().to_string();
    let password = payload.password;

    let exists = match sqlx::query("SELECT user_id FROM users WHERE email = $1")
        .bind(&email)
        .fetch_optional(&pool)
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("signup lookup failed: {e}");
            let view = SignupView {
                full_page: false,
                oob_swap: true,
                full_name,
                email,
                full_name_error: None,
                email_error: None,
                password_error: None,
                banner: Some("Something went wrong. Please try again later.".into()),
            };
            return (StatusCode::OK, render_signup(view)).into_response();
        }
    };

    if exists.is_some() {
        let view = SignupView {
            full_page: false,
            oob_swap: true,
            full_name,
            email,
            full_name_error: None,
            email_error: None,
            password_error: None,
            banner: Some(AppError::EmailTaken.to_string()),
        };
        return (StatusCode::OK, render_signup(view)).into_response();
    }

    let hashed = match hash_password(&password) {
        Ok(h) => h,
        Err(e) => {
            tracing::error!("signup hash failed: {e}");
            let view = SignupView {
                full_page: false,
                oob_swap: true,
                full_name,
                email,
                full_name_error: None,
                email_error: None,
                password_error: None,
                banner: Some("Something went wrong. Please try again later.".into()),
            };
            return (StatusCode::OK, render_signup(view)).into_response();
        }
    };

    let user_id = Uuid::new_v4();
    if let Err(e) = sqlx::query(
        "INSERT INTO users (user_id, email, full_name, password_hash) VALUES ($1, $2, $3, $4)",
    )
    .bind(user_id)
    .bind(&email)
    .bind(&full_name)
    .bind(hashed)
    .execute(&pool)
    .await
    {
        tracing::error!("signup insert failed: {e}");
        let view = SignupView {
            full_page: false,
            oob_swap: true,
            full_name,
            email,
            full_name_error: None,
            email_error: None,
            password_error: None,
            banner: Some("Something went wrong. Please try again later.".into()),
        };
        return (StatusCode::OK, render_signup(view)).into_response();
    }

    let mut res = StatusCode::NO_CONTENT.into_response();
    res.headers_mut().insert(
        HeaderName::from_static("hx-redirect"),
        "/login".parse().unwrap(),
    );
    res
}

pub async fn logout(State(state): State<AppState>, claims: Claims, jar: CookieJar) -> AppResult<Response> {
    let pool = state.db;
    let expires_at = chrono::DateTime::<chrono::Utc>::from_timestamp(claims.exp as i64, 0)
        .ok_or_else(|| AppError::Internal("Invalid token expiry".into()))?;

    blocklist_token(&pool, &claims.jti, expires_at).await?;

    if let Some(sid_cookie) = jar.get("sid") {
        if let Ok(session_id) = Uuid::parse_str(sid_cookie.value()) {
            sqlx::query!("DELETE FROM user_sessions WHERE session_id = $1", session_id)
                .execute(&pool)
                .await?;
        }
    }

    let jar = jar
        .remove(Cookie::build(("token", "")).path("/"))
        .remove(Cookie::build(("sid", "")).path("/"));

    let body = Json(LogoutResponse {
        message: "Logged out".into(),
    });

    let mut res = (jar, body).into_response();
    res.headers_mut()
        .insert("HX-Redirect", "/login".parse().unwrap());
    Ok(res)
}
