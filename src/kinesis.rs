use serde::{Deserialize, Serialize};


#[derive(Debug, Deserialize)]
pub struct KinesisRecord {
    pub data: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FirehoseRequest {
    pub request_id: String,
    pub timestamp: usize,
    pub records: Vec<KinesisRecord>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FirehoseResponse {
    pub request_id: String,
    pub timestamp: usize,
    pub error_message: Option<String>,
}
