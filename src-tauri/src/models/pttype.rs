use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PttypeRow {
    pub pttype: String,
    pub name: String,
}
