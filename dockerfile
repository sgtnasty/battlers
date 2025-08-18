# Stage 1: Build
FROM ubuntu:22.04 AS builder

# Install Rust and dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    curl ca-certificates build-essential && \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
    . $HOME/.cargo/env && \
    rustup default stable

WORKDIR /app
COPY . .

# Build in release mode
RUN . $HOME/.cargo/env && cargo build --release

# Stage 2: Runtime
FROM ubuntu:22.04

# Install runtime dependencies (if needed, e.g., OpenSSL)
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    libssl3 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled binary from the builder
COPY --from=builder /app/target/release/battlers /app/battlers

# Run as non-root user (security best practice)
RUN useradd -m appuser && chown -R appuser:appuser /app
USER appuser

# Entrypoint
CMD ["/app/battlers"]