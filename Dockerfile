FROM rust:latest AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin uma-api

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/uma-api /usr/local/bin/uma-api
CMD ["uma-api"]