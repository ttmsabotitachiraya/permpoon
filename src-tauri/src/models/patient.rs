use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PatientInfo {
    pub hn: String,
    pub fname: String,
    pub lname: String,
    pub cid: String,
    pub pttype: String,
    pub pttype_name: String,
    pub hipdata_code: String,
    pub dob: String, // YYYY-MM-DD
    pub sex: String, // 'M' or 'F' or '1'/'2'
    pub age: i32,
}

#[derive(Debug, Deserialize)]
pub struct PatientQuery {
    pub hn: Option<String>,
    pub cid: Option<String>,
    pub fname: Option<String>,
    pub lname: Option<String>,
}
