use crate::app_state::AppState;
use crate::db::Db;
use crate::handlers::admin::{
    fetch_all_role_names, fetch_user_summaries, load_roles_list_data, load_roles_summary,
    quick_create_access_level_list, quick_create_default_permission_rows, quick_create_resource_list,
    CreateRoleWizardView, QuickCreateRoleView, RoleDisplay, UserSummary,
};
use crate::handlers::dashboard::{load_my_roles, MyRole};
use crate::models::types::{AccessLevel, Resource};
use crate::models::{Claims, Role, RolePermission};
use crate::routes::error_page;

use crate::ui::global_message;
use askama::Template;
use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
    Router,
};
use chrono::Utc;

/// True when the request comes from HTMX navigation (not a history restore).
fn is_htmx_partial(headers: &HeaderMap) -> bool {
    headers
        .get("hx-request")
        .and_then(|v| v.to_str().ok())
        .map(|v| v == "true")
        .unwrap_or(false)
        && !headers.contains_key("hx-history-restore-request")
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(home))
        .route("/roles", get(roles_page))
        .route("/users", get(users_page))
        .route("/roles/new", get(create_role_wizard_page))
        .route("/roles/quick", get(quick_create_role_page))
        .route("/roles/:name", get(role_detail_page))
        .route("/assign", get(assign_page))
}

#[derive(serde::Deserialize)]
pub struct HomeParams {
    pub skip_onboarding: Option<bool>,
}

#[derive(serde::Deserialize, Default)]
pub struct AssignPageQuery {
    pub skip_onboarding: Option<bool>,
    pub error: Option<String>,
    pub notice: Option<String>,
    pub role: Option<String>,
}

#[derive(serde::Deserialize, Default)]
pub struct QuickCreateQuery {
    pub error: Option<String>,
}

#[derive(serde::Deserialize, Default)]
pub struct WizardPageQuery {
    pub error: Option<String>,
}

#[derive(serde::Deserialize, Default)]
pub struct UsersListQuery {
    pub flash_kind: Option<String>,
    pub flash_msg: Option<String>,
    pub error: Option<String>,
}

#[derive(askama::Template)]
#[template(path = "dashboard/users.html")]
struct UsersTemplate {
    pub sidebar_pinned: bool,
    pub user_email: String,
    pub css_version: &'static str,
    pub is_admin: bool,
    pub nav_active: &'static str,
    pub users: Vec<UserSummary>,
    pub total_users: usize,
    pub active_users: usize,
    pub disabled_users: usize,
    pub admin_users: usize,
    pub summary_date: String,
}

impl UsersTemplate {
    fn initials(full_name: &str) -> String {
        user_initials(full_name)
    }
    fn avatar_style(email: &str) -> String {
        user_avatar_style(email)
    }
}

#[derive(askama::Template)]
#[template(path = "dashboard/users_partial.html")]
struct UsersPartialTemplate {
    pub sidebar_pinned: bool,
    pub user_email: String,
    pub css_version: &'static str,
    pub is_admin: bool,
    pub nav_active: &'static str,
    pub users: Vec<UserSummary>,
    pub total_users: usize,
    pub active_users: usize,
    pub disabled_users: usize,
    pub admin_users: usize,
    pub summary_date: String,
}

impl UsersPartialTemplate {
    fn initials(full_name: &str) -> String {
        user_initials(full_name)
    }
    fn avatar_style(email: &str) -> String {
        user_avatar_style(email)
    }
}

fn user_initials(full_name: &str) -> String {
    let t = full_name.trim();
    if t.is_empty() {
        return "?".to_string();
    }
    let parts: Vec<&str> = t.split_whitespace().collect();
    if parts.len() == 1 {
        parts[0].chars().take(2).collect::<String>().to_uppercase()
    } else {
        let a = parts[0].chars().next().unwrap_or('?');
        let b = parts[1].chars().next().unwrap_or('?');
        format!("{}{}", a, b).to_uppercase()
    }
}

fn user_avatar_style(email: &str) -> String {
    let mut hash: i64 = 0;
    for b in email.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(b as i64);
    }
    let hue = hash.rem_euclid(360);
    format!(
        "background:hsl({hue},65%,28%);color:hsl({hue},75%,90%);box-shadow:inset 0 0 0 1px rgba(0,0,0,0.08)"
    )
}

async fn users_page(
    headers: HeaderMap,
    State(pool): State<Db>,
    Query(_q): Query<UsersListQuery>,
    claims: Option<Claims>,
) -> Response {
    match claims {
        None => Redirect::to("/login").into_response(),
        Some(c) if !c.is_admin => Redirect::to("/").into_response(),
        Some(c) => {
            let users = fetch_user_summaries(&pool).await.unwrap_or_default();
            let admin_users = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*)::bigint FROM users WHERE is_admin = TRUE",
            )
            .fetch_one(&pool)
            .await
            .unwrap_or(0) as usize;
            let total_users = users.len();
            let active_users = users.iter().filter(|u| u.is_active).count();
            let disabled_users = total_users.saturating_sub(active_users);
            let summary_date = Utc::now().format("%b %d").to_string();
            if is_htmx_partial(&headers) {
                UsersPartialTemplate {
                    sidebar_pinned: true,
                    user_email: c.email,
                    css_version: env!("CSS_VERSION"),
                    is_admin: c.is_admin,
                    nav_active: "users",
                    users,
                    total_users,
                    active_users,
                    disabled_users,
                    admin_users,
                    summary_date,
                }
                .into_response()
            } else {
                UsersTemplate {
                    sidebar_pinned: true,
                    user_email: c.email,
                    css_version: env!("CSS_VERSION"),
                    is_admin: c.is_admin,
                    nav_active: "users",
                    users,
                    total_users,
                    active_users,
                    disabled_users,
                    admin_users,
                    summary_date,
                }
                .into_response()
            }
        }
    }
}

#[derive(askama::Template)]
#[template(path = "dashboard/home_partial.html")]
struct HomePartialTemplate {
    pub sidebar_pinned: bool,
    pub user_email: String,
    pub css_version: &'static str,
    pub is_admin: bool,
    pub nav_active: &'static str,
    pub my_roles: Vec<MyRole>,
}

#[derive(askama::Template)]
#[template(path = "dashboard/assign_partial.html")]
struct AssignPartialTemplate {
    pub sidebar_pinned: bool,
    pub user_email: String,
    pub css_version: &'static str,
    pub is_admin: bool,
    pub nav_active: &'static str,
    pub users: Vec<UserSummary>,
    pub roles: Vec<String>,
    pub pre_role: String,
    pub error: String,
    pub notice: String,
    pub assign_redirect: String,
}

#[derive(askama::Template)]
#[template(path = "dashboard/onboarding_partial.html")]
struct OnboardingPartialTemplate {
    pub sidebar_pinned: bool,
    pub user_email: String,
    pub css_version: &'static str,
    pub is_admin: bool,
    pub current_step: u8,
    pub nav_active: &'static str,
    pub users: Vec<UserSummary>,
    pub roles: Vec<String>,
    pub pre_role: String,
    pub error: String,
    pub notice: String,
    pub assign_redirect: String,
}

#[derive(askama::Template)]
#[template(path = "dashboard/role_detail_partial.html")]
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

async fn create_role_wizard_page(
    claims: Option<Claims>,
    axum::extract::Query(q): axum::extract::Query<WizardPageQuery>,
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
            banner: q.error.filter(|e| !e.trim().is_empty()).map(|e| e),
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

async fn quick_create_role_page(
    claims: Option<Claims>,
    axum::extract::Query(q): axum::extract::Query<QuickCreateQuery>,
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
            banner: q.error.filter(|e| !e.trim().is_empty()).map(|e| e),
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

#[derive(askama::Template)]
#[template(path = "dashboard/home.html")]
struct HomeTemplate {
    sidebar_pinned: bool,
    user_email: String,
    css_version: &'static str,
    is_admin: bool,
    pub nav_active: &'static str,
    pub my_roles: Vec<MyRole>,
}

#[derive(askama::Template)]
#[template(path = "dashboard/onboarding.html")]
struct OnboardingTemplate {
    sidebar_pinned: bool,
    user_email: String,
    css_version: &'static str,
    is_admin: bool,
    current_step: u8,
    nav_active: &'static str,
    pub users: Vec<UserSummary>,
    pub roles: Vec<String>,
    pub pre_role: String,
    pub error: String,
    pub notice: String,
    pub assign_redirect: String,
}

impl AssignPartialTemplate {
    fn role_selected(&self, role_name: &str) -> bool {
        self.pre_role == role_name
    }
}

impl AssignTemplate {
    fn role_selected(&self, role_name: &str) -> bool {
        self.pre_role == role_name
    }
}

impl OnboardingPartialTemplate {
    fn role_selected(&self, role_name: &str) -> bool {
        self.pre_role == role_name
    }
}

impl OnboardingTemplate {
    fn role_selected(&self, role_name: &str) -> bool {
        self.pre_role == role_name
    }
}

async fn home(
    headers: HeaderMap,
    claims: Option<Claims>,
    State(pool): State<Db>,
    axum::extract::Query(_params): axum::extract::Query<HomeParams>,
) -> Response {
    match claims {
        None => Redirect::to("/login").into_response(),
        Some(c) => {
            let my_roles = load_my_roles(&c, &pool).await.unwrap_or_default();
            if is_htmx_partial(&headers) {
                HomePartialTemplate {
                    sidebar_pinned: true,
                    user_email: c.email,
                    css_version: env!("CSS_VERSION"),
                    is_admin: c.is_admin,
                    nav_active: "home",
                    my_roles,
                }
                .into_response()
            } else {
                HomeTemplate {
                    sidebar_pinned: true,
                    user_email: c.email,
                    css_version: env!("CSS_VERSION"),
                    is_admin: c.is_admin,
                    nav_active: "home",
                    my_roles,
                }
                .into_response()
            }
        }
    }
}

#[derive(serde::Deserialize, Default)]
pub struct RolesPageQuery {
    pub page: Option<i64>,
    pub search: Option<String>,
    pub notice: Option<String>,
    pub error: Option<String>,
    pub skip_onboarding: Option<bool>,
}

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

async fn roles_page(
    headers: HeaderMap,
    claims: Option<Claims>,
    State(pool): State<Db>,
    axum::extract::Query(params): axum::extract::Query<RolesPageQuery>,
) -> Response {
    match claims {
        None => Redirect::to("/login").into_response(),
        Some(c) if !c.is_admin => Redirect::to("/").into_response(),
        Some(c) => {
            let page = params.page.unwrap_or(1).max(1);
            let search = params.search.unwrap_or_default().trim().to_string();

            let d = load_roles_list_data(&pool, page, &search)
                .await
                .unwrap_or_else(|_| crate::handlers::admin::RolesListData {
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

            let summary = load_roles_summary(&pool).await.unwrap_or_else(|_| {
                crate::handlers::admin::RolesSummary {
                    total_roles: 0,
                    total_permissions: 0,
                    unique_resources: 0,
                    write_count: 0,
                    admin_count: 0,
                }
            });

            let summary_date = Utc::now().format("%B %d").to_string();

            let banner = match params.notice.as_deref() {
                Some("created") => Some("Role created successfully.".to_string()),
                Some("deleted") => Some("Role deleted.".to_string()),
                _ => params.error.as_ref().filter(|e| !e.trim().is_empty()).cloned(),
            };

            if is_htmx_partial(&headers) {
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

                // Clean up URL in the browser if notice/error/onboarding flags are present
                if params.notice.is_some() || params.error.is_some() || params.skip_onboarding.is_some() {
                    let mut clean_url = "/roles".to_string();
                    let mut parts = Vec::new();
                    if let Some(p) = params.page { if p > 1 { parts.push(format!("page={p}")); } }
                    if !d.search.is_empty() { parts.push(format!("search={}", d.search)); }
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

#[derive(askama::Template)]
#[template(path = "dashboard/assign.html")]
struct AssignTemplate {
    sidebar_pinned: bool,
    user_email: String,
    css_version: &'static str,
    is_admin: bool,
    nav_active: &'static str,
    pub users: Vec<UserSummary>,
    pub roles: Vec<String>,
    pub pre_role: String,
    pub error: String,
    pub notice: String,
    pub assign_redirect: String,
}

async fn assign_page(
    headers: HeaderMap,
    claims: Option<Claims>,
    State(pool): State<Db>,
    axum::extract::Query(params): axum::extract::Query<AssignPageQuery>,
) -> Response {
    match claims {
        None => Redirect::to("/login").into_response(),
        Some(c) if !c.is_admin => Redirect::to("/").into_response(),
        Some(c) => {
            let users = fetch_user_summaries(&pool).await.unwrap_or_default();
            let roles = fetch_all_role_names(&pool).await.unwrap_or_default();
            let pre_role = params.role.clone().unwrap_or_default();
            let skip = params.skip_onboarding.unwrap_or(false);
            let htmx = is_htmx_partial(&headers);

            let banner = match params.notice.as_deref() {
                Some("assigned") => Some("Role assigned successfully.".to_string()),
                _ => params.error.filter(|e| !e.trim().is_empty()),
            };

            let notice_text = banner.clone().unwrap_or_default();

            if skip {
                if htmx {
                    let mut res = AssignPartialTemplate {
                        sidebar_pinned: true,
                        user_email: c.email.clone(),
                        css_version: env!("CSS_VERSION"),
                        is_admin: c.is_admin,
                        nav_active: "assign",
                        users: users.clone(),
                        roles: roles.clone(),
                        pre_role: pre_role.clone(),
                        error: String::new(),
                        notice: notice_text.clone(),
                        assign_redirect: "/assign".to_string(),
                    }
                    .into_response();

                    if banner.is_some() || params.skip_onboarding.is_some() {
                        let hv = axum::http::HeaderValue::from_static("/assign");
                        res.headers_mut().insert("HX-Replace-Url", hv);
                    }
                    res
                } else {
                    AssignTemplate {
                        sidebar_pinned: true,
                        user_email: c.email,
                        css_version: env!("CSS_VERSION"),
                        is_admin: c.is_admin,
                        nav_active: "assign",
                        users,
                        roles,
                        pre_role,
                        error: String::new(),
                        notice: notice_text,
                        assign_redirect: "/assign".to_string(),
                    }
                    .into_response()
                }
            } else if htmx {
                let mut res = OnboardingPartialTemplate {
                    sidebar_pinned: true,
                    user_email: c.email.clone(),
                    css_version: env!("CSS_VERSION"),
                    is_admin: c.is_admin,
                    current_step: 2,
                    nav_active: "assign",
                    users: users.clone(),
                    roles: roles.clone(),
                    pre_role: pre_role.clone(),
                    error: String::new(),
                    notice: notice_text.clone(),
                    assign_redirect: "/?skip_onboarding=true".to_string(),
                }
                .into_response();

                if banner.is_some() {
                    let hv = axum::http::HeaderValue::from_static("/assign");
                    res.headers_mut().insert("HX-Replace-Url", hv);
                }
                res
            } else {
                OnboardingTemplate {
                    sidebar_pinned: true,
                    user_email: c.email,
                    css_version: env!("CSS_VERSION"),
                    is_admin: c.is_admin,
                    current_step: 2,
                    nav_active: "assign",
                    users,
                    roles,
                    pre_role,
                    error: String::new(),
                    notice: notice_text,
                    assign_redirect: "/?skip_onboarding=true".to_string(),
                }
                .into_response()
            }
        }
    }
}

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

async fn role_detail_page(
    headers: HeaderMap,
    claims: Option<Claims>,
    State(pool): State<Db>,
    Path(role_name): Path<String>,
) -> Response {
    let c = match claims {
        None => return Redirect::to("/login").into_response(),
        Some(c) => c,
    };

    let mut role = match sqlx::query_as::<_, Role>(
        "SELECT role_id, name, description, created_at FROM roles WHERE name = $1",
    )
    .bind(&role_name)
    .fetch_optional(&pool)
    .await
    {
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

    let perms =
        match sqlx::query("SELECT resource, access_level FROM role_permissions WHERE role_id = $1")
            .bind(role.role_id)
            .fetch_all(&pool)
            .await
        {
            Ok(rows) => rows,
            Err(_) => vec![],
        };

    use sqlx::Row;
    role.permissions = perms
        .into_iter()
        .map(|p| {
            let res_str: String = p.get("resource");
            let acc_str: String = p.get("access_level");
            RolePermission {
                resource: res_str.parse().unwrap_or(Resource::Orders),
                access: acc_str.parse().unwrap_or(AccessLevel::Read),
            }
        })
        .collect();

    let permissions: Vec<(String, String)> = role
        .permissions
        .iter()
        .map(|p| (p.resource.to_string(), p.access.to_string()))
        .collect();

    let assignment_rows = match sqlx::query(
        "SELECT u.email, ra.assigned_at, ra.expires_at,
                CASE WHEN ra.expires_at IS NULL OR ra.expires_at > NOW() THEN true ELSE false END AS is_active
         FROM role_assignments ra
         JOIN users u ON u.user_id = ra.user_id
         WHERE ra.role_id = $1
         ORDER BY ra.assigned_at DESC",
    )
    .bind(role.role_id)
    .fetch_all(&pool)
    .await
    {
        Ok(rows) => rows,
        Err(_) => vec![],
    };

    let assignments: Vec<(String, String, String, bool)> = assignment_rows
        .into_iter()
        .map(|r| {
            let email: String = r.get("email");
            let assigned_at: chrono::DateTime<chrono::Utc> = r.get("assigned_at");
            let expires_at: Option<chrono::DateTime<chrono::Utc>> = r.get("expires_at");
            let is_active: bool = r.get("is_active");
            (
                email,
                assigned_at.format("%b %d, %Y").to_string(),
                expires_at
                    .map(|e| e.format("%b %d, %Y %H:%M").to_string())
                    .unwrap_or_default(),
                is_active,
            )
        })
        .collect();

    let assignments_len = assignments.len();

    let role_id_short = role.role_id.to_string().chars().take(8).collect();
    let created_at_display = role.created_at.format("%b %d, %Y").to_string();
    let permissions_len = permissions.len();

    if is_htmx_partial(&headers) {
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
