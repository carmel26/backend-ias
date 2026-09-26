use serde::{Deserialize, Serialize};

use surrealdb::RecordId;

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct DiscriminationThresholds {
    pub excellent: f64, // >= 0.40
    pub good: f64,      // >= 0.30
    pub fair: f64,      // >= 0.20
    pub poor: f64,      // >= 0.00
}

impl Default for DiscriminationThresholds {
    fn default() -> Self {
        Self {
            excellent: 0.40,
            good: 0.30,
            fair: 0.20,
            poor: 0.00,
        }
    }
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct SystemSettings {
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "crate::models::serialize_id")]
    pub id: Option<RecordId>,
    pub upper_lower_percentage: f64, // default 27.0
    pub thresholds: DiscriminationThresholds,
    pub schools: Vec<String>,
    pub assessment_types: Vec<String>,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct UpdateSettingsReq {
    pub upper_lower_percentage: Option<f64>,
    pub thresholds: Option<DiscriminationThresholds>,
    pub schools: Option<Vec<String>>,
    pub assessment_types: Option<Vec<String>>,
}
