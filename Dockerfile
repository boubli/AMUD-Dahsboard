# syntax=docker/dockerfile:1

# Stage 1: Build stage
FROM rust:1-slim AS builder

ARG TARGETARCH
ARG GIT_TAG
ENV GIT_TAG=$GIT_TAG

RUN apt-get update && apt-get install -y \
    musl-tools \
    gcc-aarch64-linux-gnu \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-musl

WORKDIR /usr/src/amud

COPY . .

ENV CARGO_NET_RETRY=10
ENV CARGO_HTTP_MULTIPLEXING=false

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    set -euo pipefail; \
    case "${TARGETARCH}" in \
      amd64) export RUST_TARGET=x86_64-unknown-linux-musl ;; \
      arm64) export RUST_TARGET=aarch64-unknown-linux-musl ;; \
      *) echo "Unsupported TARGETARCH: ${TARGETARCH}" >&2; exit 1 ;; \
    esac; \
    for attempt in 1 2 3; do \
      cargo build --release --target "${RUST_TARGET}" && exit 0; \
      echo "cargo build attempt ${attempt} failed, retrying..."; \
      sleep $((attempt * 15)); \
    done; \
    exit 1

RUN mkdir -p /out/data /out/bin && \
    case "${TARGETARCH}" in \
      amd64) \
        cp target/x86_64-unknown-linux-musl/release/amud-server /out/bin/amud-server; \
        cp target/x86_64-unknown-linux-musl/release/amud-agent /out/bin/amud-agent; \
        ;; \
      arm64) \
        cp target/aarch64-unknown-linux-musl/release/amud-server /out/bin/amud-server; \
        cp target/aarch64-unknown-linux-musl/release/amud-agent /out/bin/amud-agent; \
        ;; \
    esac

# Stage 2: Runtime stage
FROM scratch

WORKDIR /app

COPY --from=builder /out/bin/amud-server /app/amud-server
COPY --from=builder /out/bin/amud-agent /app/amud-agent
COPY --from=builder /usr/src/amud/ui /app/ui
COPY --from=builder /out/data /app/data

VOLUME /app/data

EXPOSE 8000

ENTRYPOINT ["/app/amud-server"]
