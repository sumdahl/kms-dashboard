/// Role-related view types and HTMX handlers.
use super::views::{
    empty_quick_create_form, empty_wizard_form, hx_redirect_response, is_quick_create_htmx,
    is_wizard_htmx, query_param_encode, quick_create_shell, wizard_shell,
};
use crate::app_state::AppState;
use crate::auth::dto::{first_field_message, validate_and_parse_create_role_form};
use crate::db::Db;
use crate::error::{AppError, AppResult};
use crate::middleware::auth::AdminClaims;
use crate::models::Role;
use crate::repositories;
use crate::repositories::roles::CreateRoleRequest;
use crate::ui::global_message;
use askama::Template;
use axum::{
    extract::{Form, Path, Query, State},
    http::header::REFERER,
    http::HeaderMap,
    response::{Html, IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form as HtmlForm;
use axum_extra::extract::FormRejection;
use chrono::Utc;
use serde::Deserialize;
use std::collections::HashMap;
use uuid::Uuid;

// ── Permission row ────────────────────────────────────────────────────────────

#[derive(askama::Template)]
#[template(path = "partials/permission_row.html")]
pub struct PermissionRowTemplate {
    pub resources: Vec<&'static str>,
    pub access_levels: Vec<&'static str>,
}

pub async fn permission_row() -> PermissionRowTemplate {
    PermissionRowTemplate {
        resources: vec!["orders", "customers", "reports", "inventory", "admin_panel"],
        access_levels: vec!["read", "write", "admin"],
    }
}

// ── Query params ──────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct ListRolesQuery {
    pub page: Option<i64>,
    pub size: Option<i64>,
    pub search: Option<String>,
}

// ── Role display helpers ──────────────────────────────────────────────────────

pub struct RoleDisplay {
    pub role_id: String,
    pub name: String,
    pub name_encoded: String,
    pub description_display: String,
    pub fg_color: String,
    pub bg_color: String,
    pub initial: String,
    pub perm_badges: String,
    pub perm_count: usize,
    pub has_permissions: bool,
    pub created_at_display: String,
}

fn role_color_hue(name: &str) -> i64 {
    let mut hash: i64 = 0;
    for b in name.bytes() {
        hash = hash
            .wrapping_shl(5)
            .wrapping_sub(hash)
            .wrapping_add(b as i64);
    }
    hash.rem_euclid(360)
}

pub fn role_to_display(role: Role) -> RoleDisplay {
    let hue = role_color_hue(&role.name);
    let fg_color = format!("hsl({}, 70%, 30%)", hue);
    let bg_color = format!("hsl({}, 80%, 90%)", hue);

    let t = role.name.trim();
    let initial = if t.is_empty() {
        "?".to_string()
    } else {
        let parts: Vec<&str> = t.split_whitespace().collect();
        if parts.len() == 1 {
            parts[0].chars().take(2).collect::<String>().to_uppercase()
        } else {
            let a = parts[0].chars().next().unwrap_or('?');
            let b = parts[1].chars().next().unwrap_or('?');
            format!("{}{}", a, b).to_uppercase()
        }
    };

    let perm_count = role.permissions.len();
    let has_permissions = perm_count > 0;
    let perm_badges = if has_permissions {
        role.permissions
            .iter()
            .map(|p| p.resource.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    } else {
        "No permissions assigned".to_string()
    };

    let description_display = if role.description.is_empty() {
        "No description".to_string()
    } else {
        role.description.clone()
    };

    RoleDisplay {
        role_id: role.role_id.to_string(),
        name_encoded: query_param_encode(&role.name),
        name: role.name,
        description_display,
        fg_color,
        bg_color,
        initial,
        perm_badges,
        perm_count,
        has_permissions,
        created_at_display: role.created_at.format("%b %d, %Y").to_string(),
    }
}

// ── Paginated list data ───────────────────────────────────────────────────────

pub struct RolesListData {
    pub roles: Vec<RoleDisplay>,
    pub total: i64,
    pub page: i64,
    pub pages: i64,
    pub prev_page: i64,
    pub next_page: i64,
    pub search: String,
    pub start: i64,
    pub end: i64,
}

pub async fn load_roles_list_data(pool: &Db, page: i64, search: &str) -> AppResult<RolesListData> {
    let size = crate::utils::pagination::PAGE_SIZE;
    let data = repositories::roles::load_paginated_roles(pool, page, size, search).await?;

    let (start, end) = if data.total > 0 {
        let s = (page - 1) * size + 1;
        let e = s + data.roles.len() as i64 - 1;
        (s, e)
    } else {
        (0, 0)
    };

    let prev_page = (page - 1).max(1);
    let next_page = (page + 1).min(data.pages);

    Ok(RolesListData {
        roles: data.roles.into_iter().map(role_to_display).collect(),
        total: data.total,
        page: data.page,
        pages: data.pages,
        prev_page,
        next_page,
        search: search.to_string(),
        start,
        end,
    })
}

// ── Fragment templates ────────────────────────────────────────────────────────

#[derive(askama::Template)]
#[template(path = "dashboard/roles_list_fragment.html")]
pub struct RolesListFragment {
    pub roles: Vec<RoleDisplay>,
    pub total: i64,
    pub page: i64,
    pub pages: i64,
    pub prev_page: i64,
    pub next_page: i64,
    pub search: String,
    pub start: i64,
    pub end: i64,
}

fn roles_list_fragment_render(d: RolesListData) -> Result<String, AppError> {
    let frag = RolesListFragment {
        roles: d.roles,
        total: d.total,
        page: d.page,
        pages: d.pages,
        prev_page: d.prev_page,
        next_page: d.next_page,
        search: d.search,
        start: d.start,
        end: d.end,
    };
    frag.render().map_err(|e| AppError::Internal(e.to_string()))
}

#[derive(askama::Template)]
#[template(path = "dashboard/roles_partial.html")]
#[allow(dead_code)]
struct RolesHtmxFragment {
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

#[allow(dead_code)]
async fn roles_htmx_html(pool: &Db, _admin: &AdminClaims) -> Result<String, AppError> {
    let d = load_roles_list_data(pool, 1, "").await?;
    let summary = repositories::roles::load_roles_summary(pool).await?;
    RolesHtmxFragment {
        banner: None,
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
        summary_date: Utc::now().format("%b %d").to_string(),
    }
    .render()
    .map_err(|e| AppError::Internal(e.to_string()))
}

// ── Route handlers ────────────────────────────────────────────────────────────

pub async fn roles_list_htmx(
    _admin: AdminClaims,
    State(state): State<AppState>,
    Query(params): Query<ListRolesQuery>,
) -> AppResult<RolesListFragment> {
    let pool = state.db;
    let page = params.page.unwrap_or(1).max(1);
    let search = params.search.unwrap_or_default().trim().to_string();
    let d = load_roles_list_data(&pool, page, &search).await?;
    Ok(RolesListFragment {
        roles: d.roles,
        total: d.total,
        page: d.page,
        pages: d.pages,
        prev_page: d.prev_page,
        next_page: d.next_page,
        search: d.search,
        start: d.start,
        end: d.end,
    })
}

pub async fn delete_role_htmx(
    _admin: AdminClaims,
    State(state): State<AppState>,
    Path(role_id): Path<Uuid>,
    Form(form): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    let pool = state.db;
    let search = form
        .get("search")
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    let list_html = match load_roles_list_data(&pool, 1, &search).await {
        Ok(d) => match roles_list_fragment_render(d) {
            Ok(h) => h,
            Err(e) => {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    format!("<!-- {e} -->"),
                )
                    .into_response();
            }
        },
        Err(e) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("<!-- {e} -->"),
            )
                .into_response();
        }
    };

    match repositories::roles::delete_by_id(&pool, role_id).await {
        Ok(true) => {
            let html = match load_roles_list_data(&pool, 1, &search).await {
                Ok(d) => match roles_list_fragment_render(d) {
                    Ok(h) => h,
                    Err(e) => {
                        return (
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            format!("<!-- {e} -->"),
                        )
                            .into_response();
                    }
                },
                Err(e) => {
                    return (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        format!("<!-- {e} -->"),
                    )
                        .into_response();
                }
            };
            Html(html + &global_message::with_success("Role deleted.")).into_response()
        }
        Ok(false) => {
            Html(list_html + &global_message::with_error("Role not found or already deleted."))
                .into_response()
        }
        Err(e) => Html(list_html + &global_message::with_error(&e.to_string())).into_response(),
    }
}

pub async fn delete_role_submit(
    _admin: AdminClaims,
    State(state): State<AppState>,
    Path(role_id): Path<Uuid>,
) -> impl IntoResponse {
    let pool = state.db;
    match repositories::roles::delete_by_id(&pool, role_id).await {
        Ok(true) => Redirect::to("/roles?notice=deleted").into_response(),
        Ok(false) => Redirect::to(&format!(
            "/roles?error={}",
            query_param_encode(&AppError::RoleNotFound.to_string())
        ))
        .into_response(),
        Err(e) => Redirect::to(&format!(
            "/roles?error={}",
            query_param_encode(&e.to_string())
        ))
        .into_response(),
    }
}

fn redirect_for_invalid_role_form(headers: &HeaderMap) -> &'static str {
    match headers.get(REFERER).and_then(|v| v.to_str().ok()) {
        Some(referer) if referer.contains("/roles/new") => "/roles/new",
        _ => "/roles/quick",
    }
}

pub async fn create_role_form(
    admin: AdminClaims,
    State(state): State<AppState>,
    headers: HeaderMap,
    form: Result<HtmlForm<crate::auth::dto::CreateRoleFormRequest>, FormRejection>,
) -> Response {
    let pool = state.db;
    let fallback = redirect_for_invalid_role_form(&headers);

    let form = match form {
        Ok(f) => f.0,
        Err(e) => {
            let is_wizard = is_wizard_htmx(&headers, None);
            let is_quick = is_quick_create_htmx(&headers, None);
            let msg = format!("Form submission error: {}", e);

            if is_quick {
                let view = quick_create_shell(
                    &admin,
                    &empty_quick_create_form(),
                    Some(msg),
                    None,
                    None,
                    None,
                    true,
                );
                let html = view
                    .render()
                    .unwrap_or_else(|e| format!("<!-- template error: {e} -->"));
                return Html(html).into_response();
            }
            if is_wizard {
                let view = wizard_shell(
                    &admin,
                    &empty_wizard_form(),
                    Some(msg),
                    None,
                    None,
                    None,
                    true,
                );
                let html = view
                    .render()
                    .unwrap_or_else(|e| format!("<!-- template error: {e} -->"));
                return Html(html).into_response();
            }
            return Redirect::to(&format!(
                "{}?error={}",
                fallback,
                query_param_encode("Invalid form submission.")
            ))
            .into_response();
        }
    };

    let err_base = form
        .error_redirect
        .as_deref()
        .map(str::trim)
        .filter(|s: &&str| !s.is_empty())
        .unwrap_or("/roles/quick");

    let is_quick_htmx = is_quick_create_htmx(&headers, Some(&form));
    let is_wizard_htmx_flag = is_wizard_htmx(&headers, Some(&form));

    let permissions = match validate_and_parse_create_role_form(&form) {
        Ok(p) => p,
        Err(errs) => {
            let resource_error = first_field_message(&errs, "resource");
            let is_duplicate_error = resource_error
                .as_ref()
                .map(|e| e.contains("Duplicate"))
                .unwrap_or(false);

            if is_quick_htmx {
                let view = quick_create_shell(
                    &admin,
                    &form,
                    None,
                    first_field_message(&errs, "name"),
                    first_field_message(&errs, "description"),
                    if is_duplicate_error { None } else { resource_error },
                    true,
                );
                let mut html = view
                    .render()
                    .unwrap_or_else(|e| format!("<!-- template error: {e} -->"));
                if is_duplicate_error {
                    html.insert_str(0, &global_message::with_error("Duplicate permission rows."));
                }
                return Html(html).into_response();
            }
            if is_wizard_htmx_flag {
                let view = wizard_shell(
                    &admin,
                    &form,
                    None,
                    first_field_message(&errs, "name"),
                    first_field_message(&errs, "description"),
                    if is_duplicate_error { None } else { resource_error },
                    true,
                );
                let mut html = view
                    .render()
                    .unwrap_or_else(|e| format!("<!-- template error: {e} -->"));
                if is_duplicate_error {
                    html.insert_str(0, &global_message::with_error("Duplicate permission rows."));
                }
                return Html(html).into_response();
            }
            let msg = first_field_message(&errs, "name")
                .or_else(|| first_field_message(&errs, "description"))
                .or_else(|| first_field_message(&errs, "resource"))
                .unwrap_or_else(|| "Invalid form.".into());
            return Redirect::to(&format!("{}?error={}", err_base, query_param_encode(&msg)))
                .into_response();
        }
    };

    let req = CreateRoleRequest {
        name: form.name.trim().to_string(),
        description: form.description.trim().to_string(),
        permissions,
    };

    let ok_target = form
        .redirect
        .as_deref()
        .map(str::trim)
        .filter(|s: &&str| !s.is_empty())
        .unwrap_or("/roles?notice=created");

    match repositories::roles::persist_new_role(&pool, &req).await {
        Ok(_) => {
            if is_quick_htmx {
                let view = quick_create_shell(
                    &admin,
                    &empty_quick_create_form(),
                    None,
                    None,
                    None,
                    None,
                    true,
                );
                let mut html = view
                    .render()
                    .unwrap_or_else(|e| format!("<!-- template error: {e} -->"));
                html.push_str(&global_message::with_success("Role created successfully."));
                return Html(html).into_response();
            }
            if is_wizard_htmx_flag {
                return hx_redirect_response(&ok_target);
            }
            Redirect::to(&ok_target).into_response()
        }
        Err(e) => {
            if is_quick_htmx {
                let view = quick_create_shell(&admin, &form, None, None, None, None, true);
                let mut html = view
                    .render()
                    .unwrap_or_else(|e| format!("<!-- template error: {e} -->"));
                html.push_str(&global_message::with_error(&e.to_string()));
                return Html(html).into_response();
            }
            if is_wizard_htmx_flag {
                let view = wizard_shell(&admin, &form, None, None, None, None, true);
                let mut html = view
                    .render()
                    .unwrap_or_else(|e| format!("<!-- template error: {e} -->"));
                html.push_str(&global_message::with_error(&e.to_string()));
                return Html(html).into_response();
            }
            Redirect::to(&format!(
                "{}?error={}",
                err_base,
                query_param_encode(&e.to_string())
            ))
            .into_response()
        }
    }
}
