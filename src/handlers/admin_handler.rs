use axum::{extract::State, Json};
use crate::auth::Claims;
use crate::db::AppDb;
use crate::error::AppError;
use crate::models::*;

pub async fn get_settings(
    State(db): State<AppDb>,
) -> Result<Json<SystemSettings>, AppError> {
    let mut settings: Vec<SystemSettings> = db.query("SELECT * FROM settings").await?.take(0)?;
    if settings.is_empty() {
        let default_settings = SystemSettings {
            id: None,
            upper_lower_percentage: 27.0,
            thresholds: DiscriminationThresholds::default(),
            schools: vec![
                "School of Science & Technology".to_string(),
                "School of Business & Economics".to_string(),
                "Faculty of Education".to_string(),
            ],
            assessment_types: vec![
                "Test".to_string(),
                "UE".to_string(),
                "Assignment".to_string(),
                "Examination".to_string(),
            ],
            updated_at: chrono::Utc::now().to_rfc3339(),
        };
        let created: Option<SystemSettings> = db
            .create(("settings", "default"))
            .content(default_settings.clone())
            .await?;
        return Ok(Json(created.unwrap_or(default_settings)));
    }

    Ok(Json(settings.remove(0)))
}

pub async fn update_settings(
    State(db): State<AppDb>,
    claims: Claims,
    Json(req): Json<UpdateSettingsReq>,
) -> Result<Json<SystemSettings>, AppError> {
    if claims.role != UserRole::Superadmin && claims.role != UserRole::Admin {
        return Err(AppError::Forbidden("Only Admins can update system settings".to_string()));
    }

    let mut settings: Vec<SystemSettings> = db.query("SELECT * FROM settings").await?.take(0)?;
    let mut current = if settings.is_empty() {
        SystemSettings {
            id: None,
            upper_lower_percentage: 27.0,
            thresholds: DiscriminationThresholds::default(),
            schools: vec![],
            assessment_types: vec![],
            updated_at: chrono::Utc::now().to_rfc3339(),
        }
    } else {
        settings.remove(0)
    };

    if let Some(pct) = req.upper_lower_percentage {
        if pct <= 0.0 || pct > 50.0 {
            return Err(AppError::BadRequest("Percentage must be between 1% and 50%".to_string()));
        }
        current.upper_lower_percentage = pct;
    }

    if let Some(thresh) = req.thresholds {
        current.thresholds = thresh;
    }

    if let Some(schools) = req.schools {
        current.schools = schools;
    }

    if let Some(types) = req.assessment_types {
        current.assessment_types = types;
    }

    current.updated_at = chrono::Utc::now().to_rfc3339();

    let updated: Option<SystemSettings> = db
        .update(("settings", "default"))
        .content(current)
        .await?;

    let res = updated.ok_or_else(|| AppError::Internal("Failed to update settings".to_string()))?;
    Ok(Json(res))
}
