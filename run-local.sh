#!/bin/bash
# Helper script to run backend locally from project root

set -e

# Load .env if it exists
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

# Ensure data directories exist
mkdir -p backend/cache/raw backend/cache/parsed backend/data

# Build if needed
if [ ! -f backend/target/release/warsaw_pool_ranking ]; then
    echo "Building backend..."
    cargo build --release --manifest-path backend/Cargo.toml
fi

# Run the command
echo "Running: warsaw_pool_ranking $@"
backend/target/release/warsaw_pool_ranking "$@"
