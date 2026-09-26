use axum::{
    extract::{Path, State},
    Json,
};
use crate::auth::Claims;
use crate::db::AppDb;
use crate::error::AppError;
use crate::models::*;

fn compute_question_item(
    assessment_id: &str,
    q_num: usize,
    prompt: Option<String>,
    upper_c: usize,
    lower_c: usize,
    group_size: usize,
    _thresholds: &DiscriminationThresholds,
) -> Result<QuestionItem, AppError> {
    if upper_c > group_size {
        return Err(AppError::BadRequest(format!(
            "Question {}: Upper group correct ({}) exceeds calculated group size ({})",
            q_num, upper_c, group_size
        )));
    }

    if lower_c > group_size {
        return Err(AppError::BadRequest(format!(
            "Question {}: Lower group correct ({}) exceeds calculated group size ({})",
            q_num, lower_c, group_size
        )));
    }

    let upper_w = group_size - upper_c;
    let lower_w = group_size - lower_c;

    let total_group = 2 * group_size;
    let diff_idx = if total_group > 0 {
        (upper_c + lower_c) as f64 / total_group as f64
    } else {
        0.0
    };

    let disc_idx = if group_size > 0 {
        (upper_c as f64 - lower_c as f64) / group_size as f64
    } else {
        0.0
    };

    let diff_idx_rounded = (diff_idx * 1000.0).round() / 1000.0;
    let disc_idx_rounded = (disc_idx * 1000.0).round() / 1000.0;

    // Rasch Difficulty calculation = LN((1 - P) / P)
    let p_clamped = diff_idx.clamp(0.001, 0.999);
    let rasch_diff = ((1.0 - p_clamped) / p_clamped).ln();
    let rasch_difficulty = (rasch_diff * 1000.0).round() / 1000.0;

    // Interpretation provided by:
    // IF(J5>=0.4,"Very Good Item", IF(J5>=0.3,"Good Item", IF(J5>=0.2,"Acceptable Item",IF(J5>=0,"Poor Item",IF(J5<0,"Very poor Item")))))
    let interpretation = if disc_idx >= 0.4 {
        "Very Good Item"
    } else if disc_idx >= 0.3 {
        "Good Item"
    } else if disc_idx >= 0.2 {
        "Acceptable Item"
    } else if disc_idx >= 0.0 {
        "Poor Item"
    } else {
        "Very poor Item"
    };

    // Decision options: retain, retain/revise, revise, remove (with why in parentheses)
    let decision = if disc_idx >= 0.4 {
        "retain (Very good discrimination ability)"
    } else if disc_idx >= 0.3 {
        "retain/revise (Good item, minor adjustments if necessary)"
    } else if disc_idx >= 0.2 {
        "revise (Acceptable item, needs revision or distractor review)"
    } else if disc_idx >= 0.0 {
        "revise (Poor item, major overhaul required)"
    } else {
        "remove (Very poor item, negative discrimination or incorrect key)"
    };

    Ok(QuestionItem {
        id: None,
        assessment_id: assessment_id.to_string(),
        question_number: q_num,
        prompt,
        upper_correct: upper_c,
        upper_wrong: upper_w,
        lower_correct: lower_c,
        lower_wrong: lower_w,
        difficulty_index: diff_idx_rounded,
        discrimination_index: disc_idx_rounded,
        rasch_difficulty,
        interpretation: interpretation.to_string(),
        decision: decision.to_string(),
        classification: interpretation.to_string(),
        recommendation: decision.to_string(),
    })
}

pub async fn list_questions(
    State(db): State<AppDb>,
    Path(assessment_id): Path<String>,
) -> Result<Json<Vec<QuestionItem>>, AppError> {
    let mut questions: Vec<QuestionItem> = db
        .query("SELECT * FROM question WHERE assessment_id = $aid ORDER BY question_number ASC")
        .bind(("aid", assessment_id))
        .await?
        .take(0)?;

    questions.sort_by_key(|q| q.question_number);
    Ok(Json(questions))
}

pub async fn add_question(
    State(db): State<AppDb>,
    claims: Claims,
    Path(assessment_id): Path<String>,
    Json(req): Json<CreateQuestionReq>,
) -> Result<Json<QuestionItem>, AppError> {
    let mut assessments: Vec<Assessment> = db
        .query("SELECT * FROM assessment WHERE id = type::record('assessment', $id) OR id = $id")
        .bind(("id", assessment_id.clone()))
        .await?
        .take(0)?;

    let asm = assessments
        .pop()
        .ok_or_else(|| AppError::NotFound("Assessment not found".to_string()))?;

    if claims.role == UserRole::Lecturer && asm.user_id != claims.sub {
        return Err(AppError::Forbidden("You do not own this assessment".to_string()));
    }

    let settings: Vec<SystemSettings> = db.query("SELECT * FROM settings").await?.take(0)?;
    let thresholds = settings
        .first()
        .map(|s| s.thresholds.clone())
        .unwrap_or_default();

    let question = compute_question_item(
        &assessment_id,
        req.question_number,
        req.prompt,
        req.upper_correct,
        req.lower_correct,
        asm.upper_group_size,
        &thresholds,
    )?;

    let q_key = format!("q_{}_{}", assessment_id, question.question_number);
    let created: Option<QuestionItem> = db
        .create(("question", q_key.as_str()))
        .content(question.clone())
        .await?;

    Ok(Json(created.unwrap_or(question)))
}

pub async fn batch_save_questions(
    State(db): State<AppDb>,
    claims: Claims,
    Path(assessment_id): Path<String>,
    Json(req): Json<BatchQuestionsReq>,
) -> Result<Json<Vec<QuestionItem>>, AppError> {
    let mut assessments: Vec<Assessment> = db
        .query("SELECT * FROM assessment WHERE id = type::record('assessment', $id) OR id = $id")
        .bind(("id", assessment_id.clone()))
        .await?
        .take(0)?;

    let asm = assessments
        .pop()
        .ok_or_else(|| AppError::NotFound("Assessment not found".to_string()))?;

    if claims.role == UserRole::Lecturer && asm.user_id != claims.sub {
        return Err(AppError::Forbidden("You do not own this assessment".to_string()));
    }

    let settings: Vec<SystemSettings> = db.query("SELECT * FROM settings").await?.take(0)?;
    let thresholds = settings
        .first()
        .map(|s| s.thresholds.clone())
        .unwrap_or_default();

    // Delete existing questions for this assessment first
    let _: Vec<QuestionItem> = db
        .query("DELETE FROM question WHERE assessment_id = $aid")
        .bind(("aid", assessment_id.clone()))
        .await?
        .take(0)?;

    let mut created_questions = Vec::new();

    for q_input in req.questions {
        let question = compute_question_item(
            &assessment_id,
            q_input.question_number,
            q_input.prompt,
            q_input.upper_correct,
            q_input.lower_correct,
            asm.upper_group_size,
            &thresholds,
        )?;

        let q_key = format!("q_{}_{}", assessment_id, question.question_number);
        let created: Option<QuestionItem> = db
            .create(("question", q_key.as_str()))
            .content(question.clone())
            .await?;

        if let Some(q) = created {
            created_questions.push(q);
        } else {
            created_questions.push(question);
        }
    }

    created_questions.sort_by_key(|q| q.question_number);
    Ok(Json(created_questions))
}

pub async fn delete_question(
    State(db): State<AppDb>,
    claims: Claims,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut questions: Vec<QuestionItem> = db
        .query("SELECT * FROM question WHERE id = $id")
        .bind(("id", id.clone()))
        .await?
        .take(0)?;

    let question = questions
        .pop()
        .ok_or_else(|| AppError::NotFound("Question not found".to_string()))?;

    // Check ownership via assessment
    let mut assessments: Vec<Assessment> = db
        .query("SELECT * FROM assessment WHERE id = $id")
        .bind(("id", question.assessment_id))
        .await?
        .take(0)?;

    if let Some(asm) = assessments.pop() {
        if claims.role == UserRole::Lecturer && asm.user_id != claims.sub {
            return Err(AppError::Forbidden("You do not own this assessment".to_string()));
        }
    }

    let _: Option<QuestionItem> = db.delete(("question", id.as_str())).await?;
    Ok(Json(serde_json::json!({ "message": "Question deleted" })))
}
