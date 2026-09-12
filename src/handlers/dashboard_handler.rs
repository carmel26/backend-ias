use axum::{extract::State, Json};
use crate::auth::Claims;
use crate::db::AppDb;
use crate::error::AppError;
use crate::models::*;

pub async fn get_dashboard_stats(
    State(db): State<AppDb>,
    claims: Claims,
) -> Result<Json<DashboardStats>, AppError> {
    let assessments: Vec<Assessment> = if claims.role == UserRole::Superadmin {
        db.select("assessment").await?
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

    let total_assessments = assessments.len();
    let mut submitted = 0;
    let mut under_verification = 0;
    let mut verified = 0;
    let mut rejected = 0;
    let mut draft = 0;

    for a in &assessments {
        match a.status {
            AssessmentStatus::Draft => draft += 1,
            AssessmentStatus::Submitted => submitted += 1,
            AssessmentStatus::UnderVerification => under_verification += 1,
            AssessmentStatus::Verified => verified += 1,
            AssessmentStatus::Rejected => rejected += 1,
        }
    }

    let mut sorted_assessments = assessments;
    sorted_assessments.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    let recent_assessments = sorted_assessments.into_iter().take(10).collect();

    Ok(Json(DashboardStats {
        total_assessments,
        submitted,
        under_verification,
        verified,
        rejected,
        draft,
        recent_assessments,
    }))
}
