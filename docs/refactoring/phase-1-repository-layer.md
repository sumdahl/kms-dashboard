# Phase 1 — Repository Layer

## Goal
Extract all raw SQL queries from handlers and routes into a dedicated `src/repositories/` module. Establish a clean data-access layer with no view/handler logic.

## Files Created

### `src/repositories/mod.rs`
Module root. Declares three sub-modules.

### `src/repositories/roles.rs`
All role-related DB queries.

| Symbol | Description |
|--------|-------------|
| `PaginatedRoles` | Struct: paginated query result |
| `RolesSummary` | Struct: aggregate stats (counts, permissions) |
| `CreateRoleRequest` | Struct: input for role creation |
| `load_paginated_roles()` | Paginated role list with optional search + permissions populated |
| `fetch_all_role_names()` | All role names for dropdowns |
| `load_roles_summary()` | Aggregate stats across roles/permissions |
| `persist_new_role()` | Create role + permissions in single transaction |
| `delete_by_id()` | Delete role, returns `bool` (was deleted) |
| `find_with_permissions()` | Fetch one role with permissions loaded |
| `find_id_by_name()` | Look up role UUID by name |

### `src/repositories/users.rs`
All user-related DB queries.

| Symbol | Description |
|--------|-------------|
| `UserSummary` | Struct: non-admin user row for UI display |
| `fetch_user_summaries()` | All non-admin users ordered by created_at |
| `count_admins()` | Count of admin users (was duplicated in 2 places) |
| `find_id_by_email()` | Look up user UUID by email |

### `src/repositories/assignments.rs`
All role-assignment-related DB queries.

| Symbol | Description |
|--------|-------------|
| `AssignmentWithUser` | Struct: assignment row joined with user email |
| `upsert_assignment()` | Insert or update assignment (ON CONFLICT) |
| `find_by_role_with_users()` | All assignments for a role with user emails |

## Files Modified

### `src/main.rs`
Added `mod repositories;` declaration.

### `src/handlers/admin.rs` (~1,442 → ~870 LOC)
- Removed: `PaginatedRoles`, `RolesSummary`, `UserSummary` struct definitions
- Removed: `load_paginated_roles`, `fetch_all_role_names`, `load_roles_summary`, `persist_new_role`, `fetch_user_summaries` function bodies
- Added: `pub use` re-exports for `fetch_all_role_names`, `load_roles_summary`, `RolesSummary`, `fetch_user_summaries`, `UserSummary` — backward-compatible, routes importing from `handlers::admin` require no changes
- Updated `users_htmx_html`: replaced inline `COUNT(*) admin` query → `repositories::users::count_admins()`
- Updated `delete_role_htmx`: replaced inline `DELETE FROM roles` → `repositories::roles::delete_by_id()`
- Updated `delete_role_submit`: same deletion replaced with repo call
- Updated `assign_role`: replaced 3 inline queries (user lookup, role lookup, upsert) → `repositories::users::find_id_by_email()`, `repositories::roles::find_id_by_name()`, `repositories::assignments::upsert_assignment()`
- Updated `create_role_form`: calls `repositories::roles::persist_new_role()` directly
- Updated `load_roles_list_data`: calls `repositories::roles::load_paginated_roles()` directly

### `src/routes/dashboard.rs`
- Removed unused imports: `AccessLevel`, `Resource`, `RolePermission`, `Role` (from models), `sqlx::Row`
- Added `use crate::repositories;`
- `users_page`: replaced duplicate admin count query → `repositories::users::count_admins()`
- `role_detail_page`: replaced 3 inline SQL blocks (find role, find permissions, find assignments) → `repositories::roles::find_with_permissions()` + `repositories::assignments::find_by_role_with_users()`

## Invariants Preserved
- All SQL queries are **byte-for-byte identical** to original — no logic changes
- All transactions (`persist_new_role`, `disable_user`, `enable_user`) preserved
- All HTTP responses unchanged
- All template rendering unchanged
- `cargo build` passes with zero new warnings

## Duplication Eliminated
The admin count query `SELECT COUNT(*)::bigint FROM users WHERE is_admin = TRUE` existed in:
- `handlers/admin.rs::users_htmx_html`
- `routes/dashboard.rs::users_page`

Both now delegate to `repositories::users::count_admins()`.
