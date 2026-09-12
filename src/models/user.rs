use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum UserRole {
    Superadmin,
    Admin,
    Lecturer,
}

impl Default for UserRole {
    fn default() -> Self {
        UserRole::Lecturer
    }
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::Superadmin => write!(f, "Superadmin"),
            UserRole::Admin => write!(f, "Admin"),
            UserRole::Lecturer => write!(f, "Lecturer"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "crate::models::serialize_id")]
    pub id: Option<Thing>,
    pub first_name: String,
    pub middle_name: Option<String>,
    pub surname: String,
    pub school: String,
    pub profession: String,
    pub specialization: String,
    pub email: String,
    pub password_hash: String,
    pub role: UserRole,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDto {
    pub id: String,
    pub first_name: String,
    pub middle_name: Option<String>,
    pub surname: String,
    pub school: String,
    pub profession: String,
    pub specialization: String,
    pub email: String,
    pub role: UserRole,
    pub created_at: String,
}

impl From<User> for UserDto {
    fn from(u: User) -> Self {
        let id_str = u.id.as_ref().map(|t| t.id.to_string()).unwrap_or_default();
        Self {
            id: id_str,
            first_name: u.first_name,
            middle_name: u.middle_name,
            surname: u.surname,
            school: u.school,
            profession: u.profession,
            specialization: u.specialization,
            email: u.email,
            role: u.role,
            created_at: u.created_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RegisterReq {
    pub first_name: String,
    pub middle_name: Option<String>,
    pub surname: String,
    pub school: String,
    pub profession: String,
    pub specialization: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginReq {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserDto,
}

#[derive(Debug, Deserialize)]
pub struct CreateAdminReq {
    pub first_name: String,
    pub middle_name: Option<String>,
    pub surname: String,
    pub school: String,
    pub profession: String,
    pub specialization: String,
    pub email: String,
    pub password: String,
    pub role: UserRole,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRoleReq {
    pub role: UserRole,
}
