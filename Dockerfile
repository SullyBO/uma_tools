FROM rust:latest AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin uma_api

FROM debian:trixie-slim
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/uma_api /usr/local/bin/uma_api
CMD ["uma_api"]