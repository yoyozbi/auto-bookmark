/// Calibration Page Schema for Printer Calibration
/// 
/// This module provides a calibration test page design to help users measure
/// and correct printing offsets when printing on both sides of the paper.
/// 
/// ## Purpose
/// When printing double-sided documents, some printers may not align the front 
/// (recto) and back (verso) pages perfectly. This calibration page helps users:
/// 1. Print a test page
/// 2. Measure the offset between expected and actual positions
/// 3. Configure these offsets in the application
/// 4. Generate PDFs with corrected margins to compensate for the printer's misalignment
///
/// ## Calibration Process
/// 1. Generate and print the calibration page
/// 2. Hold the page up to a light source to see both sides simultaneously
/// 3. Measure the horizontal (left) and vertical (top) offset between the alignment marks
/// 4. Enter these measurements into the application settings
/// 5. Future PDF generations will apply these offsets to ensure proper alignment
///
/// ## Page Structure
/// The calibration page contains:
/// - Corner alignment marks at each corner of the page
/// - Grid lines every 1cm for precise measurement
/// - Measurement rulers along the top and left edges
/// - Instructions printed on the page
/// - A unique pattern on recto and verso that overlaps when aligned
///
use super::{PageMargins};

const CALIBRATION_HTML: &str = r#"<!DOCTYPE html>
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
        
        /* Grid lines */
        .grid-overlay {
            position: absolute;
            top: 0;
            left: 0;
            width: 100%;
            height: 100%;
            pointer-events: none;
        }
        
        .grid-line-h, .grid-line-v {
            position: absolute;
            background: rgba(0, 0, 0, 0.1);
        }
        
        .grid-line-h {
            width: 100%;
            height: 1px;
        }
        
        .grid-line-v {
            width: 1px;
            height: 100%;
        }
        
        /* Measurement rulers */
        .ruler {
            position: absolute;
            display: flex;
        }
        
        .ruler-top {
            top: 0.5cm;
            left: 2cm;
            right: 2cm;
            height: 0.5cm;
            flex-direction: row;
        }
        
        .ruler-left {
            left: 0.5cm;
            top: 2cm;
            bottom: 2cm;
            width: 0.5cm;
            flex-direction: column;
        }
        
        .ruler-tick {
            flex: 1;
            border-left: 1px solid black;
            position: relative;
            font-size: 8pt;
        }
        
        .ruler-top .ruler-tick {
            border-top: 1px solid black;
            border-left: none;
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
"#;

/// Generates a calibration test page as HTML
/// 
/// Returns HTML content ready to be converted to PDF
pub fn generate_calibration_html() -> String {
    CALIBRATION_HTML.to_string()
}

/// Configuration for printer calibration offsets
/// 
/// These offsets are measured from the calibration page and applied
/// to subsequent PDF generations to compensate for printer misalignment.
#[derive(Clone, Debug)]
pub struct CalibrationOffsets {
    /// Horizontal offset in centimeters (positive = shift right)
    pub horizontal_cm: f64,
    /// Vertical offset in centimeters (positive = shift down)
    pub vertical_cm: f64,
}

impl Default for CalibrationOffsets {
    fn default() -> Self {
        Self {
            horizontal_cm: 0.0,
            vertical_cm: 0.0,
        }
    }
}

impl CalibrationOffsets {
    /// Apply calibration offsets to page margins
    /// 
    /// This adjusts the margins to compensate for printer misalignment.
    /// The offsets are applied to the left and top margins.
    pub fn apply_to_margins(&self, margins: &PageMargins) -> PageMargins {
        PageMargins {
            top: margins.top + self.vertical_cm,
            bottom: margins.bottom,
            left: margins.left + self.horizontal_cm,
            right: margins.right,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calibration_html_generation() {
        let html = generate_calibration_html();
        assert!(html.contains("RECTO"));
        assert!(html.contains("VERSO"));
        assert!(html.contains("Calibration Instructions"));
    }

    #[test]
    fn test_calibration_offsets_default() {
        let offsets = CalibrationOffsets::default();
        assert_eq!(offsets.horizontal_cm, 0.0);
        assert_eq!(offsets.vertical_cm, 0.0);
    }

    #[test]
    fn test_apply_calibration_to_margins() {
        let offsets = CalibrationOffsets {
            horizontal_cm: 0.2,
            vertical_cm: 0.3,
        };
        
        let original_margins = PageMargins {
            top: 0.0,
            bottom: 0.0,
            left: 2.0,
            right: 2.0,
        };
        
        let adjusted = offsets.apply_to_margins(&original_margins);
        
        assert_eq!(adjusted.top, 0.3);
        assert_eq!(adjusted.left, 2.2);
        assert_eq!(adjusted.bottom, 0.0);
        assert_eq!(adjusted.right, 2.0);
    }
}
