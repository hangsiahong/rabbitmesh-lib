# Multi-stage Dockerfile for Rust microservice
FROM rustlang/rust:nightly as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set work directory
WORKDIR /app

# Copy workspace files
COPY . .

# Build the auth-service binary
RUN cargo build --release --bin auth-service

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false appuser

# Set work directory
WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/auth-service /app/auth-service

# Copy configuration files if needed
COPY --from=builder /app/auth-service/config/ /app/config/

# Change ownership
RUN chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Expose port (if needed for health checks)
EXPOSE 8081

# Set default command
CMD ["./auth-service"]