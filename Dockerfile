# syntax=docker/dockerfile:1.5

FROM rust:1.95.0-slim AS builder
WORKDIR /usr/src/status-service

# Cache dependencies first by copying manifest files.
COPY Cargo.toml Cargo.lock ./
COPY clippy.toml rustfmt.toml ./

# Install musl target for a smaller Alpine-compatible runtime image.
RUN rustup target add x86_64-unknown-linux-musl \
    && apt-get update \
    && apt-get install -y --no-install-recommends musl-tools \
    && rm -rf /var/lib/apt/lists/*

# Copy source and build in release mode for a static musl binary.
COPY src ./src
RUN cargo build --release --target x86_64-unknown-linux-musl

# Copy sample config separately so the runtime image can start without baking secrets.
COPY config.example.yaml ./config.example.yaml

FROM alpine:3.19 AS runtime

# Create a non-root app user.
RUN addgroup -S status && adduser -S -G status status
WORKDIR /app

# Install only what is needed for runtime and healthcheck.
RUN apk add --no-cache ca-certificates curl

COPY --from=builder /usr/src/status-service/target/x86_64-unknown-linux-musl/release/status-service ./status-service
COPY --from=builder /usr/src/status-service/config.example.yaml ./config.yaml
RUN chown status:status /app/status-service /app/config.yaml

USER status
EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=5s --start-period=15s --retries=3 \
  CMD curl -fsS http://127.0.0.1:3000/healthcheck || exit 1

ENTRYPOINT ["./status-service"]
