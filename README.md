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

### Environment Variables

#### PDF Generation Method
- `USE_HTML_PDF`: Set to `true` to use HTML/CSS generation instead of Typst (default: `false`)

#### Printer Calibration (HTML/CSS mode only)
- `CALIBRATION_OFFSET_HORIZONTAL`: Horizontal offset in cm to compensate for printer misalignment (default: `0.0`)
- `CALIBRATION_OFFSET_VERTICAL`: Vertical offset in cm to compensate for printer misalignment (default: `0.0`)

#### Legacy Flask Variables (from previous Python version)
- `ADMIN_PASSWORD`: The password to access the server (the username will be admin) you need to use `generate_password_hash` from `werkzeug.security` to generate the hash.
- `PWD_FILE`: The file to read the password from. If this is set, the `PASSWORD` variable will be ignored. The file should contain a `username:password` combo, one combo per line.
- `SECRET_KEY`: The secret key to use for the flask app. This is used to sign the session cookie.
- `SECRET_KEY_FILE`: The file to read the secret key from. If this is set, the `SECRET_KEY` variable will be ignored.
- `ALLOWED_HOSTS`: The allowed hosts for the server. This is used to set the `Access-Control-Allow-Origin` header. If this is not set, the header will default to `*` (This is insecure).

### Running with Docker

```bash
$ docker run -p 3000:3000 \
  -e USE_HTML_PDF=true \
  -e CALIBRATION_OFFSET_HORIZONTAL=0.2 \
  -e CALIBRATION_OFFSET_VERTICAL=0.1 \
  -d --name auto-bookmark auto-bookmark
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
