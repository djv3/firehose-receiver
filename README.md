# OTLP Firehose Receiver & Preprocessor

## Overview

This repo is just a fun side project and not intended to for production use. The acts as a sort of pre-processor for Base64 encoded OTLP data. The more idiomatic approach to take would be to extend the capabilities of the awsfirehose receiver in the OpenTelemetry Collector Contrib distribution, which currently only metrics.

## Reasoning

In constrained and latency sensitive environments such as aws lambda (when behaving as an ApiGateway proxy), I sought a performant way to asynchronously process OTLP data. Lambda in particular provides an interesting challenge as some lambdas may be deployed into a VPC, some may not be. This can make 'normal' http communication tricky, and sometimes flakey due to the way lambda can handle network connections during runtime freezes. To remedy this I trialed (along with a colleague) exporting to a Kinesis Stream which yielded positive results. Nothing is free however, so while exports are pleasantly snappy, the cost and complexity in allowing many producers to emit to Kinesis is not trivial.
Reading from Kinesis for a collector's benefit can additionally be more complex than it might seem. There are options like the Kinesis Client Library, but it does seem like there is a strong push from AWS to get you to use a Kinesis Data Firehose. With Firehose it is possible to send any records drawn from the stream to a HTTPS endpoint, however they are in the Kinesis format, which is base64 encoded. Furthermore if you were to extend the `awsfirehose` receiver in the OpenTelemetry Collector Contrib distribution to allow the processing of logs and traces as well, you would ideally want 3 kinesis streams and three matching firehoses, all pointing at different receivers. This is down to the assumptions made about the receiver - that a stream would only contain one type of data. Remedying that is again possible, but would be significant work.
Then what is the least worst option (or one of)? Well a small application which leverages the excellent Tonic and Axum in Rust to receive Firehose records, decode them into one of the OTLP formats (logs, metrics or traces) and then export them to a collector (I think... though I'm probably wrong).

## Responsibilities

- Serve an endpoint for Firehose to send records to
- Decode the records from base64
- Convert the raw bytes into an OTLP prost message (thanks to the forward thinking folks who authored the opentelemetry-proto crate)
- Send the decoded messages to a collector endpoint

## Usage

This application uses CLAP to parse command line arguments, to see the options available run:

```bash
cargo run -- --help
```

I'd expect the most common use case will be in a containerised environment, so I've included a basic Dockerfile and `compose.yaml` to ease the process of running it.

To run the compose application which includes a collector and the firehose receiver, run:

```bash
docker compose up -d --build
```

This will build the firehose receiver and start it alongside an OpenTelemetry Collector.

## Final Thoughts

Please don't use this for anything serious, it's really not intended for that.
