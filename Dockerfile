# Use a slim Rust base image for a smaller final image
FROM rust:1.82-slim AS build

# Install necessary dependencies for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /app

# Copy the Cargo.toml file
COPY Cargo.toml ./

# Create a dummy src/main.rs to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --bin groupme-bot
RUN rm -rf src

# Copy the actual source code
COPY src ./src

# Build the application
RUN cargo build --release --bin groupme-bot

# Start runtime stage - use minimal Debian image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy the binary from the build stage
COPY --from=build /app/target/release/groupme-bot ./groupme-bot

# Copy service account JSON file
COPY service-account.json ./service-account.json

# Create a non-root user for security
RUN useradd -r -s /bin/false appuser
RUN chown appuser:appuser /app/groupme-bot
RUN chown appuser:appuser /app/service-account.json
RUN mkdir -p /app/data && chown appuser:appuser /app/data

# Switch to non-root user
USER appuser

# Expose port
EXPOSE 18080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:18080/ || exit 1

# Run the application
CMD ["./groupme-bot"]
