use serde::{Deserialize, Serialize};

use surrealdb::types::{RecordId, SurrealValue};

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue, PartialEq, Eq)]
pub enum AssessmentStatus {
    Draft,
    Submitted,
    #[surreal(rename = "Under Verification")]
    UnderVerification,
    Verified,
    Rejected,
}

impl std::fmt::Display for AssessmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssessmentStatus::Draft => write!(f, "Draft"),
            AssessmentStatus::Submitted => write!(f, "Submitted"),
            AssessmentStatus::UnderVerification => write!(f, "Under Verification"),
            AssessmentStatus::Verified => write!(f, "Verified"),
            AssessmentStatus::Rejected => write!(f, "Rejected"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct Assessment {
    #[surreal(skip)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RecordId>,

    pub user_id: String,
    pub lecturer_name: String,
    pub school: String,
    pub subject: String,
    pub assessment_type: String,
    pub num_students: usize,
    pub title: String,
    pub date: String,
    pub upper_percentage: f64,
    pub upper_group_size: usize,
    pub lower_group_size: usize,
    pub middle_group_size: usize,
    pub status: AssessmentStatus,
    pub admin_feedback: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateAssessmentReq {
    pub subject: String,
    pub assessment_type: String,
    pub num_students: usize,
    pub title: String,
    pub date: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAssessmentReq {
    pub subject: String,
    pub assessment_type: String,
    pub num_students: usize,
    pub title: String,
    pub date: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyAssessmentReq {
    pub status: AssessmentStatus,
    pub feedback: Option<String>,
}