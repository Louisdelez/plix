# =============================================================================
# Plix Server Dockerfile - Multi-stage build for minimal runtime image
# Feature 028: Dedicated Server Packaging
# =============================================================================

# -----------------------------------------------------------------------------
# Stage 1: Builder
# Compiles plix-server in release mode
# -----------------------------------------------------------------------------
FROM debian:bookworm-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    build-essential \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install Rust toolchain (version from rust-toolchain.toml)
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- -y --default-toolchain 1.83.0 --profile minimal

# Copy workspace files
WORKDIR /build
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates ./crates

# Build release binary
RUN cargo build --release --bin plix-server

# -----------------------------------------------------------------------------
# Stage 2: Runtime
# Minimal image with only the binary and required assets
# -----------------------------------------------------------------------------
FROM debian:bookworm-slim

# Install minimal runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    procps \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user (UID 1000 for common host mapping)
RUN groupadd -g 1000 plix && \
    useradd -u 1000 -g plix -m -s /bin/bash plix

# Create data directories with correct ownership
RUN mkdir -p /data/config /data/worlds /data/logs /app/assets && \
    chown -R plix:plix /data /app

# Copy binary from builder
COPY --from=builder /build/target/release/plix-server /app/plix-server

# Copy assets (arenas, etc.)
COPY --chown=plix:plix assets /app/assets

# Set working directory
WORKDIR /app

# Switch to non-root user
USER plix

# Default environment variables
ENV PLIX_ASSETS_DIR=/app/assets \
    PLIX_DATA_DIR=/data \
    RUST_LOG=info

# Expose game port (UDP)
EXPOSE 7777/udp

# Health check - verify server process is running
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD pgrep -x plix-server || exit 1

# Entrypoint and default command
ENTRYPOINT ["/app/plix-server"]
CMD ["--assets-dir", "/app/assets"]
