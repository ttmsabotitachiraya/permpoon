use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceHistoryRow {
    pub icode: String,
    pub vstdate: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecommendationItem {
    pub icode: String,
    pub service_name: String,
    pub department: Option<String>,
    pub pttype_alias: Vec<String>,
}
