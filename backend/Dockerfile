# Dockerfile for the Rust backend
FROM rust:1.73-slim-buster AS builder

WORKDIR /app

# Copy Cargo.toml and Cargo.lock to leverage Docker cache
COPY Cargo.toml Cargo.lock ./

# Build a dummy project to cache dependencies
RUN mkdir src && echo "fn main() {}
" > src/main.rs
RUN cargo build --release

# Copy the actual source code
COPY src ./src

# Build the Rust application
RUN cargo build --release

# --- Final Stage ---
FROM debian:buster-slim

WORKDIR /app

# Install necessary runtime dependencies (e.g., SQLite)
RUN apt-get update && apt-get install -y --no-install-recommends \
    libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/*

# Copy the built binary from the builder stage
COPY --from=builder /app/target/release/warsaw_pool_ranking ./

# Copy the database schema and initial data (if needed at runtime)
COPY database/schema.sql ./database/ # Assuming database schema is needed at runtime

# Set environment variables if needed
ENV DATABASE_PATH="warsaw_pool_ranking.db"

# Command to run the application
CMD ["./warsaw_pool_ranking", "process"]
