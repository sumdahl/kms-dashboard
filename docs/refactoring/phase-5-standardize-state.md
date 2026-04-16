# Phase 5 — Standardize Handler State to AppState

## Goal
All route handlers now accept `State<AppState>` instead of the mixed `State<Db>` / `State<AppState>` pattern. The `FromRef<AppState> for Db` impl is retained for backward compat with any future code that still needs it, but no production handler uses it directly.

## Why
- Handlers that need additional AppState fields (resend client, base URL) can access them without signature changes.
- Consistent extractor type across the entire codebase — easier to grep, easier to add state fields.
- `State<Db>` worked via Axum's `FromRef` mechanism, but obscured what state was actually in scope.

## Pattern Applied
Each handler changed from:
```rust
State(pool): State<Db>
```
to:
```rust
State(state): State<AppState>
```
with `let pool = state.db;` added at the top of the function body (PgPool is Arc-based, move is cheap).

Helper functions that take `pool: &Db` as a plain parameter (not State) are unchanged — their callers pass `&pool` after the extraction.

## Files Changed

| File | Handlers Updated |
|------|-----------------|
| `src/handlers/admin/assignments.rs` | `assign_role` |
| `src/handlers/admin/roles.rs` | `roles_list_htmx`, `delete_role_htmx`, `delete_role_submit`, `create_role_form` |
| `src/handlers/admin/users.rs` | `disable_user`, `enable_user` |
| `src/handlers/auth.rs` | `login`, `signup`, `logout` |
| `src/handlers/dashboard.rs` | `my_roles` |
| `src/handlers/api.rs` | `global_search` |
| `src/routes/dashboard/home.rs` | `home` |
| `src/routes/dashboard/roles.rs` | `roles_page`, `role_detail_page` |
| `src/routes/dashboard/users.rs` | `users_page` |
| `src/routes/dashboard/assign.rs` | `assign_page` |

## Unchanged
- `src/middleware/auth.rs` — already used `State<AppState>`
- `src/handlers/password_reset.rs` — already used `State<AppState>`
- Helper functions (`load_my_roles`, `users_htmx_html`, `load_roles_list_data`, repo fns) — take `&Db` as plain params, not State

## Invariants Preserved
- All HTTP routes identical
- All SQL queries identical
- `cargo build` passes with zero new warnings (15 pre-existing remain)
