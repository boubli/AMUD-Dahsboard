# Stage 1: Build stage
FROM rust:1.75-slim AS builder

WORKDIR /usr/src/amud

# Install compilation dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace source files
COPY . .

# Compile release binaries
RUN cargo build --release

# Stage 2: Runtime stage
FROM debian:bookworm-slim

# Install system runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    sqlite3 \
    libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy compiled binaries from builder stage
COPY --from=builder /usr/src/amud/target/release/amud-server /app/amud-server
COPY --from=builder /usr/src/amud/target/release/amud-agent /app/amud-agent

# Copy static assets and UI files needed at runtime
COPY --from=builder /usr/src/amud/ui /app/ui

# Ensure data directory exists and set it as a volume
RUN mkdir -p /app/data
VOLUME /app/data

# Expose port 8000
EXPOSE 8000

# Set entrypoint to run the server
ENTRYPOINT ["/app/amud-server"]
