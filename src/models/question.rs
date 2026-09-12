use serde::{Deserialize, Serialize};
use surrealdb::types::{RecordId, SurrealValue};

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct QuestionItem {
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "crate::models::serialize_id")]
    pub id: Option<RecordId>,
    pub assessment_id: String,
    pub question_number: usize,
    pub prompt: Option<String>,
    pub upper_correct: usize,
    pub upper_wrong: usize,
    pub lower_correct: usize,
    pub lower_wrong: usize,
    pub difficulty_index: f64,
    pub discrimination_index: f64,
    pub rasch_difficulty: f64,
    pub interpretation: String,
    pub decision: String,
    pub classification: String,
    pub recommendation: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateQuestionReq {
    pub question_number: usize,
    pub prompt: Option<String>,
    pub upper_correct: usize,
    pub lower_correct: usize,
}

#[derive(Debug, Deserialize)]
pub struct QuestionInput {
    pub question_number: usize,
    pub prompt: Option<String>,
    pub upper_correct: usize,
    pub lower_correct: usize,
}

#[derive(Debug, Deserialize)]
pub struct BatchQuestionsReq {
    pub questions: Vec<QuestionInput>,
}
