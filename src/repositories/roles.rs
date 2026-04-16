use crate::db::Db;
use crate::error::{AppError, AppResult};
use crate::models::types::{AccessLevel, Resource, RolePermissionInput};
use crate::models::{Role, RolePermission};
use serde::Serialize;
use uuid::Uuid;

// ── Data transfer types ───────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct PaginatedRoles {
    pub roles: Vec<Role>,
    pub total: i64,
    pub page: i64,
    pub size: i64,
    pub pages: i64,
}

#[derive(Serialize)]
pub struct RolesSummary {
    pub total_roles: i64,
    pub total_permissions: i64,
    pub unique_resources: i64,
    pub write_count: i64,
    pub admin_count: i64,
}

pub struct CreateRoleRequest {
    pub name: String,
    pub description: String,
    pub permissions: Vec<RolePermissionInput>,
}

// ── Query functions ───────────────────────────────────────────────────────────

/// Paginated role list with optional search filter, permissions populated.
pub async fn load_paginated_roles(
    pool: &Db,
    page: i64,
    size: i64,
    search: &str,
) -> AppResult<PaginatedRoles> {
    let (total, mut roles) = if search.is_empty() {
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM roles")
            .fetch_one(pool)
            .await?;

        let roles = sqlx::query_as::<_, Role>(
            "SELECT role_id, name, description, created_at
             FROM roles ORDER BY created_at DESC
             LIMIT $1 OFFSET $2",
        )
        .bind(size)
        .bind((page - 1) * size)
        .fetch_all(pool)
        .await?;

        (total, roles)
    } else {
        let pattern = format!("%{}%", search);

        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM roles
             WHERE name ILIKE $1 OR description ILIKE $1",
        )
        .bind(&pattern)
        .fetch_one(pool)
        .await?;

        let roles = sqlx::query_as::<_, Role>(
            "SELECT role_id, name, description, created_at
             FROM roles
             WHERE name ILIKE $1 OR description ILIKE $1
             ORDER BY created_at DESC
             LIMIT $2 OFFSET $3",
        )
        .bind(&pattern)
        .bind(size)
        .bind((page - 1) * size)
        .fetch_all(pool)
        .await?;

        (total, roles)
    };

    for role in &mut roles {
        let perms =
            sqlx::query("SELECT resource, access_level FROM role_permissions WHERE role_id = $1")
                .bind(role.role_id)
                .fetch_all(pool)
                .await?;

        role.permissions = perms
            .into_iter()
            .map(|p| {
                use sqlx::Row;
                let res_str: String = p.get("resource");
                let acc_str: String = p.get("access_level");
                RolePermission {
                    resource: res_str.parse().unwrap_or(Resource::Orders),
                    access: acc_str.parse().unwrap_or(AccessLevel::Read),
                }
            })
            .collect();
    }

    let pages = if total == 0 {
        1
    } else {
        (total + size - 1) / size
    };

    Ok(PaginatedRoles {
        roles,
        total,
        page,
        size,
        pages,
    })
}

/// All role names ordered alphabetically.
pub async fn fetch_all_role_names(pool: &Db) -> AppResult<Vec<String>> {
    sqlx::query_scalar("SELECT name FROM roles ORDER BY name ASC")
        .fetch_all(pool)
        .await
        .map_err(Into::into)
}

/// Aggregate stats across roles and their permissions.
pub async fn load_roles_summary(pool: &Db) -> AppResult<RolesSummary> {
    let row = sqlx::query(
        "SELECT
            (SELECT COUNT(*) FROM roles)                AS total_roles,
            COUNT(*)                                    AS total_permissions,
            COUNT(DISTINCT resource)                    AS unique_resources,
            COUNT(*) FILTER (WHERE access_level = 'write') AS write_count,
            COUNT(*) FILTER (WHERE access_level = 'admin') AS admin_count
         FROM role_permissions",
    )
    .fetch_one(pool)
    .await?;

    use sqlx::Row;
    Ok(RolesSummary {
        total_roles: row.get("total_roles"),
        total_permissions: row.get("total_permissions"),
        unique_resources: row.get("unique_resources"),
        write_count: row.get("write_count"),
        admin_count: row.get("admin_count"),
    })
}

/// Create role + permissions in a single transaction.
pub async fn persist_new_role(pool: &Db, payload: &CreateRoleRequest) -> AppResult<Uuid> {
    let mut tx = pool.begin().await?;
    let role_id = Uuid::new_v4();

    let insert_result =
        sqlx::query("INSERT INTO roles (role_id, name, description) VALUES ($1, $2, $3)")
            .bind(role_id)
            .bind(&payload.name)
            .bind(&payload.description)
            .execute(&mut *tx)
            .await;

    match insert_result {
        Ok(_) => {}
        Err(sqlx::Error::Database(db_err)) if db_err.code().as_deref() == Some("23505") => {
            return Err(AppError::Conflict(format!(
                "A role named '{}' already exists.",
                payload.name
            )));
        }
        Err(e) => return Err(e.into()),
    }

    for perm in &payload.permissions {
        let resource_str = perm.resource.to_string();
        let access_str = perm.access.to_string();

        match sqlx::query(
            "INSERT INTO role_permissions (role_id, resource, access_level) VALUES ($1, $2, $3)",
        )
        .bind(role_id)
        .bind(&resource_str)
        .bind(&access_str)
        .execute(&mut *tx)
        .await
        {
            Ok(_) => {}
            Err(sqlx::Error::Database(db_err)) if db_err.code().as_deref() == Some("23505") => {
                return Err(AppError::Conflict(format!(
                    "Duplicate permission: '{}' with '{}' is already assigned to this role.",
                    resource_str, access_str
                )));
            }
            Err(e) => return Err(e.into()),
        }
    }

    tx.commit().await?;
    Ok(role_id)
}

/// Delete role by ID. Returns true if a row was deleted.
pub async fn delete_by_id(pool: &Db, role_id: Uuid) -> AppResult<bool> {
    let result = sqlx::query("DELETE FROM roles WHERE role_id = $1")
        .bind(role_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Find a role by ID and populate its permissions.
pub async fn find_with_permissions(pool: &Db, role_id: Uuid) -> AppResult<Option<Role>> {
    let mut role = match sqlx::query_as::<_, Role>(
        "SELECT role_id, name, description, created_at FROM roles WHERE role_id = $1",
    )
    .bind(role_id)
    .fetch_optional(pool)
    .await?
    {
        Some(r) => r,
        None => return Ok(None),
    };

    let perms =
        sqlx::query("SELECT resource, access_level FROM role_permissions WHERE role_id = $1")
            .bind(role.role_id)
            .fetch_all(pool)
            .await
            .unwrap_or_default();

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

    Ok(Some(role))
}

/// Find role_id by name.
pub async fn find_id_by_name(pool: &Db, name: &str) -> AppResult<Option<Uuid>> {
    use sqlx::Row;
    let row = sqlx::query("SELECT role_id FROM roles WHERE name = $1")
        .bind(name)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|r| r.get("role_id")))
}
