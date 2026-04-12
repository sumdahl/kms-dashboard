use crate::db::Db;
use crate::error::{AppError, AppResult};
use crate::middleware::rbac::Permissions;
use crate::models::types::{AccessLevel, Resource};
use crate::models::{Claims, Role, RolePermission, user::UserSummary};
use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::{Form, Path, Query, State};
use axum::http::StatusCode;
use axum::response::{Html, Redirect, Response};
use axum::Json;
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub async fn inventory_status(
    perms: Permissions,
) -> AppResult<Json<serde_json::Value>> {
    perms.require(Resource::Inventory, AccessLevel::Read)?;

    Ok(Json(serde_json::json!({
        "status": "online",
        "items_count": 150,
        "message": "You have active access to inventory data."
    })))
}

// --- Data Structs ---

#[derive(Serialize)]
pub struct MyPermission {
    pub resource: String,
    pub access: String,
}

#[derive(Serialize)]
pub struct MyRole {
    pub name: String,
    pub description: String,
    pub permissions: Vec<MyPermission>,
    pub expires_at: Option<String>,
}

pub async fn my_roles(claims: Claims, State(pool): State<Db>) -> AppResult<Json<Vec<MyRole>>> {
    use sqlx::Row;

    let user_id = claims
        .sub
        .parse::<Uuid>()
        .map_err(|_| crate::error::AppError::Unauthorized)?;

    let rows = sqlx::query(
        r#"
        SELECT r.role_id, r.name, r.description, ra.expires_at
        FROM role_assignments ra
        JOIN roles r ON ra.role_id = r.role_id
        WHERE ra.user_id = $1
          AND (ra.expires_at IS NULL OR ra.expires_at > NOW())
        ORDER BY r.name
        "#,
    )
    .bind(user_id)
    .fetch_all(&pool)
    .await?;

    let mut result: Vec<MyRole> = Vec::new();

    for row in rows {
        let role_id: Uuid = row.get("role_id");
        let name: String = row.get("name");
        let description: String = row.get("description");
        let expires_at: Option<chrono::DateTime<chrono::Utc>> = row.get("expires_at");

        let perm_rows =
            sqlx::query("SELECT resource, access_level FROM role_permissions WHERE role_id = $1")
                .bind(role_id)
                .fetch_all(&pool)
                .await?;

        let permissions = perm_rows
            .iter()
            .map(|p| MyPermission {
                resource: p.get::<String, _>("resource"),
                access: p.get::<String, _>("access_level"),
            })
            .collect();

        result.push(MyRole {
            name,
            description,
            permissions,
            expires_at: expires_at.map(|e| e.to_rfc3339()),
        });
    }

    Ok(Json(result))
}

// --- Templates ---

#[derive(Template)]
#[template(path = "dashboard/home.html")]
pub struct HomeTemplate {
    pub sidebar_pinned: bool,
    pub user_email: String,
    pub show_banner: bool,
    pub css_version: &'static str,
    pub is_admin: bool,
}

#[derive(Template)]
#[template(path = "dashboard/roles.html")]
pub struct RolesTemplate {
    pub sidebar_pinned: bool,
    pub user_email: String,
    pub show_banner: bool,
    pub css_version: &'static str,
    pub is_admin: bool,
}

#[derive(Template)]
#[template(path = "dashboard/users.html")]
pub struct UsersTemplate {
    pub sidebar_pinned: bool,
    pub user_email: String,
    pub show_banner: bool,
    pub css_version: &'static str,
    pub is_admin: bool,
}

#[derive(Template)]
#[template(path = "dashboard/assign.html")]
pub struct AssignTemplate {
    pub sidebar_pinned: bool,
    pub user_email: String,
    pub show_banner: bool,
    pub css_version: &'static str,
    pub is_admin: bool,
    pub users: Vec<UserSummary>,
    pub roles: Vec<Role>,
    pub pre_role: Option<String>,
}

impl AssignTemplate {
    pub fn is_pre_selected(&self, role_name: &str) -> bool {
        self.pre_role.as_deref() == Some(role_name)
    }
}

#[derive(Template)]
#[template(path = "dashboard/onboarding.html")]
pub struct OnboardingTemplate {
    pub sidebar_pinned: bool,
    pub user_email: String,
    pub show_banner: bool,
    pub css_version: &'static str,
    pub is_admin: bool,
    pub current_step: u8,
}

#[derive(Template)]
#[template(path = "dashboard/create_role_wizard.html")]
pub struct CreateRoleWizardTemplate {
    pub sidebar_pinned: bool,
    pub user_email: String,
    pub show_banner: bool,
    pub css_version: &'static str,
    pub is_admin: bool,
    pub step: u8,
    pub role_name: String,
    pub role_description: String,
    pub error: String,
    pub permissions: Vec<(String, String)>,
}

#[derive(Template)]
#[template(path = "dashboard/quick_create_role.html")]
pub struct QuickCreateRoleTemplate {
    pub sidebar_pinned: bool,
    pub user_email: String,
    pub show_banner: bool,
    pub css_version: &'static str,
    pub is_admin: bool,
}

#[derive(Template)]
#[template(path = "dashboard/role_detail.html")]
pub struct RoleDetailTemplate {
    pub sidebar_pinned: bool,
    pub user_email: String,
    pub show_banner: bool,
    pub css_version: &'static str,
    pub is_admin: bool,
    pub role_name: String,
    pub role_description: String,
    pub role_id_short: String,
    pub created_at_display: String,
    pub permissions_len: usize,
    pub permissions: Vec<(String, String)>,
    pub assignments_len: usize,
    pub assignments: Vec<(String, String, String, bool)>,
}

#[derive(Template)]
#[template(path = "partials/sidebar.html")]
pub struct SidebarTemplate {
    pub sidebar_pinned: bool,
    pub user_email: String,
    pub is_admin: bool,
}

#[derive(Deserialize)]
pub struct HomeParams {
    pub skip_onboarding: Option<bool>,
}

// --- Helpers ---

fn get_sidebar_pinned(jar: &CookieJar) -> bool {
    jar.get("sidebar_pinned")
        .map(|c| c.value() == "true")
        .unwrap_or(true) // Default to pinned
}

// --- Handlers ---

pub async fn home(claims: Option<Claims>, jar: CookieJar) -> Response {
    match claims {
        None => Redirect::to("/login").into_response(),
        Some(c) => HomeTemplate {
            sidebar_pinned: get_sidebar_pinned(&jar),
            user_email: c.email,
            show_banner: true,
            css_version: env!("CSS_VERSION"),
            is_admin: c.is_admin,
        }
        .into_response(),
    }
}

pub async fn roles_page(claims: Option<Claims>, jar: CookieJar) -> Response {
    match claims {
        None => Redirect::to("/login").into_response(),
        Some(c) => RolesTemplate {
            sidebar_pinned: get_sidebar_pinned(&jar),
            user_email: c.email,
            show_banner: false,
            css_version: env!("CSS_VERSION"),
            is_admin: c.is_admin,
        }
        .into_response(),
    }
}

pub async fn users_page(claims: Option<Claims>, jar: CookieJar) -> Response {
    match claims {
        None => Redirect::to("/login").into_response(),
        Some(c) => UsersTemplate {
            sidebar_pinned: get_sidebar_pinned(&jar),
            user_email: c.email,
            show_banner: false,
            css_version: env!("CSS_VERSION"),
            is_admin: c.is_admin,
        }
        .into_response(),
    }
}

#[derive(Deserialize)]
pub struct AssignParams {
    pub role: Option<String>,
}

pub async fn assign_page(
    claims: Option<Claims>,
    jar: CookieJar,
    State(pool): State<Db>,
    Query(params): Query<HomeParams>,
    Query(assign_params): Query<AssignParams>,
) -> Response {
    let c = match claims {
        None => return Redirect::to("/login").into_response(),
        Some(c) => c,
    };

    let skip = params.skip_onboarding.unwrap_or(false);
    let pinned = get_sidebar_pinned(&jar);

    if !skip {
        return OnboardingTemplate {
            sidebar_pinned: pinned,
            user_email: c.email,
            show_banner: false,
            css_version: env!("CSS_VERSION"),
            is_admin: c.is_admin,
            current_step: 2,
        }
        .into_response();
    }

    // Fetch users
    let users = match sqlx::query_as::<_, UserSummary>(
        "SELECT user_id, email, full_name, is_admin, is_active, disabled_reason
         FROM users
         WHERE is_admin = FALSE
         ORDER BY created_at DESC",
    )
    .fetch_all(&pool)
    .await {
        Ok(u) => u,
        Err(_) => Vec::new(),
    };

    // Fetch roles
    let roles = match sqlx::query_as::<_, Role>(
        "SELECT role_id, name, description, created_at FROM roles ORDER BY name ASC",
    )
    .fetch_all(&pool)
    .await {
        Ok(r) => r,
        Err(_) => Vec::new(),
    };

    AssignTemplate {
        sidebar_pinned: pinned,
        user_email: c.email,
        show_banner: false,
        css_version: env!("CSS_VERSION"),
        is_admin: c.is_admin,
        users,
        roles,
        pre_role: assign_params.role,
    }
    .into_response()
}

pub async fn create_role_wizard_page(claims: Option<Claims>, jar: CookieJar) -> Response {
    match claims {
        None => Redirect::to("/login").into_response(),
        Some(c) => CreateRoleWizardTemplate {
            sidebar_pinned: get_sidebar_pinned(&jar),
            user_email: c.email,
            show_banner: false,
            css_version: env!("CSS_VERSION"),
            is_admin: c.is_admin,
            step: 1,
            role_name: String::new(),
            role_description: String::new(),
            error: String::new(),
            permissions: Vec::new(),
        }
        .into_response(),
    }
}

pub async fn quick_create_role_page(claims: Option<Claims>, jar: CookieJar) -> Response {
    match claims {
        None => Redirect::to("/login").into_response(),
        Some(c) => QuickCreateRoleTemplate {
            sidebar_pinned: get_sidebar_pinned(&jar),
            user_email: c.email,
            show_banner: false,
            css_version: env!("CSS_VERSION"),
            is_admin: c.is_admin,
        }
        .into_response(),
    }
}

pub async fn role_detail_page(
    claims: Option<Claims>,
    jar: CookieJar,
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
        _ => return crate::routes::error_page_response(404, "Not Found", "Role not found").into_response(),
    };

    let perms = sqlx::query("SELECT resource, access_level FROM role_permissions WHERE role_id = $1")
        .bind(role.role_id)
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

    use sqlx::Row;
    role.permissions = perms.into_iter().map(|p| {
        RolePermission {
            resource: p.get::<String, _>("resource").parse().unwrap_or(Resource::Orders),
            access: p.get::<String, _>("access_level").parse().unwrap_or(AccessLevel::Read),
        }
    }).collect();

    let assignment_rows = sqlx::query(
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
    .unwrap_or_default();

    let assignments = assignment_rows.into_iter().map(|r| {
        let email: String = r.get("email");
        let assigned_at: chrono::DateTime<chrono::Utc> = r.get("assigned_at");
        let expires_at: Option<chrono::DateTime<chrono::Utc>> = r.get("expires_at");
        let is_active: bool = r.get("is_active");
        (
            email,
            assigned_at.format("%b %d, %Y").to_string(),
            expires_at.map(|e| e.format("%b %d, %Y %H:%M").to_string()).unwrap_or_default(),
            is_active,
        )
    }).collect::<Vec<_>>();

    RoleDetailTemplate {
        sidebar_pinned: get_sidebar_pinned(&jar),
        user_email: c.email,
        show_banner: false,
        css_version: env!("CSS_VERSION"),
        is_admin: c.is_admin,
        role_name: role.name,
        role_description: role.description,
        role_id_short: role.role_id.to_string().chars().take(8).collect(),
        created_at_display: role.created_at.format("%b %d, %Y").to_string(),
        permissions_len: role.permissions.len(),
        permissions: role.permissions.iter().map(|p| (p.resource.to_string(), p.access.to_string())).collect(),
        assignments_len: assignments.len(),
        assignments,
    }
    .into_response()
}

#[derive(Deserialize)]
pub struct SidebarPinForm {
    pub pinned: String,
}

pub async fn sidebar_pin(
    claims: Claims,
    jar: CookieJar,
    Form(form): Form<SidebarPinForm>
) -> impl IntoResponse {
    let is_pinned = form.pinned == "true";
    
    let cookie = Cookie::build(("sidebar_pinned", is_pinned.to_string()))
        .path("/")
        .same_site(SameSite::Lax)
        .build();

    let template = SidebarTemplate {
        sidebar_pinned: is_pinned,
        user_email: claims.email,
        is_admin: claims.is_admin,
    };

    // We return the sidebar HTML AND a script to toggle the body class
    let script = if is_pinned {
        "<script>document.body.classList.add('sidebar-expanded');</script>"
    } else {
        "<script>document.body.classList.remove('sidebar-expanded');</script>"
    };

    (jar.add(cookie), Html(format!("{}{}", template.render().unwrap(), script)))
}

pub async fn banner_dismiss() -> Html<&'static str> {
    Html("")
}
