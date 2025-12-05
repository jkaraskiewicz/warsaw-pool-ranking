# Dockerfile for the Rust backend
FROM rust:latest AS builder

WORKDIR /app

# Install protobuf compiler
RUN apt-get update && apt-get install -y protobuf-compiler && rm -rf /var/lib/apt/lists/*

# Copy Cargo files
COPY backend/Cargo.toml backend/Cargo.lock ./

# Copy build script and proto files
COPY backend/build.rs ./
COPY proto ./proto

# Build a dummy project to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

# Copy the actual source code
COPY backend/src ./src

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
COPY backend/entrypoint.sh ./

# Set environment variables if needed
ENV DATABASE_PATH="warsaw_pool_ranking.db"

# Command to run the application (will be overridden by docker-compose.yml)
# CMD ["./warsaw_pool_ranking", "process"]