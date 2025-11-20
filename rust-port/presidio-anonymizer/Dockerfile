# Multi-stage build for presidio-anonymizer-api
# Stage 1: Build
FROM rust:1.75-alpine as builder

# Install build dependencies
RUN apk add --no-cache musl-dev

# Create app directory
WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY presidio-common ./presidio-common
COPY presidio-anonymizer ./presidio-anonymizer

# Build the binary
RUN cargo build --release --package presidio-anonymizer --bin presidio-anonymizer-api --features api

# Stage 2: Runtime
FROM alpine:latest

# Install runtime dependencies
RUN apk add --no-cache ca-certificates libgcc

# Create non-root user
RUN addgroup -g 1000 presidio && \
    adduser -D -u 1000 -G presidio presidio

# Copy binary from builder
COPY --from=builder /app/target/release/presidio-anonymizer-api /usr/local/bin/presidio-anonymizer-api

# Set user
USER presidio

# Expose port
EXPOSE 3001

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:3001/health || exit 1

# Run the service
ENTRYPOINT ["presidio-anonymizer-api"]
