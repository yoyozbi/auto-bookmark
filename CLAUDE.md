# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Auto-bookmark is a Rust web application built with Leptos framework that processes PDF files to generate bookmarks. It uses Typst 0.14+ direct PDF embedding to create bookmark layouts without intermediate image extraction, generating PDFs formatted as bookmarks (5cm x 15cm).

## Development Commands

### Building and Running
```bash
# Build with full PDF processing support
cargo build --features ssr

# Run development server with hot reload
cargo leptos watch

# Run production server
cargo run --features ssr

# Build for production
cargo leptos build --release
```

### Testing
```bash
# Run tests with default features
cargo test

# Run tests with SSR features (server-side rendering)
cargo test --features ssr

# Run all tests including documentation tests
cargo test --doc --features ssr

# Run tests with nextest (recommended)
cargo nextest run --features ssr

# Run tests with CI profile (retries enabled)
cargo nextest run --features ssr --profile ci
```

### Code Quality
```bash
# Check code formatting
cargo fmt --all -- --check

# Run linting
cargo clippy --all-targets --all-features -- -D warnings

# Fix formatting
cargo fmt --all
```

## Architecture Overview

### Feature System
The codebase uses Rust feature flags to separate client and server functionality:

- **Default**: Client-side WASM functionality
- **ssr**: Server-side rendering with PDF processing capabilities
- **hydrate**: Client-side hydration support

### Core Components

**Application Structure:**
- `src/app.rs` - Main Leptos app component and state management
- `src/main.rs` - Server entry point with Axum integration
- `src/lib.rs` - Library entry point with WASM hydration

**PDF Processing Pipeline (`src/generation/`):**
- `extract_pdf_pages.rs` - PDF page counting and file management
- `generate_pdf.rs` - PDF generation with direct PDF page embedding using Typst 0.14+
- `validate_margins.rs` - Bookmark size validation

**Web Interface (`src/pages/`):**
- `home.rs` - Main landing page
- `file_upload.rs` - File upload interface

**Request Management:**
- `src/upload_route.rs` - File upload API endpoints
- `src/utils/upload_workflow.rs` - Upload processing workflow
- `GenerationRequest` struct manages PDF processing state

### Data Flow
1. User uploads PDF files via web interface
2. PDF page count is validated (must be even for recto-verso pairs)
3. PDFs are copied to working directory with UUID naming
4. Typst directly embeds PDF pages using `image("file.pdf", page: N)` syntax
5. Generated bookmark PDF is served back to user
6. Temporary PDF files are cleaned up automatically

### Key Dependencies
- **Leptos**: Full-stack web framework with SSR
- **Axum**: HTTP server framework
- **Typst 0.14+**: PDF generation with direct PDF page embedding
- **typst-as-lib 0.15**: Rust integration for Typst compilation
- **UUID**: Request tracking and file management

## Testing Configuration

Uses `cargo-nextest` for enhanced test execution with configuration in `.config/nextest.toml`:
- CI profile includes retries and timeout handling
- JUnit XML output for GitHub Actions integration
- Optimized for parallel execution

## Development Notes

- PDF processing requires `ssr` feature to be enabled
- Without `ssr` feature, fallback implementations are used
- **New in this version**: Uses Typst 0.14 direct PDF embedding instead of image extraction
- No intermediate image files are generated - PDFs are embedded directly
- Bookmark dimensions are fixed at approximately 5cm x 15cm
- Temporary PDF files are stored with UUID-based naming for cleanup
- Page count validation ensures even numbers for proper recto-verso pairing