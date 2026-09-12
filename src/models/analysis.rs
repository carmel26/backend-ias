use serde::{Deserialize, Serialize};
use super::question::QuestionItem;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClassificationCount {
    pub excellent: usize,
    pub good: usize,
    pub fair: usize,
    pub poor: usize,
    pub negative: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemAnalysisSummary {
    pub total_questions: usize,
    pub mean_difficulty: f64,
    pub mean_discrimination: f64,
    pub classification_counts: ClassificationCount,
    pub overall_recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullAnalysisResult {
    pub summary: ItemAnalysisSummary,
    pub questions: Vec<QuestionItem>,
}
