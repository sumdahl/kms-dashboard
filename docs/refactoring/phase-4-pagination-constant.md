# Phase 4 — Extract Pagination Constant

## Goal
Eliminate hardcoded `8i64` page size in `handlers/admin/roles.rs` by extracting it to a named constant in a dedicated utilities module.

## Changes

### New: `src/utils/mod.rs`
Declares the `pagination` sub-module.

```rust
pub mod pagination;
```

### New: `src/utils/pagination.rs`
Single named constant for the default page size.

```rust
/// Default number of items per page for paginated list queries.
pub const PAGE_SIZE: i64 = 8;
```

### Modified: `src/main.rs`
Added `mod utils;` declaration.

### Modified: `src/handlers/admin/roles.rs`
`load_roles_list_data`: replaced `let size = 8i64;` with `let size = crate::utils::pagination::PAGE_SIZE;`.

## Invariants Preserved
- Pagination behavior identical (still 8 items per page)
- `cargo build` passes with zero new warnings
