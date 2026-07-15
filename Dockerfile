# Build stage
FROM rust:1.85-slim@sha256:9f841bbe9e7d8e37ceb96ed907265a3a0df7f44e3737d0b100e7907a679acb36 AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    cmake \
    libclang-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY benches ./benches

RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim@sha256:7b140f374b289a7c2befc338f42ebe6441b7ea838a042bbd5acbfca6ec875818

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/model2vec-serve /usr/local/bin/model2vec-serve

EXPOSE 8080

ENTRYPOINT ["model2vec-serve"]
