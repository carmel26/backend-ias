use axum::{
    extract::{Path, State},
    Json,
};
use crate::db::AppDb;
use crate::error::AppError;
use crate::handlers::analysis_handler::calculate_summary;
use crate::models::*;

pub async fn get_assessment_report(
    State(db): State<AppDb>,
    Path(assessment_id): Path<String>,
) -> Result<Json<AssessmentReport>, AppError> {
    let mut assessments: Vec<Assessment> = db
        .query("SELECT * FROM assessment WHERE id = type::thing('assessment', $id) OR id = $id")
        .bind(("id", assessment_id.clone()))
        .await?
        .take(0)?;

    let assessment = assessments
        .pop()
        .ok_or_else(|| AppError::NotFound("Assessment not found".to_string()))?;

    let mut questions: Vec<QuestionItem> = db
        .query("SELECT * FROM question WHERE assessment_id = $aid ORDER BY question_number ASC")
        .bind(("aid", assessment_id))
        .await?
        .take(0)?;

    questions.sort_by_key(|q| q.question_number);
    let summary = calculate_summary(&questions);

    Ok(Json(AssessmentReport {
        assessment,
        summary,
        questions,
        generated_at: chrono::Utc::now().to_rfc3339(),
    }))
}
