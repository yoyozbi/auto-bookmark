# Get started with a build env with Rust nightly
FROM rustlang/rust:nightly-alpine AS builder

# Install system dependencies
RUN apk update && \
    apk add --no-cache bash curl npm libc-dev binaryen

# Install sass globally
RUN npm install -g sass

# Install cargo-leptos
RUN curl --proto '=https' --tlsv1.2 -LsSf https://github.com/leptos-rs/cargo-leptos/releases/download/v0.2.42/cargo-leptos-installer.sh | sh

# Add the WASM target
RUN rustup target add wasm32-unknown-unknown

WORKDIR /work

# Copy dependency files first for layer caching
COPY Cargo.toml Cargo.lock ./

# Create dummy source files to satisfy cargo build for dependencies
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    echo "// dummy lib" > src/lib.rs

# Build dependencies only (this layer will be cached until Cargo.toml/Cargo.lock changes)
RUN cargo build --release --bin auto-bookmark --features ssr
RUN cargo build --release --lib --target wasm32-unknown-unknown

# Remove dummy source files
RUN rm -rf src

# Copy the actual source code and other necessary files
COPY src/ ./src/
COPY style/ ./style/
COPY public/ ./public/

# Build the actual application (only this step runs when source code changes)
RUN cargo leptos build --release -vv

# Runtime stage
FROM rustlang/rust:nightly-alpine AS runner

WORKDIR /app

# Copy built artifacts from builder stage
COPY --from=builder /work/target/release/auto-bookmark /app/
COPY --from=builder /work/target/site /app/site
COPY --from=builder /work/Cargo.toml /app/

# Set environment variables
ENV RUST_LOG="info"
ENV LEPTOS_SITE_ADDR="0.0.0.0:8080"
ENV LEPTOS_SITE_ROOT=./site

EXPOSE 8080

CMD ["/app/auto-bookmark"]
