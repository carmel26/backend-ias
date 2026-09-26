use serde::{Deserialize, Serialize};

use surrealdb::types::{SurrealValue};
use super::assessment::Assessment;
use super::question::QuestionItem;
use super::analysis::ItemAnalysisSummary;

#[derive(Debug, Deserialize, Clone, Serialize, SurrealValue)]
pub struct AssessmentReport {
    pub assessment: Assessment,
    pub summary: ItemAnalysisSummary,
    pub questions: Vec<QuestionItem>,
    pub generated_at: String,
}

#[derive(Debug, Deserialize, Clone, Serialize, SurrealValue)]
pub struct DashboardStats {
    pub total_assessments: usize,
    pub submitted: usize,
    pub under_verification: usize,
    pub verified: usize,
    pub rejected: usize,
    pub draft: usize,
    pub recent_assessments: Vec<Assessment>,
}
