/// Roles-related page route handlers (list, detail, create forms).
use crate::app_state::AppState;
use crate::handlers::admin::{
    load_roles_list_data, load_roles_summary, quick_create_access_level_list,
    quick_create_default_permission_rows, quick_create_resource_list, CreateRoleWizardView,
    QuickCreateRoleView, RoleDisplay, RolesListData, RolesSummary,
};
use crate::models::Claims;
use crate::repositories;
use crate::routes::error_page;
use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    response::{IntoResponse, Redirect, Response},
};
use chrono::Utc;

// ── Query params ──────────────────────────────────────────────────────────────

#[derive(serde::Deserialize, Default)]
pub struct RolesPageQuery {
    pub page: Option<i64>,
    pub search: Option<String>,
    pub notice: Option<String>,
    pub error: Option<String>,
    pub skip_onboarding: Option<bool>,
}

#[derive(serde::Deserialize, Default)]
pub struct QuickCreateQuery {
    pub error: Option<String>,
}

#[derive(serde::Deserialize, Default)]
pub struct WizardPageQuery {
    pub error: Option<String>,
}

// ── Roles list templates ──────────────────────────────────────────────────────

#[derive(askama::Template)]
#[template(path = "dashboard/roles.html")]
struct RolesTemplate {
    sidebar_pinned: bool,
    user_email: String,
    css_version: &'static str,
    is_admin: bool,
    nav_active: &'static str,
    banner: Option<String>,
    roles: Vec<RoleDisplay>,
    total: i64,
    page: i64,
    pages: i64,
    prev_page: i64,
    next_page: i64,
    search: String,
    start: i64,
    end: i64,
    summary_total: i64,
    summary_perms: i64,
    summary_res: i64,
    summary_admin: i64,
    summary_date: String,
}

#[derive(askama::Template)]
#[template(path = "dashboard/roles_partial.html")]
#[allow(dead_code)]
struct RolesPartialTemplate {
    sidebar_pinned: bool,
    user_email: String,
    css_version: &'static str,
    is_admin: bool,
    nav_active: &'static str,
    banner: Option<String>,
    roles: Vec<RoleDisplay>,
    total: i64,
    page: i64,
    pages: i64,
    prev_page: i64,
    next_page: i64,
    search: String,
    start: i64,
    end: i64,
    summary_total: i64,
    summary_perms: i64,
    summary_res: i64,
    summary_admin: i64,
    summary_date: String,
}

// ── Role detail templates ─────────────────────────────────────────────────────

#[derive(askama::Template)]
#[template(path = "dashboard/role_detail.html")]
struct RoleDetailTemplate {
    sidebar_pinned: bool,
    user_email: String,
    css_version: &'static str,
    is_admin: bool,
    nav_active: &'static str,
    role_name: String,
    role_description: String,
    role_id_short: String,
    created_at_display: String,
    permissions_len: usize,
    permissions: Vec<(String, String)>,
    assignments_len: usize,
    assignments: Vec<(String, String, String, bool)>,
}

#[derive(askama::Template)]
#[template(path = "dashboard/role_detail_partial.html")]
#[allow(dead_code)]
struct RoleDetailPartialTemplate {
    pub sidebar_pinned: bool,
    pub user_email: String,
    pub css_version: &'static str,
    pub is_admin: bool,
    pub nav_active: &'static str,
    pub role_name: String,
    pub role_description: String,
    pub role_id_short: String,
    pub created_at_display: String,
    pub permissions_len: usize,
    pub permissions: Vec<(String, String)>,
    pub assignments_len: usize,
    pub assignments: Vec<(String, String, String, bool)>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn roles_page(
    headers: HeaderMap,
    claims: Option<Claims>,
    State(state): State<AppState>,
    Query(params): Query<RolesPageQuery>,
) -> Response {
    let pool = state.db;
    match claims {
        None => Redirect::to("/login").into_response(),
        Some(c) if !c.is_admin => Redirect::to("/").into_response(),
        Some(c) => {
            let page = params.page.unwrap_or(1).max(1);
            let search = params.search.unwrap_or_default().trim().to_string();

            let d = load_roles_list_data(&pool, page, &search)
                .await
                .unwrap_or_else(|_| RolesListData {
                    roles: vec![],
                    total: 0,
                    page: 1,
                    pages: 1,
                    prev_page: 1,
                    next_page: 1,
                    search: search.clone(),
                    start: 0,
                    end: 0,
                });

            let summary = load_roles_summary(&pool)
                .await
                .unwrap_or_else(|_| RolesSummary {
                    total_roles: 0,
                    total_permissions: 0,
                    unique_resources: 0,
                    write_count: 0,
                    admin_count: 0,
                });

            let summary_date = Utc::now().format("%B %d").to_string();

            let banner = match params.notice.as_deref() {
                Some("created") => Some("Role created successfully.".to_string()),
                Some("deleted") => Some("Role deleted.".to_string()),
                _ => params
                    .error
                    .as_ref()
                    .filter(|e| !e.trim().is_empty())
                    .cloned(),
            };

            if super::is_htmx_partial(&headers) {
                let mut res = RolesPartialTemplate {
                    sidebar_pinned: true,
                    user_email: c.email,
                    css_version: env!("CSS_VERSION"),
                    is_admin: c.is_admin,
                    nav_active: "roles",
                    banner,
                    roles: d.roles,
                    total: d.total,
                    page: d.page,
                    pages: d.pages,
                    prev_page: d.prev_page,
                    next_page: d.next_page,
                    search: d.search.clone(),
                    start: d.start,
                    end: d.end,
                    summary_total: summary.total_roles,
                    summary_perms: summary.total_permissions,
                    summary_res: summary.unique_resources,
                    summary_admin: summary.admin_count,
                    summary_date,
                }
                .into_response();

                if params.notice.is_some()
                    || params.error.is_some()
                    || params.skip_onboarding.is_some()
                {
                    let mut clean_url = "/roles".to_string();
                    let mut parts = Vec::new();
                    if let Some(p) = params.page {
                        if p > 1 {
                            parts.push(format!("page={p}"));
                        }
                    }
                    if !d.search.is_empty() {
                        parts.push(format!("search={}", d.search));
                    }
                    if !parts.is_empty() {
                        clean_url.push('?');
                        clean_url.push_str(&parts.join("&"));
                    }
                    if let Ok(hv) = axum::http::HeaderValue::from_str(&clean_url) {
                        res.headers_mut().insert("HX-Replace-Url", hv);
                    }
                }
                res
            } else {
                RolesTemplate {
                    sidebar_pinned: true,
                    user_email: c.email,
                    css_version: env!("CSS_VERSION"),
                    is_admin: c.is_admin,
                    nav_active: "roles",
                    banner,
                    roles: d.roles,
                    total: d.total,
                    page: d.page,
                    pages: d.pages,
                    prev_page: d.prev_page,
                    next_page: d.next_page,
                    search: d.search,
                    start: d.start,
                    end: d.end,
                    summary_total: summary.total_roles,
                    summary_perms: summary.total_permissions,
                    summary_res: summary.unique_resources,
                    summary_admin: summary.admin_count,
                    summary_date,
                }
                .into_response()
            }
        }
    }
}

pub async fn create_role_wizard_page(
    claims: Option<Claims>,
    Query(q): Query<WizardPageQuery>,
) -> Response {
    match claims {
        None => Redirect::to("/login").into_response(),
        Some(c) if !c.is_admin => Redirect::to("/").into_response(),
        Some(c) => CreateRoleWizardView {
            sidebar_pinned: true,
            user_email: c.email,
            css_version: env!("CSS_VERSION"),
            is_admin: c.is_admin,
            nav_active: "roles",
            banner: q.error.filter(|e| !e.trim().is_empty()),
            name: String::new(),
            description: String::new(),
            name_error: None,
            description_error: None,
            resource_error: None,
            oob_swap: false,
            resources: quick_create_resource_list(),
            access_levels: quick_create_access_level_list(),
            permission_rows: quick_create_default_permission_rows(),
            redirect: "/roles?notice=created".to_string(),
        }
        .into_response(),
    }
}

pub async fn quick_create_role_page(
    claims: Option<Claims>,
    Query(q): Query<QuickCreateQuery>,
) -> Response {
    match claims {
        None => Redirect::to("/login").into_response(),
        Some(c) if !c.is_admin => Redirect::to("/").into_response(),
        Some(c) => QuickCreateRoleView {
            sidebar_pinned: true,
            user_email: c.email,
            css_version: env!("CSS_VERSION"),
            is_admin: c.is_admin,
            nav_active: "roles",
            banner: q.error.filter(|e| !e.trim().is_empty()),
            name: String::new(),
            description: String::new(),
            name_error: None,
            description_error: None,
            resource_error: None,
            oob_swap: false,
            resources: quick_create_resource_list(),
            access_levels: quick_create_access_level_list(),
            permission_rows: quick_create_default_permission_rows(),
        }
        .into_response(),
    }
}

pub async fn role_detail_page(
    headers: HeaderMap,
    claims: Option<Claims>,
    State(state): State<AppState>,
    Path(role_id): Path<uuid::Uuid>,
) -> Response {
    let pool = state.db;
    let c = match claims {
        None => return Redirect::to("/login").into_response(),
        Some(c) => c,
    };

    let role = match repositories::roles::find_with_permissions(&pool, role_id).await {
        Ok(Some(r)) => r,
        _ => {
            return error_page(
                404,
                "Not Found",
                "The role you are looking for does not exist.",
            )
            .into_response()
        }
    };

    let permissions: Vec<(String, String)> = role
        .permissions
        .iter()
        .map(|p| (p.resource.to_string(), p.access.to_string()))
        .collect();

    let raw_assignments =
        repositories::assignments::find_by_role_with_users(&pool, role.role_id)
            .await
            .unwrap_or_default();

    let assignments: Vec<(String, String, String, bool)> = raw_assignments
        .into_iter()
        .map(|a| {
            (
                a.email,
                a.assigned_at.format("%b %d, %Y").to_string(),
                a.expires_at
                    .map(|e| e.format("%b %d, %Y %H:%M").to_string())
                    .unwrap_or_default(),
                a.is_active,
            )
        })
        .collect();

    let assignments_len = assignments.len();
    let role_id_short = role.role_id.to_string().chars().take(8).collect();
    let created_at_display = role.created_at.format("%b %d, %Y").to_string();
    let permissions_len = permissions.len();

    if super::is_htmx_partial(&headers) {
        RoleDetailPartialTemplate {
            sidebar_pinned: true,
            user_email: c.email,
            css_version: env!("CSS_VERSION"),
            is_admin: c.is_admin,
            nav_active: "roles",
            role_name: role.name,
            role_description: role.description,
            role_id_short,
            created_at_display,
            permissions_len,
            permissions,
            assignments_len,
            assignments,
        }
        .into_response()
    } else {
        RoleDetailTemplate {
            sidebar_pinned: true,
            user_email: c.email,
            css_version: env!("CSS_VERSION"),
            is_admin: c.is_admin,
            nav_active: "roles",
            role_name: role.name,
            role_description: role.description,
            role_id_short,
            created_at_display,
            permissions_len,
            permissions,
            assignments_len,
            assignments,
        }
        .into_response()
    }
}
