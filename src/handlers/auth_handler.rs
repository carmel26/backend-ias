use axum::{extract::State, Json};
use crate::auth::{create_jwt, hash_password, verify_password, Claims};
use crate::db::AppDb;
use crate::error::AppError;
use crate::models::*;

pub async fn register(
    State(db): State<AppDb>,
    Json(req): Json<RegisterReq>,
) -> Result<Json<AuthResponse>, AppError> {
    if req.email.trim().is_empty() || req.password.trim().is_empty() {
        return Err(AppError::BadRequest("Email and password are required".to_string()));
    }

    // Check if user email already exists
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
        role: UserRole::Lecturer,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    let created: Option<User> = db
        .create(("user", user_key.as_str()))
        .content(user.clone())
        .await?;

    let user_obj = created.unwrap_or(user);
    let user_dto: UserDto = user_obj.into();
    let token = create_jwt(&user_dto.id, &user_dto.email, user_dto.role.clone())?;

    Ok(Json(AuthResponse {
        token,
        user: user_dto,
    }))
}

pub async fn login(
    State(db): State<AppDb>,
    Json(req): Json<LoginReq>,
) -> Result<Json<AuthResponse>, AppError> {
    let users: Vec<User> = db
        .query("SELECT * FROM user WHERE email = $email")
        .bind(("email", req.email.to_lowercase()))
        .await?
        .take(0)?;

    let user = users
        .into_iter()
        .next()
        .ok_or_else(|| AppError::Unauthorized("Invalid email or password".to_string()))?;

    if !verify_password(&req.password, &user.password_hash)? {
        return Err(AppError::Unauthorized("Invalid email or password".to_string()));
    }

    let user_dto: UserDto = user.into();
    let token = create_jwt(&user_dto.id, &user_dto.email, user_dto.role.clone())?;

    Ok(Json(AuthResponse {
        token,
        user: user_dto,
    }))
}

pub async fn get_me(
    State(db): State<AppDb>,
    claims: Claims,
) -> Result<Json<UserDto>, AppError> {
    let users: Vec<User> = db
        .query("SELECT * FROM user WHERE email = $email")
        .bind(("email", claims.email))
        .await?
        .take(0)?;

    let user = users
        .into_iter()
        .next()
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(Json(user.into()))
}
