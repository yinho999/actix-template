# Leveraging the pre-built Docker images with 
# cargo-chef and the Rust toolchain
FROM lukemathwalker/cargo-chef:latest-rust-1.59.0 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
# Sqlx to offline mode 
ENV SQLX_OFFLINE=true

RUN cargo build --release --bin {{project-name}}

# We do not need the Rust toolchain to run the binary!
FROM debian:bullseye-slim AS runtime
WORKDIR /app
# Install necessary dependencies for running binary
RUN apt-get update -y \
    # openssl for dependencies and ca-certificates for tls/https certificates
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/app {{project-name}}
# Copy configration folder 
COPY configuration configuration
# Change environment variables here for configuration
ENV APP_ENVIRONMENT=production

ENTRYPOINT ["/{{project-name}}"]