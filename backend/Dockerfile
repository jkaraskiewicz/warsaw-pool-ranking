# Dockerfile for the Rust backend
FROM rust:latest AS builder

WORKDIR /app

# Copy Cargo files
COPY Cargo.toml ./

# Build a dummy project to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy the actual source code
COPY src ./src

# Build the Rust application
RUN cargo build --release

# --- Final Stage ---
FROM debian:bookworm-slim

WORKDIR /app

# Install necessary runtime dependencies (e.g., SQLite)
RUN apt-get update && apt-get install -y --no-install-recommends \
    libsqlite3-0 \
    sqlite3 \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the built binary from the builder stage
COPY --from=builder /app/target/release/warsaw_pool_ranking ./

# Copy entrypoint script
COPY entrypoint.sh ./

# Set environment variables if needed
ENV DATABASE_PATH="warsaw_pool_ranking.db"

# Command to run the application (will be overridden by docker-compose.yml)
