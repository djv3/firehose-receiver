use base64::prelude::*;
use opentelemetry_proto::tonic::collector::metrics::v1::{
    ExportMetricsServiceRequest, metrics_service_client::MetricsServiceClient,
};
use prost::Message;
use tonic::transport::Channel;

pub async fn create_metric_client(
    addr: String,
) -> Result<MetricsServiceClient<Channel>, tonic::transport::Error> {
    let channel = tonic::transport::Channel::from_shared(addr)
        .unwrap()
        .connect()
        .await?;
    Ok(MetricsServiceClient::new(channel))
}

pub struct MetricRequest(pub ExportMetricsServiceRequest);

impl TryFrom<String> for MetricRequest {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let decoded = BASE64_STANDARD.decode(value)?;
        let bytes = prost::bytes::Bytes::from(decoded);
        let decoded = ExportMetricsServiceRequest::decode(bytes)?;
        Ok(MetricRequest(decoded))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_request_try_from() {
        let request = ExportMetricsServiceRequest::default();

        let mut buf = Vec::new();
        request.encode(&mut buf).unwrap();
        let encoded = BASE64_STANDARD.encode(&buf);

        let result = MetricRequest::try_from(encoded);

        assert!(result.is_ok());
    }

    #[test]
    fn test_metric_request_try_from_invalid_data() {
        let result = MetricRequest::try_from("not base64".to_string());
        assert!(result.is_err());

        let encoded = BASE64_STANDARD.encode("not valid protobuf");
        let result = MetricRequest::try_from(encoded);
        assert!(result.is_err());
    }
}
