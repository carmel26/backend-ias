use axum::{
    extract::{Path, State},
    Json,
};
use crate::db::AppDb;
use crate::error::AppError;
use crate::models::*;

pub fn calculate_summary(questions: &[QuestionItem]) -> ItemAnalysisSummary {
    if questions.is_empty() {
        return ItemAnalysisSummary {
            total_questions: 0,
            mean_difficulty: 0.0,
            mean_discrimination: 0.0,
            classification_counts: ClassificationCount::default(),
            overall_recommendation: "No question items added yet.".to_string(),
        };
    }

    let total_questions = questions.len();
    let sum_diff: f64 = questions.iter().map(|q| q.difficulty_index).sum();
    let sum_disc: f64 = questions.iter().map(|q| q.discrimination_index).sum();

    let mean_difficulty = (sum_diff / total_questions as f64 * 1000.0).round() / 1000.0;
    let mean_discrimination = (sum_disc / total_questions as f64 * 1000.0).round() / 1000.0;

    let mut counts = ClassificationCount::default();

    for q in questions {
        match q.interpretation.as_str() {
            "Very Good Item" | "Excellent" => counts.excellent += 1,
            "Good Item" | "Good" => counts.good += 1,
            "Acceptable Item" | "Fair" => counts.fair += 1,
            "Poor Item" | "Poor" => counts.poor += 1,
            "Very poor Item" | "Negative" => counts.negative += 1,
            _ => {},
        }
    }

    let overall_recommendation = if counts.negative > 0 {
        format!(
            "CRITICAL: {} question(s) have negative discrimination index! Review answer key and discard/rewrite these items immediately.",
            counts.negative
        )
    } else if mean_discrimination >= 0.40 {
        "Overall Assessment Quality: EXCELLENT. Items effectively differentiate between high and low performing students.".to_string()
    } else if mean_discrimination >= 0.30 {
        "Overall Assessment Quality: GOOD. Assessment items perform well. Minor revisions suggested for poor items.".to_string()
    } else if mean_discrimination >= 0.20 {
        "Overall Assessment Quality: FAIR. Several questions require item revision or distractor analysis.".to_string()
    } else {
        "Overall Assessment Quality: NEEDS IMPROVEMENT. High number of items show poor discrimination.".to_string()
    };

    ItemAnalysisSummary {
        total_questions,
        mean_difficulty,
        mean_discrimination,
        classification_counts: counts,
        overall_recommendation,
    }
}

pub async fn analyze_assessment(
    State(db): State<AppDb>,
    Path(assessment_id): Path<String>,
) -> Result<Json<FullAnalysisResult>, AppError> {
    let mut questions: Vec<QuestionItem> = db
        .query("SELECT * FROM question WHERE assessment_id = $aid ORDER BY question_number ASC")
        .bind(("aid", assessment_id))
        .await?
        .take(0)?;

    questions.sort_by_key(|q| q.question_number);
    let summary = calculate_summary(&questions);

    Ok(Json(FullAnalysisResult { summary, questions }))
}
