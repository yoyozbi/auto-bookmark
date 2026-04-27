# Auto-bookmark
![CI](https://github.com/yoyozbi/auto-bookmark/worflows/CI/badge.svg)

A Rust web application built with Leptos that generates bookmark PDFs from uploaded PDF files.
The bookmarks are formatted to approximately 5cm x 15cm dimensions.

**New in this version**: Uses Typst 0.14+ direct PDF embedding feature instead of intermediate image extraction for improved performance and quality.

# Usage
Use the docker image to run the server. The server will be available at http://localhost:8080.

```bash
docker run -p 8080:8080 -d --name auto-bookmark ghcr.io/yoyozbi/auto-bookmark
```

Environment variables:
- `LEPTOS_SITE_ADDR`: Address and port the server listens on (default: `0.0.0.0:8080`)
- `RUST_LOG`: Log level (default: `info`)

## Development

### CI/CD Pipeline

This project uses GitHub Actions for continuous integration and testing. The CI pipeline automatically:

- **Code Quality Checks:**
  - Rust formatting verification (`cargo fmt`)
  - Linting with Clippy (`cargo clippy`)
  - Build verification for both default and SSR features

- **Testing:**
  - Unit tests with default features
  - Unit tests with SSR features enabled
  - Documentation tests
  - Test results are published as JUnit XML reports

- **Test Reporting:**
  - Detailed test summaries in GitHub Actions
  - Test artifacts are uploaded and retained for 30 days
  - Failed tests are clearly highlighted with error details

The CI runs on:
- All pull requests to `main` branch
- Direct pushes to `main` branch

### Running Tests Locally

```bash
# Run tests with default features
cargo test

# Run tests with SSR features (server-side rendering)
cargo test --features ssr

# Run all tests including documentation tests
cargo test --doc --features ssr

# Check code formatting
cargo fmt --all -- --check

# Run linting
cargo clippy --all-targets --all-features -- -D warnings
```

### Test Configuration

The project uses `cargo-nextest` for enhanced test execution with:
- Retry logic for flaky tests
- Detailed failure reporting
- JUnit XML output for CI integration
- Optimized parallel test execution

Configuration is available in `.config/nextest.toml` for both local development and CI environments.
