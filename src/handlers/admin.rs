/// Admin handler module.
///
/// Sub-modules are declared here; Rust resolves them from `src/handlers/admin/`.
/// Re-exports maintain backward compatibility for all existing route imports.
pub mod assignments;
pub mod roles;
pub mod users;
pub mod views;

// ── Re-exports from repositories (routes/dashboard.rs imports these) ──────────
pub use crate::repositories::roles::{fetch_all_role_names, load_roles_summary, RolesSummary};
pub use crate::repositories::users::{fetch_user_summaries, UserSummary};

// ── Re-exports from sub-modules (routes/admin.rs + routes/dashboard.rs) ───────
pub use assignments::assign_role;
pub use roles::{
    create_role_form, delete_role_htmx, delete_role_submit, load_roles_list_data, permission_row,
    roles_list_htmx, RoleDisplay, RolesListData,
};
pub use users::{disable_user, enable_user};
pub use views::{
    quick_create_access_level_list, quick_create_default_permission_rows,
    quick_create_resource_list, CreateRoleWizardView, QuickCreateRoleView,
};
