use axum::{
    extract::{Path, State},
    Json,
};
use crate::auth::{hash_password, Claims};
use crate::db::AppDb;
use crate::error::AppError;
use crate::models::*;

pub async fn list_users(
    State(db): State<AppDb>,
    claims: Claims,
) -> Result<Json<Vec<UserDto>>, AppError> {
    if claims.role != UserRole::Superadmin && claims.role != UserRole::Admin {
        return Err(AppError::Forbidden("Only Admins and Superadmins can list users".to_string()));
    }

    let users: Vec<User> = if claims.role == UserRole::Superadmin {
        db.query("SELECT * FROM user").await?.take(0)?
    } else {
        db.query("SELECT * FROM user WHERE school = $school AND role = 'Lecturer'")
            .bind(("school", claims.school))
            .await?
            .take(0)?
    };
    let user_dtos: Vec<UserDto> = users.into_iter().map(UserDto::from).collect();

    Ok(Json(user_dtos))
}

pub async fn create_admin_user(
    State(db): State<AppDb>,
    claims: Claims,
    Json(mut req): Json<CreateAdminReq>,
) -> Result<Json<UserDto>, AppError> {
    if claims.role != UserRole::Superadmin && claims.role != UserRole::Admin {
        return Err(AppError::Forbidden("Permission denied".to_string()));
    }

    if req.role == UserRole::Superadmin && claims.role != UserRole::Superadmin {
        return Err(AppError::Forbidden("Only Superadmins can create Superadmin accounts".to_string()));
    }

    if claims.role == UserRole::Admin {
        if req.role != UserRole::Lecturer {
            return Err(AppError::Forbidden("Admins can only create Lecturer accounts".to_string()));
        }
        req.school = claims.school;
    }

    if req.password.chars().count() < 6 {
        return Err(AppError::BadRequest("Password must be at least 6 characters long".to_string()));
    }

    let existing: Vec<User> = db
        .query("SELECT * FROM user WHERE email = $email")
        .bind(("email", req.email.to_lowercase()))
        .await?
        .take(0)?;

    if !existing.is_empty() {
        return Err(AppError::BadRequest("User with this email already exists".to_string()));
    }

    let password_hash = hash_password(&req.password)?;
    let user_key = format!("user_{}", uuid::Uuid::new_v4().simple());

    let user = User {
        id: None,
        first_name: req.first_name,
        middle_name: req.middle_name,
        surname: req.surname,
        school: req.school,
        profession: req.profession,
        specialization: req.specialization,
        email: req.email.to_lowercase(),
        password_hash,
        role: req.role,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    let created: Option<User> = db
        .create(("user", user_key.as_str()))
        .content(user.clone())
        .await?;

    let user_obj = created.unwrap_or(user);
    Ok(Json(user_obj.into()))
}

pub async fn update_user_role(
    State(db): State<AppDb>,
    claims: Claims,
    Path(id): Path<String>,
    Json(req): Json<UpdateUserRoleReq>,
) -> Result<Json<UserDto>, AppError> {
    if claims.role != UserRole::Superadmin {
        return Err(AppError::Forbidden("Only Superadmins can modify user roles".to_string()));
    }

    let mut users: Vec<User> = db
        .query("SELECT * FROM user WHERE id = type::record('user', $id) OR email = $id")
        .bind(("id", id.clone()))
        .await?
        .take(0)?;

    let mut user = users
        .pop()
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    user.role = req.role;

    let updated: Option<User> = db
        .update(("user", id.as_str()))
        .content(user)
        .await?;

    let user_obj = updated.ok_or_else(|| AppError::Internal("Failed to update user".to_string()))?;
    Ok(Json(user_obj.into()))
}

pub async fn delete_user(
    State(db): State<AppDb>,
    claims: Claims,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    if claims.role != UserRole::Superadmin {
        return Err(AppError::Forbidden("Only Superadmins can delete users".to_string()));
    }

    let _: Option<User> = db.delete(("user", id.as_str())).await?;
    Ok(Json(serde_json::json!({ "message": "User deleted successfully" })))
}
