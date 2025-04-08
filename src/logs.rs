use base64::prelude::*;
use opentelemetry_proto::tonic::collector::logs::v1::{
    ExportLogsServiceRequest, logs_service_client::LogsServiceClient,
};
use prost::Message;
use tonic::transport::Channel;
pub async fn create_log_client(
    addr: String,
) -> Result<LogsServiceClient<Channel>, tonic::transport::Error> {
    let channel = tonic::transport::Channel::from_shared(addr)
        .unwrap()
        .connect()
        .await?;
    Ok(LogsServiceClient::new(channel))
}

pub struct LogRequest(pub ExportLogsServiceRequest);

impl TryFrom<String> for LogRequest {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let decoded = BASE64_STANDARD.decode(value)?;
        let bytes = prost::bytes::Bytes::from(decoded);
        let decoded = ExportLogsServiceRequest::decode(bytes)?;
        Ok(LogRequest(decoded))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_request_try_from() {
        let request = ExportLogsServiceRequest::default();

        let mut buf = Vec::new();
        request.encode(&mut buf).unwrap();
        let encoded = BASE64_STANDARD.encode(&buf);

        let result = LogRequest::try_from(encoded);

        assert!(result.is_ok());
    }

    #[test]
    fn test_log_request_try_from_invalid_data() {
        let result = LogRequest::try_from("not base64".to_string());
        assert!(result.is_err());

        let encoded = BASE64_STANDARD.encode("not valid protobuf");
        let result = LogRequest::try_from(encoded);
        assert!(result.is_err());
    }
}
