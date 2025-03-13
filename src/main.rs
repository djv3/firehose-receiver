use axum::{Json, Router, extract::State, response::IntoResponse, routing::post};
use clap::Parser;
use logs::LogRequest;
use metrics::MetricRequest;
use opentelemetry_proto::tonic::collector::{
    logs::v1::logs_service_client::LogsServiceClient,
    metrics::v1::metrics_service_client::MetricsServiceClient,
    trace::v1::trace_service_client::TraceServiceClient,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::Channel;
use trace::TraceRequest;

mod logs;
mod metrics;
mod trace;

#[derive(Debug, Deserialize)]
struct KinesisRecord {
    data: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct FirehoseRequest {
    request_id: String,
    timestamp: usize,
    records: Vec<KinesisRecord>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FirehoseResponse {
    request_id: String,
    timestamp: usize,
    error_message: Option<String>,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// OpenTelemetry collector endpoint
    #[arg(short, long, default_value = "http://localhost:4317")]
    export_address: String,

    /// Endpoint to listen on
    #[arg(short, long, default_value = "http://localhost:5318")]
    receiver_address: String,
}

#[derive(Clone, Debug)]
struct AppState {
    log_client: Arc<Mutex<LogsServiceClient<Channel>>>,
    metric_client: Arc<Mutex<MetricsServiceClient<Channel>>>,
    trace_client: Arc<Mutex<TraceServiceClient<Channel>>>,
}

enum TelemetrySignal {
    Log(LogRequest),
    Metric(MetricRequest),
    Trace(TraceRequest),
    UnparseableRecord(String),
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let Args {
        export_address,
        receiver_address,
    } = args;

    let lc = logs::create_log_client(export_address.clone())
        .await
        .unwrap();
    let mc = metrics::create_metric_client(export_address.clone())
        .await
        .unwrap();
    let tc = trace::create_trace_client(export_address.clone())
        .await
        .unwrap();

    let state = Arc::new(AppState {
        log_client: Arc::new(Mutex::new(lc)),
        metric_client: Arc::new(Mutex::new(mc)),
        trace_client: Arc::new(Mutex::new(tc)),
    });

    let app = Router::new().route("/", post(handler)).with_state(state);

    let listener = tokio::net::TcpListener::bind(receiver_address)
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}

async fn handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<FirehoseRequest>,
) -> impl IntoResponse {
    let FirehoseRequest {
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
            if let Ok(r) = trace::TraceRequest::try_from(record.data.clone()) {
                return TelemetrySignal::Trace(r);
            }
            TelemetrySignal::UnparseableRecord(record.data.clone())
        })
        .partition(|result| !matches!(result, TelemetrySignal::UnparseableRecord(_)));

    if valid.is_empty() {
        return Json(FirehoseResponse {
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

    Json(FirehoseResponse {
        request_id,
        timestamp,
        error_message: None,
    })
}
