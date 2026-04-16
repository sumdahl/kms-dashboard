# Phase 3 — Split routes/dashboard.rs into Sub-modules

## Goal
Break the 840-line `routes/dashboard.rs` monolith into domain-focused files. Each file has a single concern. The router and HTMX helper remain in the facade.

## Structure

```
src/routes/dashboard.rs        ← thin facade (sub-module decls + router + is_htmx_partial)
src/routes/dashboard/
├── home.rs                    ← home page handler + templates
├── users.rs                   ← users list handler + templates + avatar helpers
├── roles.rs                   ← roles list, role detail, wizard/quick-create handlers + templates
└── assign.rs                  ← assign page handler + templates + tests
```

## Files Created

### `src/routes/dashboard/home.rs`
| Symbol | Type | Description |
|--------|------|-------------|
| `HomeParams` | struct | Query params (skip_onboarding) |
| `HomeTemplate` | Askama template | Full-page home |
| `HomePartialTemplate` | Askama template | HTMX partial home |
| `home()` | async fn | GET / handler |

### `src/routes/dashboard/users.rs`
| Symbol | Type | Description |
|--------|------|-------------|
| `UsersListQuery` | struct | Query params (flash_kind, flash_msg, error) |
| `UsersTemplate` | Askama template | Full-page users list |
| `UsersPartialTemplate` | Askama template | HTMX partial users list |
| `user_initials()` | fn | Avatar initials from full name |
| `user_avatar_style()` | fn | HSL avatar background from email hash |
| `users_page()` | async fn | GET /users handler |

### `src/routes/dashboard/roles.rs`
| Symbol | Type | Description |
|--------|------|-------------|
| `RolesPageQuery` | struct | Query params (page, search, notice, error, skip_onboarding) |
| `QuickCreateQuery` | struct | Query params (error) |
| `WizardPageQuery` | struct | Query params (error) |
| `RolesTemplate` | Askama template | Full-page roles list |
| `RolesPartialTemplate` | Askama template | HTMX partial roles list |
| `RoleDetailTemplate` | Askama template | Full-page role detail |
| `RoleDetailPartialTemplate` | Askama template | HTMX partial role detail |
| `roles_page()` | async fn | GET /roles handler |
| `create_role_wizard_page()` | async fn | GET /roles/new handler |
| `quick_create_role_page()` | async fn | GET /roles/quick handler |
| `role_detail_page()` | async fn | GET /roles/:role_id handler |

### `src/routes/dashboard/assign.rs`
| Symbol | Type | Description |
|--------|------|-------------|
| `AssignPageQuery` | struct | Query params (skip_onboarding, error, notice, role) |
| `AssignTemplate` | Askama template | Full-page assign |
| `AssignPartialTemplate` | Askama template | HTMX partial assign |
| `OnboardingTemplate` | Askama template | Onboarding full page |
| `OnboardingPartialTemplate` | Askama template | Onboarding HTMX partial |
| `assign_message_parts()` | fn | Map notice/error query params to (kind, message) |
| `assign_page()` | async fn | GET /assign handler |
| `assign_tests` | mod | Unit tests for `assign_message_parts` (moved from `dashboard_tests`) |

## `routes/dashboard.rs` After Phase 3
Reduced to 28 lines — sub-module declarations, `is_htmx_partial` helper, and `router()` only.

## Invariants Preserved
- All HTTP routes unchanged
- All template structs unchanged
- All SQL queries unchanged (delegated to repositories)
- `cargo build` passes with zero new warnings (15 pre-existing warnings remain)
- Both test cases in `assign_tests` pass (moved from `dashboard_tests`)
