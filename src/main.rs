use axum::{Router, routing::post};
use clap::Parser;

use std::sync::Arc;
use tokio::sync::Mutex;

mod handler;
mod kinesis;
mod logs;
mod metrics;
mod traces;

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

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let Args {
        export_address,
        receiver_address,
    } = args;

    println!("Using export address: {:#?}", export_address);
    println!("Using receiver address: {:#?}", receiver_address);

    let (lc, mc, tc) = handler::create_clients(export_address).await;

    let state = Arc::new(handler::AppState {
        log_client: Arc::new(Mutex::new(lc)),
        metric_client: Arc::new(Mutex::new(mc)),
        trace_client: Arc::new(Mutex::new(tc)),
    });

    let app = Router::new()
        .route("/", post(handler::handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(receiver_address)
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
