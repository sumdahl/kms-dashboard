/// Shared view types, helpers, and utility functions used by admin sub-handlers.
use crate::auth::dto::CreateRoleFormRequest;
use crate::middleware::auth::AdminClaims;
use askama::Template;
use axum::{
    body::Body,
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};

// ── View types ────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct QuickPermissionRow {
    pub resource_sel: usize,
    pub access_sel: usize,
}

/// Quick create role page (`/roles/quick`).
#[derive(Template)]
#[template(path = "dashboard/quick_create_role.html")]
pub struct QuickCreateRoleView {
    pub sidebar_pinned: bool,
    pub user_email: String,
    pub css_version: &'static str,
    pub is_admin: bool,
    pub nav_active: &'static str,
    pub banner: Option<String>,
    pub name: String,
    pub description: String,
    pub name_error: Option<String>,
    pub description_error: Option<String>,
    pub resource_error: Option<String>,
    pub oob_swap: bool,
    pub resources: Vec<String>,
    pub access_levels: Vec<String>,
    pub permission_rows: Vec<QuickPermissionRow>,
}

/// Create role wizard (`/roles/new`).
#[derive(Template)]
#[template(path = "dashboard/create_role_wizard.html")]
pub struct CreateRoleWizardView {
    pub sidebar_pinned: bool,
    pub user_email: String,
    pub css_version: &'static str,
    pub is_admin: bool,
    pub nav_active: &'static str,
    pub banner: Option<String>,
    pub name: String,
    pub description: String,
    pub name_error: Option<String>,
    pub description_error: Option<String>,
    pub resource_error: Option<String>,
    pub oob_swap: bool,
    pub resources: Vec<String>,
    pub access_levels: Vec<String>,
    pub permission_rows: Vec<QuickPermissionRow>,
    pub redirect: String,
}

// ── List helpers ──────────────────────────────────────────────────────────────

pub fn quick_create_resource_list() -> Vec<String> {
    vec![
        "orders".into(),
        "customers".into(),
        "reports".into(),
        "inventory".into(),
        "admin_panel".into(),
    ]
}

pub fn quick_create_access_level_list() -> Vec<String> {
    vec!["read".into(), "write".into(), "admin".into()]
}

fn option_index(opts: &[String], value: &str) -> usize {
    opts.iter().position(|o| o == value).unwrap_or(0)
}

pub fn quick_create_default_permission_rows() -> Vec<QuickPermissionRow> {
    vec![QuickPermissionRow {
        resource_sel: 0,
        access_sel: 0,
    }]
}

pub fn quick_permission_rows_from_form(form: &CreateRoleFormRequest) -> Vec<QuickPermissionRow> {
    let resources = quick_create_resource_list();
    let access_levels = quick_create_access_level_list();
    let n = form.resource.len().min(form.access.len());
    let mut rows = Vec::new();
    for i in 0..n {
        let r = form.resource[i].trim();
        let a = form.access[i].trim();
        rows.push(QuickPermissionRow {
            resource_sel: option_index(&resources, r),
            access_sel: option_index(&access_levels, a),
        });
    }
    if rows.is_empty() {
        quick_create_default_permission_rows()
    } else {
        rows
    }
}

// ── HTMX / referer detection ──────────────────────────────────────────────────

pub fn is_htmx(headers: &HeaderMap) -> bool {
    headers.get("hx-request").and_then(|v| v.to_str().ok()) == Some("true")
}

pub fn referer_is_quick_create(headers: &HeaderMap) -> bool {
    headers
        .get(axum::http::header::REFERER)
        .and_then(|v| v.to_str().ok())
        .map(|r| r.contains("/roles/quick"))
        .unwrap_or(false)
}

pub fn is_quick_create_htmx(headers: &HeaderMap, form: Option<&CreateRoleFormRequest>) -> bool {
    if !is_htmx(headers) {
        return false;
    }
    match form {
        Some(f) => f.error_redirect.as_deref().map(str::trim) == Some("/roles/quick"),
        None => referer_is_quick_create(headers),
    }
}

pub fn referer_is_wizard(headers: &HeaderMap) -> bool {
    headers
        .get(axum::http::header::REFERER)
        .and_then(|v| v.to_str().ok())
        .map(|r| r.contains("/roles/new"))
        .unwrap_or(false)
}

pub fn is_wizard_htmx(headers: &HeaderMap, form: Option<&CreateRoleFormRequest>) -> bool {
    if !is_htmx(headers) {
        return false;
    }
    match form {
        Some(f) => f.error_redirect.as_deref().map(str::trim) == Some("/roles/new"),
        None => referer_is_wizard(headers),
    }
}

// ── Response helpers ──────────────────────────────────────────────────────────

pub fn hx_redirect_response(target: &str) -> Response {
    let hv = HeaderValue::try_from(target).unwrap_or_else(|_| HeaderValue::from_static("/roles"));
    Response::builder()
        .status(StatusCode::NO_CONTENT)
        .header(HeaderName::from_static("hx-redirect"), hv)
        .body(Body::empty())
        .unwrap()
        .into_response()
}

pub fn quick_create_shell(
    admin: &AdminClaims,
    form: &CreateRoleFormRequest,
    banner: Option<String>,
    name_error: Option<String>,
    description_error: Option<String>,
    resource_error: Option<String>,
    oob_swap: bool,
) -> QuickCreateRoleView {
    QuickCreateRoleView {
        sidebar_pinned: true,
        user_email: admin.0.email.clone(),
        css_version: env!("CSS_VERSION"),
        is_admin: admin.0.is_admin,
        nav_active: "roles",
        banner,
        name: form.name.clone(),
        description: form.description.clone(),
        name_error,
        description_error,
        resource_error,
        oob_swap,
        resources: quick_create_resource_list(),
        access_levels: quick_create_access_level_list(),
        permission_rows: quick_permission_rows_from_form(form),
    }
}

pub fn wizard_shell(
    admin: &AdminClaims,
    form: &CreateRoleFormRequest,
    banner: Option<String>,
    name_error: Option<String>,
    description_error: Option<String>,
    resource_error: Option<String>,
    oob_swap: bool,
) -> CreateRoleWizardView {
    let redirect = form
        .redirect
        .as_deref()
        .map(str::trim)
        .filter(|s: &&str| !s.is_empty())
        .unwrap_or("/roles?skip_onboarding=true")
        .to_string();
    CreateRoleWizardView {
        sidebar_pinned: true,
        user_email: admin.0.email.clone(),
        css_version: env!("CSS_VERSION"),
        is_admin: admin.0.is_admin,
        nav_active: "roles",
        banner,
        name: form.name.clone(),
        description: form.description.clone(),
        name_error,
        description_error,
        resource_error,
        oob_swap,
        resources: quick_create_resource_list(),
        access_levels: quick_create_access_level_list(),
        permission_rows: quick_permission_rows_from_form(form),
        redirect,
    }
}

pub fn empty_quick_create_form() -> CreateRoleFormRequest {
    CreateRoleFormRequest {
        name: String::new(),
        description: String::new(),
        resource: Vec::new(),
        access: Vec::new(),
        redirect: None,
        error_redirect: Some("/roles/quick".into()),
    }
}

pub fn empty_wizard_form() -> CreateRoleFormRequest {
    CreateRoleFormRequest {
        name: String::new(),
        description: String::new(),
        resource: Vec::new(),
        access: Vec::new(),
        redirect: Some("/roles?skip_onboarding=true".into()),
        error_redirect: Some("/roles/new".into()),
    }
}

// ── URL encoding utilities ────────────────────────────────────────────────────

pub fn query_param_encode(value: &str) -> String {
    value.bytes().fold(String::new(), |mut acc, b| {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                acc.push(b as char)
            }
            b' ' => acc.push('+'),
            _ => acc.push_str(&format!("%{:02X}", b)),
        }
        acc
    })
}

pub fn append_query_param(base: &str, key: &str, value: &str) -> String {
    let sep = if base.contains('?') { '&' } else { '?' };
    format!("{base}{sep}{key}={}", query_param_encode(value))
}

#[cfg(test)]
mod views_tests {
    use super::append_query_param;

    #[test]
    fn append_query_param_uses_question_mark_for_clean_urls() {
        assert_eq!(
            append_query_param("/assign", "notice", "assigned"),
            "/assign?notice=assigned"
        );
    }

    #[test]
    fn append_query_param_uses_ampersand_when_query_exists() {
        assert_eq!(
            append_query_param("/?skip_onboarding=true", "notice", "assigned"),
            "/?skip_onboarding=true&notice=assigned"
        );
    }
}
