# The Model Layer (Data & Logic)

The **Model** layer defines how data is structured, stored, and manipulated. All models are defined within `src/models/`.

## 🏗️ Core Components

1.  **Rust Structs:**
    *   Models are standard Rust structs decorated with `#[derive(sqlx::FromRow)]`.
    *   Example: `User` struct in `src/models/user.rs`.

2.  **SQLx Integration:**
    *   Database queries map directly to these structs using the `fetch_one`, `fetch_all`, or `fetch_optional` methods.
    *   Asynchronous DB interactions ensure the server remains highly concurrent.

3.  **Type-Safe Enums:**
    *   Constants for system resources and permissions are implemented as Rust enums (e.g., `Resource::Inventory`, `AccessLevel::Read`).
    *   These enums implement `Display` and `FromStr` for easy database and template integration.

## ⚙️ Data Logic

Business logic (calculations, validations, constructors) is kept within the `impl` blocks of the model structs. This keeps data and its related behaviors localized.

**Example Pattern:**
```rust
impl User {
    pub fn new(email: &str, full_name: &str, password_hash: &str) -> Self {
        Self {
            user_id: Uuid::new_v4(),
            email: email.to_string(),
            full_name: full_name.to_string(),
            password_hash: password_hash.to_string(),
            is_admin: false,
            is_active: true,
            created_at: Utc::now(),
            // ... other fields
        }
    }
}
```

## 🔐 Authentication & Claims
JWT claims and session versioning (for secure logout/token revocation) are also modeled in this layer (see `src/models/auth.rs`).
