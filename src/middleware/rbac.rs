use axum::{
    async_trait,
    extract::FromRequestParts,
    http::request::Parts,
};
use crate::db::Db;
use crate::error::{AppError, AppResult};
use crate::models::{Claims, ResolvedPermission};
use crate::models::types::{AccessLevel, Resource};
use uuid::Uuid;
use sqlx::Row;

pub struct Permissions(pub Vec<ResolvedPermission>);

#[async_trait]
impl<S> FromRequestParts<S> for Permissions
where
    Db: FromRef<S>, // Correct way to access state in 0.7
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // 1. Get identity
        let claims = Claims::from_request_parts(parts, state).await?;
        let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Unauthorized)?;

        // 2. Get DB from state
        let pool = Db::from_ref(state);

        // 3. Query Active Permissions
        let rows = sqlx::query(
            r#"
            SELECT 
                rp.resource, 
                rp.access_level, 
                ra.expires_at, 
                r.name as role_name
            FROM role_assignments ra
            JOIN roles r ON ra.role_id = r.role_id
            JOIN role_permissions rp ON r.role_id = rp.role_id
            WHERE ra.user_id = $1
            AND (ra.expires_at IS NULL OR ra.expires_at > NOW())
            "#
        )
        .bind(user_id)
        .fetch_all(&pool)
        .await?;

        let mut resolved: Vec<ResolvedPermission> = Vec::new();

        for row in rows {
            let res_str: String = row.get("resource");
            let acc_str: String = row.get("access_level");
            let role_name: String = row.get("role_name");
            
            let resource: Resource = res_str.parse().unwrap_or(Resource::Orders);
            let access: AccessLevel = acc_str.parse().unwrap_or(AccessLevel::Read);

            if let Some(existing) = resolved.iter_mut().find(|p| p.resource == resource) {
                if access > existing.access {
                    existing.access = access;
                }
                existing.granted_by_roles.push(role_name);
            } else {
                resolved.push(ResolvedPermission {
                    resource,
                    access,
                    expires_at: row.get("expires_at"),
                    granted_by_roles: vec![role_name],
                });
            }
        }

        Ok(Permissions(resolved))
    }
}

// We need FromRef for Axum to pull the pool from the state
use axum::extract::FromRef;

impl Permissions {
    pub fn require(&self, resource: Resource, level: AccessLevel) -> AppResult<()> {
        let has_it = self.0.iter().any(|p| {
            p.resource == resource && p.access >= level
        });

        if has_it {
            Ok(())
        } else {
            Err(AppError::InsufficientAccess)
        }
    }
}
