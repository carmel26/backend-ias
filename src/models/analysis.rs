use serde::{Deserialize, Serialize};

use surrealdb::types::{SurrealValue};
use super::question::QuestionItem;

#[derive(Debug, Clone, Serialize, Deserialize, Default, SurrealValue)]
pub struct ClassificationCount {
    pub excellent: usize,
    pub good: usize,
    pub fair: usize,
    pub poor: usize,
    pub negative: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct ItemAnalysisSummary {
    pub total_questions: usize,
    pub mean_difficulty: f64,
    pub mean_discrimination: f64,
    pub classification_counts: ClassificationCount,
    pub overall_recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct FullAnalysisResult {
    pub summary: ItemAnalysisSummary,
    pub questions: Vec<QuestionItem>,
}
