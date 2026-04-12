//! Field-level validation — messages shared by auth handlers, Askama forms, and admin role forms.

const MAX_ROLE_NAME_CHARS: usize = 100;
const MAX_ROLE_DESCRIPTION_CHARS: usize = 8000;

/// `None` = valid. Matches `roles.name` (`VARCHAR(100)`).
pub fn role_name_field_error(raw: &str) -> Option<String> {
    let t = raw.trim();
    if t.is_empty() {
        return Some("Role name is required.".into());
    }
    let n = t.chars().count();
    if n < 2 {
        return Some("Role name is too short (at least 2 characters).".into());
    }
    if n > MAX_ROLE_NAME_CHARS {
        return Some(format!(
            "Role name is too long (max {MAX_ROLE_NAME_CHARS} characters)."
        ));
    }
    None
}

/// `None` = valid.
pub fn role_description_field_error(raw: &str) -> Option<String> {
    let t = raw.trim();
    if t.is_empty() {
        return Some("Description is required.".into());
    }
    let n = t.chars().count();
    if n < 3 {
        return Some("Description is too short (at least 3 characters).".into());
    }
    if n > MAX_ROLE_DESCRIPTION_CHARS {
        return Some(format!(
            "Description is too long (max {MAX_ROLE_DESCRIPTION_CHARS} characters)."
        ));
    }
    None
}

/// `None` = valid.
pub fn full_name_field_error(raw: &str) -> Option<String> {
    let t = raw.trim();
    if t.is_empty() {
        return Some("Full name is required.".into());
    }
    if t.chars().count() < 2 {
        return Some("Full name is too short.".into());
    }
    None
}

/// `None` = valid.
pub fn email_field_error(raw: &str) -> Option<String> {
    let t = raw.trim();
    if t.is_empty() {
        return Some("Email is required.".into());
    }
    if !t.contains('@') {
        return Some("Email must contain @.".into());
    }
    if !email_format_ok(t) {
        return Some("Enter a valid email address.".into());
    }
    None
}

fn email_format_ok(t: &str) -> bool {
    let at_count = t.chars().filter(|&c| c == '@').count();
    if at_count != 1 {
        return false;
    }
    let Some((local, domain)) = t.split_once('@') else {
        return false;
    };
    if local.is_empty() || domain.is_empty() {
        return false;
    }
    if local.chars().any(|c| c == '@' || c.is_whitespace()) {
        return false;
    }
    if domain.chars().any(|c| c == '@' || c.is_whitespace()) {
        return false;
    }
    let Some((_, tld)) = domain.rsplit_once('.') else {
        return false;
    };
    !tld.is_empty() && domain.contains('.')
}

/// `None` = valid.
pub fn password_field_error(raw: &str) -> Option<String> {
    if raw.is_empty() {
        return Some("Password is required.".into());
    }
    let n = raw.chars().count();
    if n < 6 {
        return Some(format!("Password must be at least 6 characters ({n}/6)."));
    }
    None
}

/// Reset form: confirm matches new password. `None` = valid.
pub fn reset_confirm_password_error(new_password: &str, confirm_password: &str) -> Option<String> {
    if confirm_password.is_empty() {
        return Some("Please confirm your password.".into());
    }
    if new_password != confirm_password {
        return Some("Passwords do not match.".into());
    }
    None
}
