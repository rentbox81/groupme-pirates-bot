# Use a slim Rust base image for a smaller final image
FROM rust:1.78.0-slim as build

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
RUN cargo build --release
RUN rm -rf src

# Copy the actual source code
COPY src ./src

# Build the release binary
RUN cargo build --release

# Use a minimal base image for the final container
FROM debian:bookworm-slim

# Install SSL certificates and CA certificates
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /app

# Copy the built binary from the build stage
COPY --from=build /app/target/release/groupme-bot ./groupme-bot

# Create a non-root user
RUN useradd -r -s /bin/false appuser
RUN chown appuser:appuser /app/groupme-bot

# Switch to non-root user
USER appuser

# Expose the port the application listens on
EXPOSE 18080

# Set environment variables
ENV RUST_LOG=info
ENV PORT=18080

# The command to run the application
CMD ["./groupme-bot"]
