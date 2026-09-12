use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header, request::Parts},
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use crate::error::AppError;
use crate::models::UserRole;

const JWT_SECRET: &[u8] = b"super_secret_academic_item_analysis_jwt_key_2026";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub role: UserRole,
    #[serde(default)]
    pub school: String,
    pub exp: usize,
}

pub fn create_jwt(
    user_id: &str,
    email: &str,
    role: UserRole,
    school: &str,
) -> Result<String, AppError> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(7))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        role,
        school: school.to_string(),
        exp: expiration,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET),
    )?;

    Ok(token)
}

pub fn verify_jwt(token: &str) -> Result<Claims, AppError> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}

pub fn hash_password(password: &str) -> Result<String, AppError> {
    let hashed = bcrypt::hash(password, bcrypt::DEFAULT_COST)?;
    Ok(hashed)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let valid = bcrypt::verify(password, hash)?;
    Ok(valid)
}

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|val| val.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".to_string()))?;

        if !auth_header.starts_with("Bearer ") {
            return Err(AppError::Unauthorized(
                "Invalid Authorization header format".to_string(),
            ));
        }

        let token = &auth_header[7..];
        let claims = verify_jwt(token)?;
        Ok(claims)
    }
}
