use axum::{
    extract::{Path, State},
    Json,
};
use crate::auth::Claims;
use crate::db::AppDb;
use crate::error::AppError;
use crate::models::*;

pub async fn create_assessment(
    State(db): State<AppDb>,
    claims: Claims,
    Json(req): Json<CreateAssessmentReq>,
) -> Result<Json<Assessment>, AppError> {
    if req.num_students == 0 {
        return Err(AppError::BadRequest("Number of students must be greater than 0".to_string()));
    }

    // Get current system settings for upper/lower percentage
    let settings: Vec<SystemSettings> = db.query("SELECT * FROM settings").await?.take(0)?;
    let percentage = settings
        .first()
        .map(|s| s.upper_lower_percentage)
        .unwrap_or(27.0);

    let upper_group_size = (req.num_students as f64 * percentage / 100.0).round() as usize;
    let lower_group_size = upper_group_size;
    let middle_group_size = req.num_students.saturating_sub(upper_group_size + lower_group_size);

    // Fetch user details for lecturer name and school
    let users: Vec<User> = db
        .query("SELECT * FROM user WHERE id = type::record('user', $id) OR email = $email")
        .bind(("id", claims.sub.clone()))
        .bind(("email", claims.email.clone()))
        .await?
        .take(0)?;

    let user = users.first();
    let lecturer_name = user
        .map(|u| format!("{} {}", u.first_name, u.surname))
        .unwrap_or_else(|| claims.email.clone());
    let school = user
        .map(|u| u.school.clone())
        .unwrap_or_else(|| "General".to_string());

    let asm_id = format!("asm_{}", uuid::Uuid::new_v4().simple());

    let assessment = Assessment {
        id: None,
        user_id: claims.sub,
        lecturer_name,
        school,
        subject: req.subject,
        assessment_type: req.assessment_type,
        num_students: req.num_students,
        title: req.title,
        date: req.date,
        upper_percentage: percentage,
        upper_group_size,
        lower_group_size,
        middle_group_size,
        status: AssessmentStatus::Draft,
        admin_feedback: None,
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
    };

    let created: Option<Assessment> = db
        .create(("assessment", asm_id.as_str()))
        .content(assessment.clone())
        .await?;

    Ok(Json(created.unwrap_or(assessment)))
}

pub async fn list_assessments(
    State(db): State<AppDb>,
    claims: Claims,
) -> Result<Json<Vec<Assessment>>, AppError> {
    let assessments: Vec<Assessment> = if claims.role == UserRole::Superadmin {
        db.query("SELECT * FROM assessment").await?.take(0)?
    } else if claims.role == UserRole::Admin {
        db.query("SELECT * FROM assessment WHERE school = $school")
            .bind(("school", claims.school))
            .await?
            .take(0)?
    } else {
        db.query("SELECT * FROM assessment WHERE user_id = $uid")
            .bind(("uid", claims.sub))
            .await?
            .take(0)?
    };

    Ok(Json(assessments))
}

pub async fn get_assessment(
    State(db): State<AppDb>,
    Path(id): Path<String>,
) -> Result<Json<Assessment>, AppError> {
    let mut assessments: Vec<Assessment> = db
        .query("SELECT * FROM assessment WHERE id = type::record('assessment', $id) OR id = $id")
        .bind(("id", id.clone()))
        .await?
        .take(0)?;

    let asm = assessments
        .pop()
        .ok_or_else(|| AppError::NotFound("Assessment not found".to_string()))?;

    Ok(Json(asm))
}

pub async fn update_assessment(
    State(db): State<AppDb>,
    claims: Claims,
    Path(id): Path<String>,
    Json(req): Json<UpdateAssessmentReq>,
) -> Result<Json<Assessment>, AppError> {
    let mut assessments: Vec<Assessment> = db
        .query("SELECT * FROM assessment WHERE id = type::record('assessment', $id) OR id = $id")
        .bind(("id", id.clone()))
        .await?
        .take(0)?;

    let mut asm = assessments
        .pop()
        .ok_or_else(|| AppError::NotFound("Assessment not found".to_string()))?;

    if claims.role == UserRole::Lecturer && asm.user_id != claims.sub {
        return Err(AppError::Forbidden("You do not own this assessment".to_string()));
    }

    if asm.status != AssessmentStatus::Draft && claims.role == UserRole::Lecturer {
        return Err(AppError::BadRequest("Only Draft assessments can be modified by lecturer".to_string()));
    }

    let settings: Vec<SystemSettings> = db.query("SELECT * FROM settings").await?.take(0)?;
    let percentage = settings
        .first()
        .map(|s| s.upper_lower_percentage)
        .unwrap_or(27.0);

    let upper_group_size = (req.num_students as f64 * percentage / 100.0).round() as usize;
    let lower_group_size = upper_group_size;
    let middle_group_size = req.num_students.saturating_sub(upper_group_size + lower_group_size);

    asm.subject = req.subject;
    asm.assessment_type = req.assessment_type;
    asm.num_students = req.num_students;
    asm.title = req.title;
    asm.date = req.date;
    asm.upper_percentage = percentage;
    asm.upper_group_size = upper_group_size;
    asm.lower_group_size = lower_group_size;
    asm.middle_group_size = middle_group_size;
    asm.updated_at = chrono::Utc::now().to_rfc3339();

    let updated: Option<Assessment> = db
        .update(("assessment", id.as_str()))
        .content(asm)
        .await?;

    let res = updated.ok_or_else(|| AppError::Internal("Failed to update assessment".to_string()))?;
    Ok(Json(res))
}

pub async fn delete_assessment(
    State(db): State<AppDb>,
    claims: Claims,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut assessments: Vec<Assessment> = db
        .query("SELECT * FROM assessment WHERE id = type::record('assessment', $id) OR id = $id")
        .bind(("id", id.clone()))
        .await?
        .take(0)?;

    let asm = assessments
        .pop()
        .ok_or_else(|| AppError::NotFound("Assessment not found".to_string()))?;

    if claims.role == UserRole::Lecturer && asm.user_id != claims.sub {
        return Err(AppError::Forbidden("You do not own this assessment".to_string()));
    }

    let _: Option<Assessment> = db.delete(("assessment", id.as_str())).await?;

    // Also delete associated questions
    let _: Vec<QuestionItem> = db
        .query("DELETE FROM question WHERE assessment_id = $aid")
        .bind(("aid", id.clone()))
        .await?
        .take(0)?;

    Ok(Json(serde_json::json!({ "message": "Assessment deleted successfully" })))
}

pub async fn submit_assessment(
    State(db): State<AppDb>,
    claims: Claims,
    Path(id): Path<String>,
) -> Result<Json<Assessment>, AppError> {
    let mut assessments: Vec<Assessment> = db
        .query("SELECT * FROM assessment WHERE id = type::record('assessment', $id) OR id = $id")
        .bind(("id", id.clone()))
        .await?
        .take(0)?;

    let mut asm = assessments
        .pop()
        .ok_or_else(|| AppError::NotFound("Assessment not found".to_string()))?;

    if claims.role == UserRole::Lecturer && asm.user_id != claims.sub {
        return Err(AppError::Forbidden("You do not own this assessment".to_string()));
    }

    asm.status = AssessmentStatus::Submitted;
    asm.updated_at = chrono::Utc::now().to_rfc3339();

    let updated: Option<Assessment> = db
        .update(("assessment", id.as_str()))
        .content(asm)
        .await?;

    let res = updated.ok_or_else(|| AppError::Internal("Failed to submit assessment".to_string()))?;
    Ok(Json(res))
}

pub async fn verify_assessment(
    State(db): State<AppDb>,
    claims: Claims,
    Path(id): Path<String>,
    Json(req): Json<VerifyAssessmentReq>,
) -> Result<Json<Assessment>, AppError> {
    if claims.role != UserRole::Superadmin && claims.role != UserRole::Admin {
        return Err(AppError::Forbidden("Only Admins can verify assessments".to_string()));
    }

    let mut assessments: Vec<Assessment> = db
        .query("SELECT * FROM assessment WHERE id = type::record('assessment', $id) OR id = $id")
        .bind(("id", id.clone()))
        .await?
        .take(0)?;

    let mut asm = assessments
        .pop()
        .ok_or_else(|| AppError::NotFound("Assessment not found".to_string()))?;

    if claims.role == UserRole::Admin && asm.school != claims.school {
        return Err(AppError::Forbidden("You can only verify assessments from your school".to_string()));
    }

    asm.status = req.status;
    asm.admin_feedback = req.feedback;
    asm.updated_at = chrono::Utc::now().to_rfc3339();

    let updated: Option<Assessment> = db
        .update(("assessment", id.as_str()))
        .content(asm)
        .await?;

    let res = updated.ok_or_else(|| AppError::Internal("Failed to update verification state".to_string()))?;
    Ok(Json(res))
}
