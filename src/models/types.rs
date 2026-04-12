use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AccessLevel {
    Read = 1,
    Write = 2,
    Admin = 3,
}

impl fmt::Display for AccessLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AccessLevel::Read => write!(f, "read"),
            AccessLevel::Write => write!(f, "write"),
            AccessLevel::Admin => write!(f, "admin"),
        }
    }
}

impl FromStr for AccessLevel {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "read" => Ok(AccessLevel::Read),
            "write" => Ok(AccessLevel::Write),
            "admin" => Ok(AccessLevel::Admin),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Resource {
    Orders,
    Customers,
    Reports,
    Inventory,
    AdminPanel,
}

impl fmt::Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Resource::Orders => write!(f, "orders"),
            Resource::Customers => write!(f, "customers"),
            Resource::Reports => write!(f, "reports"),
            Resource::Inventory => write!(f, "inventory"),
            Resource::AdminPanel => write!(f, "admin_panel"),
        }
    }
}

impl FromStr for Resource {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "orders" => Ok(Resource::Orders),
            "customers" => Ok(Resource::Customers),
            "reports" => Ok(Resource::Reports),
            "inventory" => Ok(Resource::Inventory),
            "admin_panel" => Ok(Resource::AdminPanel),
            _ => Err(()),
        }
    }
}

/// One resource/access pair for role create (HTML form or JSON API).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RolePermissionInput {
    pub resource: Resource,
    pub access: AccessLevel,
}

