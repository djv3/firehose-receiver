use base64::{Engine, prelude::BASE64_STANDARD};
use opentelemetry_proto::tonic::collector::trace::v1::{
    ExportTraceServiceRequest, trace_service_client::TraceServiceClient,
};
use prost::Message;
use tonic::transport::Channel;

pub async fn create_trace_client(
    addr: String,
) -> Result<TraceServiceClient<Channel>, tonic::transport::Error> {
    TraceServiceClient::connect(addr).await
}

pub struct TraceRequest(pub ExportTraceServiceRequest);

impl TryFrom<String> for TraceRequest {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let decoded = BASE64_STANDARD.decode(value)?;
        let bytes = prost::bytes::Bytes::from(decoded);
        let decoded = ExportTraceServiceRequest::decode(bytes)?;
        Ok(TraceRequest(decoded))
    }
}
