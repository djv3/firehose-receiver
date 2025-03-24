use crate::{kinesis, logs, metrics, traces};
use axum::{Json, extract::State, response::IntoResponse};
use opentelemetry_proto::tonic::collector::{
    logs::v1::logs_service_client::LogsServiceClient,
    metrics::v1::metrics_service_client::MetricsServiceClient,
    trace::v1::trace_service_client::TraceServiceClient,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::Channel;


#[derive(Clone, Debug)]
pub struct AppState {
    pub log_client: Arc<Mutex<LogsServiceClient<Channel>>>,
    pub metric_client: Arc<Mutex<MetricsServiceClient<Channel>>>,
    pub trace_client: Arc<Mutex<TraceServiceClient<Channel>>>,
}

enum TelemetrySignal {
    Log(logs::LogRequest),
    Metric(metrics::MetricRequest),
    Trace(traces::TraceRequest),
    UnparseableRecord(String),
}

pub async fn create_clients(
    export_address: String,
) -> (
    LogsServiceClient<Channel>,
    MetricsServiceClient<Channel>,
    TraceServiceClient<Channel>,
) {
    let lc = logs::create_log_client(export_address.clone())
        .await
        .unwrap();
    let mc = metrics::create_metric_client(export_address.clone())
        .await
        .unwrap();
    let tc = traces::create_trace_client(export_address.clone())
        .await
        .unwrap();
    (lc, mc, tc)
}

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<kinesis::FirehoseRequest>,
) -> impl IntoResponse {
    let kinesis::FirehoseRequest {
        request_id,
        timestamp,
        records,
    } = request;

    let (valid, invalid): (Vec<TelemetrySignal>, Vec<TelemetrySignal>) = records
        .iter()
        .map(|record| {
            if let Ok(r) = logs::LogRequest::try_from(record.data.clone()) {
                return TelemetrySignal::Log(r);
            }
            if let Ok(r) = metrics::MetricRequest::try_from(record.data.clone()) {
                return TelemetrySignal::Metric(r);
            }
            if let Ok(r) = traces::TraceRequest::try_from(record.data.clone()) {
                return TelemetrySignal::Trace(r);
            }
            TelemetrySignal::UnparseableRecord(record.data.clone())
        })
        .partition(|result| !matches!(result, TelemetrySignal::UnparseableRecord(_)));

    if valid.is_empty() {
        return Json(kinesis::FirehoseResponse {
            request_id,
            timestamp,
            error_message: Some("All records failed to parse".into()),
        });
    }

    if !invalid.is_empty() {
        for problem in invalid {
            match problem {
                TelemetrySignal::UnparseableRecord(err) => eprintln!("{err}"),
                _ => (),
            }
        }
    }

    for parsed in valid {
        match parsed {
            TelemetrySignal::Log(log_request) => {
                let mut client = state.log_client.lock().await;
                if let Err(e) = client.export(log_request.0).await {
                    eprintln!("Error exporting logs: {:?}", e);
                }
            }
            TelemetrySignal::Metric(metric_request) => {
                let mut client = state.metric_client.lock().await;
                if let Err(e) = client.export(metric_request.0).await {
                    eprintln!("Error exporting metrics: {:?}", e);
                }
            }
            TelemetrySignal::Trace(trace_request) => {
                let mut client = state.trace_client.lock().await;
                if let Err(e) = client.export(trace_request.0).await {
                    eprintln!("Error exporting traces: {:?}", e);
                }
            }
            _ => (),
        }
    }

    Json(kinesis::FirehoseResponse {
        request_id,
        timestamp,
        error_message: None,
    })
}
