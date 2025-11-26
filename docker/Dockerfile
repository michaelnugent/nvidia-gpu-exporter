# Build stage
FROM rust:slim-trixie AS builder

WORKDIR /build

# Copy manifest files
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build the release binary
RUN cargo build --release

# Runtime stage
FROM debian:trixie-slim

# Install basic dependencies
# Note: libnvidia-ml.so.1 should be provided by NVIDIA Container Toolkit
# when running with --gpus flag. The library is mounted from the host.
RUN sed -i 's/ main$/ main contrib non-free/g' /etc/apt/sources.list.d/debian.sources
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    libnvidia-ml1 \
    && rm -rf /var/lib/apt/lists/*

# Create symlink from libnvidia-ml.so to libnvidia-ml.so.1 if it doesn't already exist
RUN [ ! -e /usr/lib/x86_64-linux-gnu/libnvidia-ml.so ] && \
    ln -s /usr/lib/x86_64-linux-gnu/libnvidia-ml.so.1 /usr/lib/x86_64-linux-gnu/libnvidia-ml.so || true

# Create a non-root user
RUN useradd -m -u 1000 exporter

# Copy the binary from builder
COPY --from=builder /build/target/release/nvidia-gpu-exporter /usr/local/bin/nvidia-gpu-exporter

# Build arguments for customization
ARG WEB_LISTEN_PORT=9445
ARG WEB_TELEMETRY_PATH=/metrics

# Environment variables (set from build args for runtime use)
ENV WEB_LISTEN_PORT=${WEB_LISTEN_PORT}
ENV WEB_TELEMETRY_PATH=${WEB_TELEMETRY_PATH}

# Create wrapper script to expand environment variables
RUN echo '#!/bin/sh\n\
exec /usr/local/bin/nvidia-gpu-exporter \\\n\
  --web-listen-address "0.0.0.0:${WEB_LISTEN_PORT:-9445}" \\\n\
  --web-telemetry-path "${WEB_TELEMETRY_PATH:-/metrics}"' > /usr/local/bin/entrypoint.sh && \
    chmod +x /usr/local/bin/entrypoint.sh

# Switch to non-root user
USER exporter

# Expose the port
EXPOSE ${WEB_LISTEN_PORT}

# Set the entrypoint
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]

