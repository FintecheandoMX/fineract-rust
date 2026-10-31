# Dockerfile for the Fineract Rust Axum provider.
#
# Multi-stage build: compile in a builder stage, run from a slim runtime image.
#
# Usage:
#   docker build -t fineract-rust-provider -f rust/Dockerfile .
#
# Build args:
#   FINERACT_PROFILE — release or dev (default: release)

FROM rust:1.99-slim AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace manifests for dependency caching
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./

# Copy the entire rust workspace
COPY rust/ rust/

# Build the provider (release by default)
ARG FINERACT_PROFILE=release
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --${FINERACT_PROFILE} -p fineract-provider

# ---------------------------------------------------------------
# Runtime stage
# ---------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    wget \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/fineract-provider /app/fineract-provider

EXPOSE 8080

ENV FINERACT_DATABASE_URL=postgresql://postgres:postgres@postgres:5432/fineract_desarrollo
ENV FINERACT_PLATFORM_TENANT_IDENTIFIER=desarrollo
ENV RUST_LOG=fineract_provider=debug,tower_http=debug

CMD ["/app/fineract-provider"]
