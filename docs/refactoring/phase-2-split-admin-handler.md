# Phase 2 — Split handlers/admin.rs into Sub-modules

## Goal
Break the 1,442-line `handlers/admin.rs` monolith into domain-focused files. Each file has a single concern. The original import paths remain valid via `pub use` re-exports.

## Structure

In Rust, when `admin.rs` declares `pub mod foo;`, the compiler looks for
`src/handlers/admin/foo.rs`. No directory rename needed.

```
src/handlers/admin.rs          ← thin facade (re-exports only)
src/handlers/admin/
├── views.rs                   ← shared view types + utilities
├── roles.rs                   ← role CRUD handlers + role display types
├── users.rs                   ← user management handlers
└── assignments.rs             ← role assignment handler
```

## Files Created

### `src/handlers/admin/views.rs`
Shared primitives used across all admin handlers.

| Symbol | Type | Description |
|--------|------|-------------|
| `QuickPermissionRow` | struct | Permission row selection state |
| `QuickCreateRoleView` | Askama template | Quick-create role page |
| `CreateRoleWizardView` | Askama template | Role creation wizard |
| `quick_create_resource_list()` | fn | Static list of resource options |
| `quick_create_access_level_list()` | fn | Static list of access level options |
| `quick_create_default_permission_rows()` | fn | Default single empty row |
| `quick_permission_rows_from_form()` | fn | Hydrate rows from form input |
| `is_htmx()` | fn | Check HX-Request header |
| `is_quick_create_htmx()` | fn | Detect quick-create HTMX context |
| `is_wizard_htmx()` | fn | Detect wizard HTMX context |
| `hx_redirect_response()` | fn | Build HX-Redirect 204 response |
| `quick_create_shell()` | fn | Build QuickCreateRoleView from form state |
| `wizard_shell()` | fn | Build CreateRoleWizardView from form state |
| `empty_quick_create_form()` | fn | Default empty form for quick-create |
| `empty_wizard_form()` | fn | Default empty form for wizard |
| `query_param_encode()` | fn | URL-encode a query parameter value |
| `append_query_param()` | fn | Append key=value to a URL |

Tests for `append_query_param` moved here from the old `admin_tests` module.

### `src/handlers/admin/roles.rs`
Role-related display types and HTTP handlers.

| Symbol | Description |
|--------|-------------|
| `PermissionRowTemplate` | Template for single permission row fragment |
| `permission_row()` | Handler: GET /admin/roles/permission-row |
| `ListRolesQuery` | Query params for paginated role list |
| `RoleDisplay` | Computed display struct (colors, initials, badges) |
| `role_to_display()` | Convert DB `Role` → `RoleDisplay` |
| `RolesListData` | Paginated list with display fields |
| `load_roles_list_data()` | Build `RolesListData` from repo |
| `RolesListFragment` | HTMX fragment template |
| `roles_list_htmx()` | Handler: GET /admin/roles/list |
| `delete_role_htmx()` | Handler: POST /admin/roles/:id/htmx-delete |
| `delete_role_submit()` | Handler: POST /admin/roles/:id/delete |
| `create_role_form()` | Handler: POST /admin/roles/create |

### `src/handlers/admin/users.rs`
User management handlers.

| Symbol | Description |
|--------|-------------|
| `DisableUserRequest` | Form payload for disable action |
| `users_htmx_html()` | Build users fragment HTML (used in disable/enable responses) |
| `disable_user()` | Handler: POST /admin/users/disable/:id |
| `enable_user()` | Handler: POST /admin/users/enable/:id |

### `src/handlers/admin/assignments.rs`
Role assignment handler.

| Symbol | Description |
|--------|-------------|
| `AssignRoleRequest` | Parsed assignment intent |
| `AssignRoleHtmlForm` | Raw form fields from HTML form |
| `assign_role()` | Handler: POST /admin/assign |

## `handlers/admin.rs` After Phase 2
Reduced to 30 lines — module declarations + `pub use` re-exports only.
All imports from `crate::handlers::admin::*` continue to work unchanged.

## Invariants Preserved
- All HTTP routes unchanged
- All template structs unchanged  
- All SQL queries unchanged (delegated to repositories from Phase 1)
- `cargo build` passes with zero new warnings
- Both test modules pass: `views_tests::*` (moved from `admin_tests`)
