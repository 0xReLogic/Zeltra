# Zeltra Backend - Multi-stage Dockerfile
# Optimized for production with minimal image size

# ============================================================
# Stage 1: Build
# ============================================================
FROM rust:1.92-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconfig

WORKDIR /app

# Copy manifests first for better caching
COPY Cargo.toml Cargo.lock ./
COPY bins/server/Cargo.toml bins/server/
COPY bins/migrator/Cargo.toml bins/migrator/
COPY crates/core/Cargo.toml crates/core/
COPY crates/db/Cargo.toml crates/db/
COPY crates/api/Cargo.toml crates/api/

# Create dummy files to build dependencies
RUN mkdir -p bins/server/src bins/migrator/src crates/core/src crates/db/src crates/api/src && \
    echo "fn main() {}" > bins/server/src/main.rs && \
    echo "fn main() {}" > bins/migrator/src/main.rs && \
    echo "" > crates/core/src/lib.rs && \
    echo "" > crates/db/src/lib.rs && \
    echo "" > crates/api/src/lib.rs

# Build dependencies only (cached layer)
RUN cargo build --release --bin server && \
    rm -rf bins crates

# Copy actual source code
COPY bins bins
COPY crates crates
COPY config config

# Touch files to invalidate cache and rebuild
RUN touch bins/server/src/main.rs && \
    touch bins/migrator/src/main.rs

# Build the actual application
RUN cargo build --release --bin server --bin migrator

# ============================================================
# Stage 2: Runtime
# ============================================================
FROM alpine:3.23 AS runtime

# Install runtime dependencies
RUN apk add --no-cache ca-certificates libgcc

# Create non-root user
RUN addgroup -S zeltra && adduser -S zeltra -G zeltra

WORKDIR /app

# Copy binaries from builder
COPY --from=builder /app/target/release/server /app/server
COPY --from=builder /app/target/release/migrator /app/migrator
COPY --from=builder /app/config /app/config

# Set ownership
RUN chown -R zeltra:zeltra /app

USER zeltra

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8080/health || exit 1

# Default command
CMD ["./server"]
