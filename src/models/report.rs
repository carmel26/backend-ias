use serde::{Deserialize, Serialize};
use super::assessment::Assessment;
use super::question::QuestionItem;
use super::analysis::ItemAnalysisSummary;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentReport {
    pub assessment: Assessment,
    pub summary: ItemAnalysisSummary,
    pub questions: Vec<QuestionItem>,
    pub generated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    pub total_assessments: usize,
    pub submitted: usize,
    pub under_verification: usize,
    pub verified: usize,
    pub rejected: usize,
    pub draft: usize,
    pub recent_assessments: Vec<Assessment>,
}
