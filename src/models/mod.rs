pub mod types;
pub mod user;
pub mod role;
pub mod auth;

// Re-export only what is currently used to avoid warnings, 
// or keep them if you plan to use them soon.
pub use types::{AccessLevel, Resource};
pub use user::User;
pub use role::{Role, RolePermission, RoleAssignment};
pub use auth::{Claims, ResolvedPermission};
