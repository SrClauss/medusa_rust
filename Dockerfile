# ── MedusaRust — multi-stage Dockerfile ──────────────────────────────────────
#
# Stage 1: build the Rust binary with release optimisations.
# Stage 2: minimal Debian-slim runtime image (~50 MB total).

# ── Builder ───────────────────────────────────────────────────────────────────
FROM rust:1.82-slim-bookworm AS builder

# Install native build deps (OpenSSL, pkg-config for sqlx/tls).
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Cache dependencies separately from source code.
COPY Cargo.toml Cargo.lock ./
# Create a dummy main.rs so `cargo build` caches deps.
RUN mkdir src && echo 'fn main(){}' > src/main.rs
RUN cargo build --release 2>/dev/null || true
RUN rm -rf src

# Build the real application.
COPY src ./src
COPY migrations ./migrations
RUN touch src/main.rs          # force rebuild
RUN cargo build --release

# ── Runtime ───────────────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled binary and migrations.
COPY --from=builder /build/target/release/medusa_rust ./medusa_rust
COPY --from=builder /build/migrations ./migrations

# Create the local uploads directory (fallback when S3 is not configured).
RUN mkdir -p uploads

# Expose the default Axum port.
EXPOSE 9000

ENV HOST=0.0.0.0 \
    PORT=9000

ENTRYPOINT ["./medusa_rust"]
