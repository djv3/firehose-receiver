FROM rust:1.85.0 AS builder

COPY Cargo.toml Cargo.lock ./

COPY src ./src/

RUN cargo fetch

RUN cargo build --release

FROM debian:stable-slim

COPY --from=builder /target/release/kinesis-handler .
