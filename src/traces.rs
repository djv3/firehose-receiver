use base64::prelude::*;
use opentelemetry_proto::tonic::collector::trace::v1::{
    ExportTraceServiceRequest, trace_service_client::TraceServiceClient,
};
use prost::Message;
use tonic::transport::Channel;

pub async fn create_trace_client(
    addr: String,
) -> Result<TraceServiceClient<Channel>, tonic::transport::Error> {
    let channel = tonic::transport::Channel::from_shared(addr)
        .unwrap()
        .connect()
        .await?;
    Ok(TraceServiceClient::new(channel))
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
