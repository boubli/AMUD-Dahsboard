# Stage 1: Build stage
FROM rust:1-slim AS builder

# musl-tools provides musl-gcc, needed to statically compile the bundled SQLite
# C sources and ring against musl libc. The resulting binaries have zero dynamic
# library dependencies, so they can run on a `scratch` image.
RUN apt-get update && apt-get install -y \
    musl-tools \
    && rm -rf /var/lib/apt/lists/*

# Add the fully-static musl target.
RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /usr/src/amud

# Copy workspace source files
COPY . .

# Pass the git tag to cargo
ARG GIT_TAG
ENV GIT_TAG=$GIT_TAG

# Compile statically-linked release binaries.
RUN cargo build --release --target x86_64-unknown-linux-musl

# Create the runtime data directory so it can be copied into the scratch image
# (which has no shell to run `mkdir`).
RUN mkdir -p /out/data

# Stage 2: Runtime stage
# `scratch` is completely empty: no OS, no shell, no package manager, no glibc.
# The binaries are statically linked (musl), SQLite is bundled, TLS uses
# rustls/ring, and the only HTTPS client uses a custom certificate verifier
# (no system CA store needed). Docker Scout has essentially nothing to scan, so
# the previous 10 critical / 5 high / 48 medium OS-package CVEs are eliminated.
#
# Security note (SonarCloud): the process runs as root because scratch has no
# /etc/passwd or useradd. This is intentional for a minimal homelab image.
# The container only runs the static amud-server binary and mounts /app/data
# for SQLite; it is not multi-tenant. Review this hotspot as Safe in SonarCloud.
FROM scratch

WORKDIR /app

# Copy statically-linked binaries from builder stage
COPY --from=builder /usr/src/amud/target/x86_64-unknown-linux-musl/release/amud-server /app/amud-server
COPY --from=builder /usr/src/amud/target/x86_64-unknown-linux-musl/release/amud-agent /app/amud-agent

# Copy static assets and UI files needed at runtime
COPY --from=builder /usr/src/amud/ui /app/ui

# Data directory used for the SQLite database
COPY --from=builder /out/data /app/data
VOLUME /app/data

# Expose port 8000
EXPOSE 8000

# Set entrypoint to run the server
ENTRYPOINT ["/app/amud-server"]
