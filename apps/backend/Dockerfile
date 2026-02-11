# syntax=docker/dockerfile:1

# Stage 1: Planner - Create dependency list
FROM rust:1.93-bookworm AS planner

WORKDIR /app

# Install cargo-chef for dependency caching
RUN cargo install cargo-chef --locked

# Copy complete crate structure for cargo-chef
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Builder - Build dependencies
FROM rust:1.93-bookworm AS builder

WORKDIR /app

# Install cargo-chef
RUN cargo install cargo-chef --locked

# Copy the recipe from planner
COPY --from=planner /app/recipe.json recipe.json

# Build dependencies (this layer is cached)
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo chef cook --release --recipe-path recipe.json

# Copy source code
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build application
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release && \
    cp /app/target/release/backend /app/backend

# Stage 3: Runtime - Slim production image
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Install runtime dependencies (including SQLite)
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r appuser && useradd -r -g appuser appuser

# Copy binary from builder
COPY --from=builder --chown=appuser:appuser /app/backend /app/backend

# Create default data directory
RUN mkdir -p /run/media && chown appuser:appuser /run/media

USER appuser

EXPOSE 3000

ENV RUST_LOG=info
ENV DATA_DIR=/run/media

CMD ["/app/backend"]
