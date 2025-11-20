# Multi-stage build for presidio-analyzer-api
# Stage 1: Build
FROM rust:1.75-alpine as builder

# Install build dependencies
RUN apk add --no-cache musl-dev

# Create app directory
WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY presidio-common ./presidio-common
COPY presidio-analyzer ./presidio-analyzer

# Build the binary
RUN cargo build --release --package presidio-analyzer --bin presidio-analyzer-api --features api

# Stage 2: Runtime
FROM alpine:latest

# Install runtime dependencies
RUN apk add --no-cache ca-certificates libgcc

# Create non-root user
RUN addgroup -g 1000 presidio && \
    adduser -D -u 1000 -G presidio presidio

# Copy binary from builder
COPY --from=builder /app/target/release/presidio-analyzer-api /usr/local/bin/presidio-analyzer-api

# Set user
USER presidio

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:3000/health || exit 1

# Run the service
ENTRYPOINT ["presidio-analyzer-api"]
