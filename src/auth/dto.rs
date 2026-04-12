//! Auth + admin form DTOs — `validator` rules delegate to `field_validation` for stable UX copy.

use crate::auth::field_validation::{
    email_field_error, full_name_field_error, password_field_error, role_description_field_error,
    role_name_field_error,
};
use crate::models::types::{AccessLevel, Resource, RolePermissionInput};
use serde::Deserialize;
use std::borrow::Cow;
use std::collections::HashSet;
use validator::{Validate, ValidationError, ValidationErrors};

const MAX_ROLE_PERMISSIONS: usize = 32;

fn ve(msg: impl Into<String>) -> ValidationError {
    let mut e = ValidationError::new("custom");
    e.message = Some(Cow::Owned(msg.into()));
    e
}

fn validate_signup_full_name(name: &String) -> Result<(), ValidationError> {
    match full_name_field_error(name) {
        None => Ok(()),
        Some(msg) => Err(ve(msg)),
    }
}

fn validate_signup_email(email: &String) -> Result<(), ValidationError> {
    match email_field_error(email) {
        None => Ok(()),
        Some(msg) => Err(ve(msg)),
    }
}

fn validate_signup_password(password: &String) -> Result<(), ValidationError> {
    match password_field_error(password) {
        None => Ok(()),
        Some(msg) => Err(ve(msg)),
    }
}

fn validate_login_email(email: &String) -> Result<(), ValidationError> {
    match email_field_error(email) {
        None => Ok(()),
        Some(msg) => Err(ve(msg)),
    }
}

fn validate_login_password(password: &String) -> Result<(), ValidationError> {
    match password_field_error(password) {
        None => Ok(()),
        Some(msg) => Err(ve(msg)),
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(custom(function = "validate_login_email"))]
    pub email: String,
    #[validate(custom(function = "validate_login_password"))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct SignupRequest {
    #[validate(custom(function = "validate_signup_full_name"))]
    pub full_name: String,
    #[validate(custom(function = "validate_signup_email"))]
    pub email: String,
    #[validate(custom(function = "validate_signup_password"))]
    pub password: String,
}

/// First human message for a field from `validator` output.
pub fn first_field_message(errs: &ValidationErrors, field: &str) -> Option<String> {
    errs.field_errors()
        .get(field)
        .and_then(|v| v.first())
        .and_then(|e| e.message.as_ref().map(|m| m.to_string()))
}

// ── Create role (HTML form only) ─────────────────────────────────────────

/// HTML `POST /admin/roles/create` body (same fields as wizard + quick form).
#[derive(Debug, Deserialize)]
pub struct CreateRoleFormRequest {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub resource: Vec<String>,
    #[serde(default)]
    pub access: Vec<String>,
    #[serde(default)]
    pub redirect: Option<String>,
    #[serde(default)]
    pub error_redirect: Option<String>,
}

/// Parse + validate; returns typed permissions for DB insert.
pub fn validate_and_parse_create_role_form(
    form: &CreateRoleFormRequest,
) -> Result<Vec<RolePermissionInput>, ValidationErrors> {
    let perms = match parse_create_role_permissions(&form.resource, &form.access) {
        Ok(p) => p,
        Err(msg) => {
            let mut e = ValidationErrors::new();
            e.add("resource", ve(msg));
            return Err(e);
        }
    };
    validate_create_role_body(&form.name, &form.description, &perms)?;
    Ok(perms)
}

/// Parse `resource[]` / `access[]` form pairs into typed permissions.
fn parse_create_role_permissions(
    resource: &[String],
    access: &[String],
) -> Result<Vec<RolePermissionInput>, String> {
    if resource.len() != access.len() {
        return Err(
            "Permission rows are misaligned (resource/access count mismatch). Refresh and try again."
                .into(),
        );
    }
    let mut permissions = Vec::new();
    for i in 0..resource.len() {
        let r = resource[i].trim();
        let a = access[i].trim();
        match (r.is_empty(), a.is_empty()) {
            (true, true) => {}
            (false, false) => {
                let res: Resource = r
                    .parse()
                    .map_err(|_| format!("Invalid resource: {}", r))?;
                let acc: AccessLevel = a
                    .parse()
                    .map_err(|_| format!("Invalid access: {}", a))?;
                permissions.push(RolePermissionInput {
                    resource: res,
                    access: acc,
                });
            }
            _ => {
                return Err(
                    "Each permission row must have both resource and access.".into(),
                );
            }
        }
    }
    Ok(permissions)
}

fn validate_role_permission_slice(perms: &[RolePermissionInput]) -> Result<(), String> {
    if perms.is_empty() {
        return Err("At least one permission is required.".into());
    }
    if perms.len() > MAX_ROLE_PERMISSIONS {
        return Err(format!(
            "Too many permission rows (max {}).",
            MAX_ROLE_PERMISSIONS
        ));
    }
    let mut seen = HashSet::<String>::new();
    for p in perms {
        let key = format!("{}:{}", p.resource, p.access);
        if !seen.insert(key) {
            return Err("Duplicate permission rows.".into());
        }
    }
    Ok(())
}

/// Shared identity + permission rules (HTML form only).
fn validate_create_role_body(
    name: &str,
    description: &str,
    parsed_permissions: &[RolePermissionInput],
) -> Result<(), ValidationErrors> {
    let mut errs = ValidationErrors::new();
    if let Some(m) = role_name_field_error(name) {
        errs.add("name", ve(m));
    }
    if let Some(m) = role_description_field_error(description) {
        errs.add("description", ve(m));
    }
    if let Err(msg) = validate_role_permission_slice(parsed_permissions) {
        errs.add("resource", ve(msg));
    }
    if errs.is_empty() {
        Ok(())
    } else {
        Err(errs)
    }
}
