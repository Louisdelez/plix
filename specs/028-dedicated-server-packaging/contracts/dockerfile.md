# Contract: Dockerfile

**Feature**: 028-dedicated-server-packaging
**Type**: Docker Build Specification

## Dockerfile (plix-server)

```dockerfile
# =============================================================================
# Stage 1: Builder
# =============================================================================
FROM debian:bookworm-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    build-essential \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Install Rust toolchain (version pinned via rust-toolchain.toml)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none
ENV PATH="/root/.cargo/bin:${PATH}"

# Copy workspace files
WORKDIR /build
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates ./crates

# Build release binary
RUN cargo build --release --bin plix-server

# =============================================================================
# Stage 2: Runtime
# =============================================================================
FROM debian:bookworm-slim

# Install minimal runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -g 1000 plix && \
    useradd -u 1000 -g plix -m -s /bin/bash plix

# Create data directories
RUN mkdir -p /data/config /data/worlds /data/logs /app/assets && \
    chown -R plix:plix /data /app

# Copy binary from builder
COPY --from=builder /build/target/release/plix-server /app/plix-server

# Copy assets
COPY --chown=plix:plix assets /app/assets

WORKDIR /app
USER plix

# Default environment
ENV PLIX_ASSETS_DIR=/app/assets
ENV PLIX_DATA_DIR=/data
ENV RUST_LOG=info

# Expose game port
EXPOSE 7777/udp

# Health check (process-based)
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD pgrep -x plix-server || exit 1

ENTRYPOINT ["/app/plix-server"]
CMD ["--assets-dir", "/app/assets"]
```

## Dockerfile.master (plix-master)

```dockerfile
# =============================================================================
# Stage 1: Builder
# =============================================================================
FROM debian:bookworm-slim AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    build-essential \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain none
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /build
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates ./crates

RUN cargo build --release --bin plix-master

# =============================================================================
# Stage 2: Runtime
# =============================================================================
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd -g 1000 plix && \
    useradd -u 1000 -g plix -m -s /bin/bash plix

COPY --from=builder /build/target/release/plix-master /app/plix-master

WORKDIR /app
USER plix

ENV RUST_LOG=info

EXPOSE 8080/tcp

HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

ENTRYPOINT ["/app/plix-master"]
CMD ["--bind", "0.0.0.0:8080"]
```

## Build Requirements

- Docker Engine 20.10+
- BuildKit enabled (for efficient caching)
- ~2GB disk space for builder stage
- ~100MB for final runtime image

## Expected Outputs

| Image | Size Target | Ports |
|-------|-------------|-------|
| plix-server | <100MB | 7777/udp |
| plix-master | <80MB | 8080/tcp |
