# syntax=docker/dockerfile:1

# Stage 1: Planner - Create dependency list
FROM rust:1.85-bookworm AS planner

WORKDIR /app

# Install cargo-chef for dependency caching
RUN cargo install cargo-chef --locked

# Copy complete crate structure for cargo-chef
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Builder - Build dependencies
FROM rust:1.85-bookworm AS builder

WORKDIR /app

# Install cargo-chef
RUN cargo install cargo-chef --locked && cargo install sqlx-cli --no-default-features --features postgres --locked

# Copy the recipe from planner
COPY --from=planner /app/recipe.json recipe.json

# Build dependencies (this layer is cached)
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo chef cook --release --recipe-path recipe.json

# Copy source code
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

# Build application
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release && \
    cp /app/target/release/backend /app/backend

# Stage 3: Runtime - Slim production image
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r appuser && useradd -r -g appuser appuser

# Copy binary and migrations from builder
COPY --from=builder --chown=appuser:appuser /app/backend /app/backend
COPY --from=builder --chown=appuser:appuser /app/migrations /app/migrations

# Add migration runner
COPY --from=builder --chown=appuser:appuser /usr/local/cargo/bin/sqlx /usr/local/bin/sqlx

USER appuser

EXPOSE 3000

ENV RUST_LOG=info

CMD ["/bin/sh", "-c", "sqlx migrate run --source /app/migrations && /app/backend"]
