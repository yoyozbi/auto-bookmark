# Printer Calibration Schema

## Overview

The calibration page is designed to help users measure and correct printing offsets when printing double-sided documents. Some printers don't perfectly align the front (recto) and back (verso) pages, causing misalignment when folding or cutting.

## Calibration Page Layout

```
┌─────────────────────────────────────────────────┐
│  ┌─┐                               ┌─┐          │  <- Corner alignment marks
│  └                                   ┘          │
│                                                 │
│           CALIBRATION INSTRUCTIONS              │
│        (printed in semi-transparent box)        │
│                                                 │
│                     ┼                           │  <- Center crosshair
│                                                 │     (critical alignment point)
│                                                 │
│                   RECTO                         │  <- Page identifier
│                 (or VERSO)                      │     (watermark style)
│                                                 │
│  ┌                                   ┌          │
│  └─┘                               └─┘          │  <- Corner alignment marks
└─────────────────────────────────────────────────┘
```

## Key Features

### 1. Corner Alignment Marks
- Located 1cm from each corner
- 2cm x 2cm L-shaped marks with filled center squares
- Used to check overall page alignment

### 2. Center Crosshair
- 4cm horizontal and vertical lines intersecting at the exact page center
- Primary measurement point for calibration
- Should align perfectly when page is held up to light

### 3. Page Identifier
- Large "RECTO" or "VERSO" text
- Semi-transparent (10% opacity)
- Helps identify which side you're looking at

### 4. Instructions Box
- Located in the upper portion of the recto page
- Contains step-by-step calibration instructions
- Semi-transparent background (90% opacity)

## Measurement Process

### Step 1: Print the Calibration Page
- Print double-sided (flip on long edge)
- Use the same printer settings you'll use for actual bookmarks
- Use the same paper type

### Step 2: Align and Measure
```
Hold page up to light source:

    RECTO crosshair                   VERSO crosshair
         │                                 │
         ├─────────────────────────────────┤
         │                                 │
         │    ← measure horizontal offset→ │
         │                                 │
    ─────┼─────                       ─────┼─────
         │                                 │
         ↓                                 ↑
    measure                           
   vertical                          
    offset                            
```

### Step 3: Record Measurements
- Horizontal offset (in mm): Distance between the vertical lines
  - Positive value: verso is shifted RIGHT relative to recto
  - Negative value: verso is shifted LEFT relative to recto
- Vertical offset (in mm): Distance between the horizontal lines
  - Positive value: verso is shifted DOWN relative to recto
  - Negative value: verso is shifted UP relative to recto

### Step 4: Apply Calibration
Convert measurements from mm to cm and set environment variables:
```bash
export CALIBRATION_OFFSET_HORIZONTAL=0.2  # 2mm right offset
export CALIBRATION_OFFSET_VERTICAL=0.1    # 1mm down offset
export USE_HTML_PDF=true
```

Or configure in the application settings (if available).

## Example Scenarios

### Scenario 1: Perfect Alignment
```
Measurements: 
- Horizontal: 0mm
- Vertical: 0mm

Action: No calibration needed!
```

### Scenario 2: Verso shifted right and down
```
Measurements:
- Horizontal: +3mm (verso is 3mm to the right)
- Vertical: +2mm (verso is 2mm down)

Settings:
CALIBRATION_OFFSET_HORIZONTAL=0.3
CALIBRATION_OFFSET_VERTICAL=0.2
```

### Scenario 3: Verso shifted left and up
```
Measurements:
- Horizontal: -2mm (verso is 2mm to the left)
- Vertical: -1mm (verso is 1mm up)

Settings:
CALIBRATION_OFFSET_HORIZONTAL=-0.2
CALIBRATION_OFFSET_VERTICAL=-0.1
```

## Technical Details

### Page Specifications
- Size: A4 (21cm x 29.7cm)
- Margins: 0cm (full bleed)
- Orientation: Portrait
- Color: Black on white

### Measurement Precision
- Corner marks: 2cm x 2cm
- Center crosshair: 4cm arms
- Recommended measurement tool: Ruler or caliper with mm precision
- Typical printer offset range: 0-5mm

## Implementation Notes

The calibration page is generated as HTML/CSS and can be exported to PDF. The HTML template includes:
- Responsive CSS for precise positioning
- Print-specific styles (@page rules)
- High contrast for easy visibility
- Page break control for double-sided printing

The calibration offsets are applied by adjusting the page margins in the PDF generation process, compensating for the printer's mechanical limitations.

## Generating the Calibration Page

```rust
use auto_bookmark::generation::calibration::generate_calibration_html;

let html = generate_calibration_html();
// Convert to PDF using your preferred method (wkhtmltopdf, headless browser, etc.)
```

## Future Enhancements

1. **Interactive Web Tool**: Allow users to upload a scanned calibration page and automatically detect offsets
2. **Per-Printer Profiles**: Save calibration settings for different printers
3. **Advanced Calibration**: Rotation and scaling corrections in addition to offset
4. **Visual Preview**: Show how the calibration will affect the final output before printing
