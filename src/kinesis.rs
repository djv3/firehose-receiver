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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_firehose_request_deserialize() {
        let json = r#"{
            "requestId": "test-request-id",
            "timestamp": 1234567890,
            "records": [
                { "data": "b64dataOne" },
                { "data": "b64dataTwo" }
            ]
        }"#;

        let request: FirehoseRequest = serde_json::from_str(json).unwrap();

        assert_eq!(request.request_id, "test-request-id");
        assert_eq!(request.timestamp, 1234567890);
        assert_eq!(request.records.len(), 2);
        assert_eq!(request.records[0].data, "b64dataOne");
        assert_eq!(request.records[1].data, "b64dataTwo");
    }

    #[test]
    fn test_firehose_response_serialize() {
        let response = FirehoseResponse {
            request_id: "test-request-id".to_string(),
            timestamp: 1234567890,
            error_message: Some("test error".to_string()),
        };

        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains("\"requestId\":\"test-request-id\""));
        assert!(json.contains("\"timestamp\":1234567890"));
        assert!(json.contains("\"errorMessage\":\"test error\""));
    }
}
