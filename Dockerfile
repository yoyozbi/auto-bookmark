# Multi-stage build using pre-built binaries from GitHub Actions
FROM alpine:3.19 AS runner

WORKDIR /app

# Install runtime dependencies
RUN apk update && \
    apk add --no-cache ca-certificates

# Get the target architecture for the binary
ARG TARGETPLATFORM
RUN case "$TARGETPLATFORM" in \
        "linux/amd64") export ARCH=amd64 ;; \
        "linux/arm64") export ARCH=arm64 ;; \
        *) echo "Unsupported platform: $TARGETPLATFORM" && exit 1 ;; \
    esac && \
    echo "ARCH=$ARCH" >> /etc/environment

# Copy pre-built artifacts based on architecture
ARG TARGETPLATFORM
COPY dist/linux/amd64/auto-bookmark /tmp/auto-bookmark-amd64
COPY dist/linux/arm64/auto-bookmark /tmp/auto-bookmark-arm64
COPY dist/site ./site
COPY Cargo.toml ./

# Select the correct binary for the target architecture and clean up
RUN case "$TARGETPLATFORM" in \
        "linux/amd64") \
            mv /tmp/auto-bookmark-amd64 /app/auto-bookmark && \
            rm -f /tmp/auto-bookmark-arm64 ;; \
        "linux/arm64") \
            mv /tmp/auto-bookmark-arm64 /app/auto-bookmark && \
            rm -f /tmp/auto-bookmark-amd64 ;; \
    esac && \
    chmod +x /app/auto-bookmark

# Set environment variables
ENV RUST_LOG="info"
ENV LEPTOS_SITE_ADDR="0.0.0.0:8080"
ENV LEPTOS_SITE_ROOT=./site
ENV LEPTOS_OUTPUT_NAME="auto-bookmark"
ENV LEPTOS_SITE_PKG_DIR="pkg"

# Create a non-root user for security
RUN addgroup -g 1001 -S appgroup && \
    adduser -S appuser -u 1001 -G appgroup

# Change ownership of the app directory
RUN chown -R appuser:appgroup /app

USER appuser

EXPOSE 8080

CMD ["/app/auto-bookmark"]
