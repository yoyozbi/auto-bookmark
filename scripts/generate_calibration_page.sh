#!/usr/bin/env bash

# Generate Calibration Page Script
# This script generates a printer calibration page that users can print
# to measure and correct their printer's double-sided alignment.

set -e

OUTPUT_FILE="${1:-calibration_page.html}"
PDF_OUTPUT="${OUTPUT_FILE%.html}.pdf"

echo "Generating calibration page..."

# Generate the HTML content
cat > "$OUTPUT_FILE" << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Printer Calibration Page</title>
    <style>
        @page {
            size: A4;
            margin: 0;
        }
        
        body {
            margin: 0;
            padding: 0;
            font-family: Arial, sans-serif;
            position: relative;
        }
        
        .page {
            width: 21cm;
            height: 29.7cm;
            position: relative;
            page-break-after: always;
        }
        
        .page:last-child {
            page-break-after: auto;
        }
        
        /* Corner alignment marks */
        .corner-mark {
            position: absolute;
            width: 2cm;
            height: 2cm;
            border: 2px solid black;
        }
        
        .corner-mark::before {
            content: '';
            position: absolute;
            top: 50%;
            left: 50%;
            width: 0.5cm;
            height: 0.5cm;
            background: black;
            transform: translate(-50%, -50%);
        }
        
        .corner-top-left {
            top: 1cm;
            left: 1cm;
            border-right-color: transparent;
            border-bottom-color: transparent;
        }
        
        .corner-top-right {
            top: 1cm;
            right: 1cm;
            border-left-color: transparent;
            border-bottom-color: transparent;
        }
        
        .corner-bottom-left {
            bottom: 1cm;
            left: 1cm;
            border-right-color: transparent;
            border-top-color: transparent;
        }
        
        .corner-bottom-right {
            bottom: 1cm;
            right: 1cm;
            border-left-color: transparent;
            border-top-color: transparent;
        }
        
        /* Center crosshair */
        .crosshair {
            position: absolute;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
        }
        
        .crosshair::before,
        .crosshair::after {
            content: '';
            position: absolute;
            background: black;
        }
        
        .crosshair::before {
            width: 4cm;
            height: 2px;
            left: -2cm;
            top: 0;
        }
        
        .crosshair::after {
            width: 2px;
            height: 4cm;
            left: 0;
            top: -2cm;
        }
        
        /* Instructions */
        .instructions {
            position: absolute;
            top: 5cm;
            left: 5cm;
            right: 5cm;
            background: rgba(255, 255, 255, 0.9);
            padding: 1cm;
            border: 1px solid black;
            font-size: 10pt;
        }
        
        .instructions h2 {
            margin-top: 0;
            font-size: 14pt;
        }
        
        .instructions ol {
            margin: 0.5cm 0;
            padding-left: 1cm;
        }
        
        .instructions li {
            margin: 0.2cm 0;
        }
        
        /* Page identifier */
        .page-id {
            position: absolute;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
            font-size: 48pt;
            font-weight: bold;
            color: rgba(0, 0, 0, 0.1);
        }
    </style>
</head>
<body>
    <!-- RECTO PAGE -->
    <div class="page">
        <div class="corner-mark corner-top-left"></div>
        <div class="corner-mark corner-top-right"></div>
        <div class="corner-mark corner-bottom-left"></div>
        <div class="corner-mark corner-bottom-right"></div>
        
        <div class="crosshair"></div>
        <div class="page-id">RECTO</div>
        
        <div class="instructions">
            <h2>Printer Calibration Instructions</h2>
            <ol>
                <li>Print this page double-sided (flip on long edge)</li>
                <li>Hold the page up to a light source</li>
                <li>Observe the alignment between RECTO and VERSO sides</li>
                <li>Measure the offset at the center crosshair:
                    <ul>
                        <li>Horizontal offset (left/right displacement in mm)</li>
                        <li>Vertical offset (up/down displacement in mm)</li>
                    </ul>
                </li>
                <li>Enter these measurements in the application settings</li>
            </ol>
            <p><strong>Note:</strong> The corner marks and center crosshair should align perfectly on both sides when held up to light.</p>
        </div>
    </div>
    
    <!-- VERSO PAGE -->
    <div class="page">
        <div class="corner-mark corner-top-left"></div>
        <div class="corner-mark corner-top-right"></div>
        <div class="corner-mark corner-bottom-left"></div>
        <div class="corner-mark corner-bottom-right"></div>
        
        <div class="crosshair"></div>
        <div class="page-id">VERSO</div>
    </div>
</body>
</html>
EOF

echo "✓ Calibration HTML generated: $OUTPUT_FILE"

# Try to convert to PDF if wkhtmltopdf is available
if command -v wkhtmltopdf &> /dev/null; then
    echo "Converting to PDF..."
    wkhtmltopdf --page-size A4 --enable-local-file-access "$OUTPUT_FILE" "$PDF_OUTPUT"
    echo "✓ Calibration PDF generated: $PDF_OUTPUT"
    echo ""
    echo "Next steps:"
    echo "1. Print $PDF_OUTPUT double-sided (flip on long edge)"
    echo "2. Hold the page up to a light"
    echo "3. Measure the offset between RECTO and VERSO crosshairs"
    echo "4. Set environment variables:"
    echo "   export CALIBRATION_OFFSET_HORIZONTAL=<offset_in_cm>"
    echo "   export CALIBRATION_OFFSET_VERTICAL=<offset_in_cm>"
    echo "   export USE_HTML_PDF=true"
else
    echo "⚠ wkhtmltopdf not found. HTML file generated only."
    echo "Install wkhtmltopdf to generate PDF:"
    echo "  - Ubuntu/Debian: sudo apt-get install wkhtmltopdf"
    echo "  - macOS: brew install wkhtmltopdf"
    echo "  - Or open $OUTPUT_FILE in a browser and print to PDF"
fi
