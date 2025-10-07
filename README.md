# Auto-bookmark

A Rust application (previously Python) to generate PDFs from bookmark images using either Typst or HTML/CSS rendering.

The bookmarks used are approximately 5cm x 15cm in size. Use at your own risk if your bookmarks differ.

This application provides a web interface for uploading PDF files and generating formatted bookmark layouts suitable for double-sided printing.

## Features

- **Dual PDF Generation Methods**: Choose between Typst (default) or HTML/CSS rendering
- **Printer Calibration Support**: Compensate for printer misalignment when printing double-sided
- **Web Interface**: Upload PDFs and download generated bookmark layouts
- **Flexible Layout**: Automatically arranges bookmarks in an optimized grid for printing

## PDF Generation Methods

### Typst-based Generation (Default)
Uses the Typst typesetting system for high-quality PDF generation. Fast and efficient.

### HTML/CSS-based Generation (PoC)
Alternative rendering method using HTML/CSS, converted to PDF via wkhtmltopdf. This method provides:
- More familiar styling with CSS
- Easier customization for web developers
- Better integration with printer calibration features
- Simpler debugging of layout issues

**Note**: HTML/CSS generation requires `wkhtmltopdf` to be installed on the system.

## Usage

### Building

#### Default Build (Typst-based PDF generation)
```bash
cargo build --features ssr
cargo run --features ssr
```

#### HTML/CSS-based PDF generation
```bash
cargo build --features ssr,html-pdf
cargo run --features ssr,html-pdf
```

### Environment Variables

#### Printer Calibration (UI-based configuration recommended)
Calibration offsets can be set directly in the web UI. For advanced use cases, environment variables are also available:
- `CALIBRATION_OFFSET_HORIZONTAL`: Horizontal offset in cm to compensate for printer misalignment (default: `0.0`)
- `CALIBRATION_OFFSET_VERTICAL`: Vertical offset in cm to compensate for printer misalignment (default: `0.0`)

**Note**: It's recommended to use the web UI for calibration instead of environment variables.

#### Legacy Flask Variables (from previous Python version)
- `ADMIN_PASSWORD`: The password to access the server (the username will be admin) you need to use `generate_password_hash` from `werkzeug.security` to generate the hash.
- `PWD_FILE`: The file to read the password from. If this is set, the `PASSWORD` variable will be ignored. The file should contain a `username:password` combo, one combo per line.
- `SECRET_KEY`: The secret key to use for the flask app. This is used to sign the session cookie.
- `SECRET_KEY_FILE`: The file to read the secret key from. If this is set, the `SECRET_KEY` variable will be ignored.
- `ALLOWED_HOSTS`: The allowed hosts for the server. This is used to set the `Access-Control-Allow-Origin` header. If this is not set, the header will default to `*` (This is insecure).

### Running with Docker

For HTML/CSS generation, build with the feature flag:
```bash
$ docker build --build-arg CARGO_FEATURES="ssr,html-pdf" -t auto-bookmark .
$ docker run -p 3000:3000 -d --name auto-bookmark auto-bookmark
```

For default Typst generation:
```bash
$ docker build --build-arg CARGO_FEATURES="ssr" -t auto-bookmark .
$ docker run -p 3000:3000 -d --name auto-bookmark auto-bookmark
```

### Printer Calibration

If you notice misalignment when printing double-sided bookmarks, use the calibration feature:

1. Generate and print the calibration page (see `CALIBRATION_SCHEMA.md` for details)
2. Measure the offset between recto and verso sides
3. Set the calibration environment variables
4. Regenerate your bookmarks with the corrections applied

See [CALIBRATION_SCHEMA.md](CALIBRATION_SCHEMA.md) for detailed calibration instructions.

## Building from Source

```bash
$ cargo build --release --features ssr
$ cargo run --features ssr
```

## Development

```bash
$ cargo test --features ssr
```
