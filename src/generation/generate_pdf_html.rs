use std::fs;
use std::io::Write;
use std::process::Command;

use super::RectoVersoImagePair;
use super::{PageMargins, GridConfig};
use super::calibration::CalibrationOffsets;

const HTML_TEMPLATE: &str = r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Auto Bookmark</title>
    <style>
        @page {{
            size: A4;
            margin-top: {top}cm;
            margin-bottom: {bottom}cm;
            margin-left: {left}cm;
            margin-right: {right}cm;
        }}
        
        body {{
            margin: 0;
            padding: 0;
            font-family: Arial, sans-serif;
        }}
        
        .page {{
            page-break-after: always;
            width: 100%;
            height: 100%;
        }}
        
        .page:last-child {{
            page-break-after: auto;
        }}
        
        .grid {{
            display: grid;
            grid-template-columns: repeat(3, auto);
            grid-template-rows: auto auto;
            column-gap: {column_gutter}cm;
            row-gap: {row_gutter}cm;
            justify-content: center;
            align-items: center;
            width: 100%;
        }}
        
        .image-cell {{
            text-align: center;
        }}
        
        .image-cell img {{
            width: {image_width}cm;
            height: auto;
            display: block;
        }}
        
        .rotated-cell {{
            grid-column: 1 / 4;
            text-align: center;
        }}
        
        .rotated-cell img {{
            width: {image_width}cm;
            height: auto;
            transform: rotate({rotation_angle}deg);
        }}
        
        .rotated-cell-negative img {{
            transform: rotate({rotation_angle_negative}deg);
        }}
    </style>
</head>
<body>
{content}
</body>
</html>
"#;

fn generate_html_content(
    images: &[RectoVersoImagePair],
    margins: &PageMargins,
    config: &GridConfig,
) -> String {
    let mut pages = String::new();

    if images.is_empty() {
        pages.push_str("<div class=\"page\">No images to display.</div>");
        return pages;
    }

    // Chunk images into groups of 4 (3 normal + 1 rotated)
    for chunk in images.chunks(4) {
        // RECTO page
        pages.push_str("<div class=\"page\">\n");
        pages.push_str("  <div class=\"grid\">\n");
        
        // First row: up to 3 images
        for img in chunk.iter().take(3) {
            pages.push_str(&format!(
                "    <div class=\"image-cell\"><img src=\"file://{}\" alt=\"bookmark\"/></div>\n",
                img.recto_path
            ));
        }
        
        // Second row: rotated image if present
        if let Some(img) = chunk.get(3) {
            pages.push_str(&format!(
                "    <div class=\"rotated-cell\"><img src=\"file://{}\" alt=\"bookmark\"/></div>\n",
                img.recto_path
            ));
        }
        
        pages.push_str("  </div>\n");
        pages.push_str("</div>\n");
        
        // VERSO page
        pages.push_str("<div class=\"page\">\n");
        pages.push_str("  <div class=\"grid\">\n");
        
        // First row: up to 3 images in REVERSE order
        for img in chunk.iter().take(3).rev() {
            pages.push_str(&format!(
                "    <div class=\"image-cell\"><img src=\"file://{}\" alt=\"bookmark\"/></div>\n",
                img.verso_path
            ));
        }
        
        // Second row: rotated image if present (negative angle)
        if let Some(img) = chunk.get(3) {
            pages.push_str(&format!(
                "    <div class=\"rotated-cell rotated-cell-negative\"><img src=\"file://{}\" alt=\"bookmark\"/></div>\n",
                img.verso_path
            ));
        }
        
        pages.push_str("  </div>\n");
        pages.push_str("</div>\n");
    }

    pages
}

pub fn generate_pdf_html(
    images: &[RectoVersoImagePair],
) -> Result<Vec<u8>, Box<dyn std::error::Error + Sync + Send>> {
    let margins = PageMargins::default();
    let config = GridConfig::default();
    generate_pdf_html_with_config(images, &margins, &config)
}

pub fn generate_pdf_html_with_config(
    images: &[RectoVersoImagePair],
    margins: &PageMargins,
    config: &GridConfig,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Sync + Send>> {
    // Check for calibration offsets from environment
    let horizontal_offset = std::env::var("CALIBRATION_OFFSET_HORIZONTAL")
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);
    
    let vertical_offset = std::env::var("CALIBRATION_OFFSET_VERTICAL")
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);
    
    let calibration_offsets = CalibrationOffsets {
        horizontal_cm: horizontal_offset,
        vertical_cm: vertical_offset,
    };
    
    // Apply calibration offsets to margins
    let adjusted_margins = calibration_offsets.apply_to_margins(margins);
    
    generate_pdf_html_internal(images, &adjusted_margins, config)
}

fn generate_pdf_html_internal(
    images: &[RectoVersoImagePair],
    margins: &PageMargins,
    config: &GridConfig,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Sync + Send>> {
    if images.is_empty() {
        return Err("No images provided for PDF generation".into());
    }

    let content = generate_html_content(images, margins, config);
    
    let html = HTML_TEMPLATE
        .replace("{top}", &margins.top.to_string())
        .replace("{bottom}", &margins.bottom.to_string())
        .replace("{left}", &margins.left.to_string())
        .replace("{right}", &margins.right.to_string())
        .replace("{column_gutter}", &config.column_gutter.to_string())
        .replace("{row_gutter}", &config.row_gutter.to_string())
        .replace("{image_width}", &config.image_width.to_string())
        .replace("{rotation_angle}", &config.rotation_angle.to_string())
        .replace("{rotation_angle_negative}", &format!("-{}", config.rotation_angle))
        .replace("{content}", &content);

    // Write HTML to temporary file
    let temp_dir = std::env::temp_dir();
    let html_file = temp_dir.join(format!("bookmark_{}.html", uuid::Uuid::new_v4()));
    let pdf_file = temp_dir.join(format!("bookmark_{}.pdf", uuid::Uuid::new_v4()));
    
    let mut file = fs::File::create(&html_file)?;
    file.write_all(html.as_bytes())?;
    drop(file);

    // Convert HTML to PDF using wkhtmltopdf
    let output = Command::new("wkhtmltopdf")
        .arg("--page-size")
        .arg("A4")
        .arg("--enable-local-file-access")
        .arg(&html_file)
        .arg(&pdf_file)
        .output()
        .map_err(|e| format!("Failed to execute wkhtmltopdf: {}. Make sure wkhtmltopdf is installed.", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("wkhtmltopdf failed: {}", stderr).into());
    }

    // Read the generated PDF
    let pdf_data = fs::read(&pdf_file)?;

    // Clean up temporary files
    let _ = fs::remove_file(&html_file);
    let _ = fs::remove_file(&pdf_file);

    Ok(pdf_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_html_content() {
        let images = vec![
            RectoVersoImagePair {
                recto_path: "/tmp/recto1.png".to_string(),
                verso_path: "/tmp/verso1.png".to_string(),
            },
            RectoVersoImagePair {
                recto_path: "/tmp/recto2.png".to_string(),
                verso_path: "/tmp/verso2.png".to_string(),
            },
        ];

        let margins = PageMargins::default();
        let config = GridConfig::default();
        let content = generate_html_content(&images, &margins, &config);

        // Check that content contains expected elements
        assert!(content.contains("class=\"page\""));
        assert!(content.contains("class=\"grid\""));
        assert!(content.contains("class=\"image-cell\""));
        assert!(content.contains("/tmp/recto1.png"));
        assert!(content.contains("/tmp/verso1.png"));
    }
}
