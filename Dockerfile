# ── Stage 1: builder ──────────────────────────────────────────────────────────
FROM rust:1.82-slim AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Cache dependency compilation by building a stub binary first.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs
RUN cargo build --release 2>/dev/null || true
RUN rm -rf src \
    target/release/phrase-board \
    target/release/deps/phrase_board*

# Build real sources.
COPY src ./src
COPY migrations ./migrations

# SQLX_OFFLINE=true skips live DB query verification at compile time.
# Run `cargo sqlx prepare` locally and commit .sqlx/ to enable this.
ENV SQLX_OFFLINE=true
RUN cargo build --release

# ── Stage 2: runtime ──────────────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/phrase-board /app/phrase-board
COPY --from=builder /app/migrations /app/migrations

# Frontend static files — copied from the build context at docker build time.
COPY static /app/static

# Environment variable defaults (override at runtime)
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=8080
ENV STATIC_DIR=/app/static
ENV ENABLE_DOCS=true
ENV DELETE_TAP_COUNT=3
ENV DEFAULT_CANVAS_WIDTH=1280
ENV DEFAULT_CANVAS_HEIGHT=800

EXPOSE 8080

CMD ["/app/phrase-board"]
