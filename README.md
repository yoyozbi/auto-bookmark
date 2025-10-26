# Auto-bookmark
![CI](https://github.com/yoyozbi/auto-bookmark/worflows/CI/badge.svg)

A Rust web application built with Leptos that generates bookmark PDFs from uploaded PDF files.
The bookmarks are formatted to approximately 5cm x 15cm dimensions.

**New in this version**: Uses Typst 0.14+ direct PDF embedding feature instead of intermediate image extraction for improved performance and quality.

# Usage
Use the docker image to run the server. The server will be available at http://localhost:5000 (The default username is `admin`)
```bash
$ docker run -p 5000:5000 -e ADMIN_PASSWORD=<your-hashed-password> -e SECRET_KEY=<random-key> -e ALLOWED_HOSTS=* -d --name auto-bookmark auto-bookmark
```
Env variables:
- `ADMIN_PASSWORD`: The password to access the server (the username will be admin) you need to use `generate_password_hash` from `werkzeug.security` to generate the hash.
- `PWD_FILE`: The file to read the password from. If this is set, the `PASSWORD` variable will be ignored. The file should contain a `username:password` combo, one combo per line.
- `SECRET_KEY`: The secret key to use for the flask app. This is used to sign the session cookie.
- `SECRET_KEY_FILE`: The file to read the secret key from. If this is set, the `SECRET_KEY` variable will be ignored.
- `ALLOWED_HOSTS`: The allowed hosts for the server. This is used to set the `Access-Control-Allow-Origin` header. If this is not set, the header will default to `*` (This is insecure).

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
