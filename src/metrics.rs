use base64::{Engine, prelude::BASE64_STANDARD};
use opentelemetry_proto::tonic::collector::metrics::v1::{
    ExportMetricsServiceRequest,
    metrics_service_client::MetricsServiceClient,
};
use prost::Message;
use tonic::transport::Channel;

pub async fn create_metric_client(
    addr: String,
) -> Result<MetricsServiceClient<Channel>, tonic::transport::Error> {
    MetricsServiceClient::connect(addr).await
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
