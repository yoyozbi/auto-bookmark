use typst_as_lib::TypstEngine;

use super::RectoVersoImagePair;

const PAGE_DEFINITION: &str = r#"#set page(margin: (
 top: {top}cm,
 bottom: {bottom}cm,
 left: {left}cm,
 right: {right}cm
))

"#;

const GRID_DEFINITION: &str = r#"#grid(
  columns: (1fr, 1fr, 1fr),
  rows: (auto, auto),
  column-gutter: {column-gutter}cm,
  row-gutter: {row-gutter}cm,
  align: center,


{cells}
)
"#;

const IMAGE_CELL: &str = r#"image("{path}", width: {width}cm),
"#;
const ROTATED_IMAGE_CELL: &str = r#"grid.cell(rotate({angle}deg, image("{path}", width: {width}cm), reflow: true), colspan: 3),
"#;

#[derive(Clone, Debug)]
pub struct PageMargins {
    pub top: f64,
    pub bottom: f64,
    pub left: f64,
    pub right: f64,
}

impl Default for PageMargins {
    fn default() -> Self {
        Self {
            top: 0.0,
            bottom: 0.0,
            left: 2.0,
            right: 2.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct GridConfig {
    pub column_gutter: f64,
    pub row_gutter: f64,
    pub image_width: f64,
    pub rotation_angle: f64,
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            column_gutter: 3.0,
            row_gutter: 0.7,
            image_width: 5.5,
            rotation_angle: 75.0,
        }
    }
}

/// Validates that margins and grid configuration won't cause content overflow
/// A4 page dimensions: 21cm width x 29.7cm height
pub fn validate_margins_and_grid(margins: &PageMargins, config: &GridConfig) -> Result<(), String> {
    const A4_WIDTH_CM: f64 = 21.0;
    const A4_HEIGHT_CM: f64 = 29.7;
    const NUM_COLUMNS: usize = 3;
    
    // Check if margins are negative (which would be problematic)
    if margins.left < 0.0 || margins.right < 0.0 || margins.top < 0.0 || margins.bottom < 0.0 {
        return Err(format!(
            "Margins cannot be negative. \
            Left: {:.2}cm, Right: {:.2}cm, Top: {:.2}cm, Bottom: {:.2}cm",
            margins.left, margins.right, margins.top, margins.bottom
        ));
    }
    
    // Calculate total horizontal space needed
    let total_horizontal = margins.left + margins.right 
        + (config.image_width * NUM_COLUMNS as f64)
        + (config.column_gutter * (NUM_COLUMNS - 1) as f64);
    
    // Only error if significantly over page width (allowing some flexibility for Typst/PDF rendering)
    if total_horizontal > A4_WIDTH_CM * 1.3 {
        return Err(format!(
            "Content width ({:.2}cm) significantly exceeds page width ({:.2}cm). \
            Margins: left={:.2}cm, right={:.2}cm. \
            Reduce calibration offsets significantly.",
            total_horizontal, A4_WIDTH_CM, margins.left, margins.right
        ));
    }
    
    // Check vertical space (2 rows + gutter) - also allow flexibility
    let total_vertical = margins.top + margins.bottom + config.row_gutter;
    if total_vertical > A4_HEIGHT_CM * 1.3 {
        return Err(format!(
            "Vertical spacing ({:.2}cm) significantly exceeds page height ({:.2}cm). \
            Margins: top={:.2}cm, bottom={:.2}cm.",
            total_vertical, A4_HEIGHT_CM, margins.top, margins.bottom
        ));
    }
    
    Ok(())
}

fn generate_typst_content(
    images: &[RectoVersoImagePair],
    margins: &PageMargins,
    config: &GridConfig,
) -> String {
    let mut content = String::new();

    let page_def = PAGE_DEFINITION
        .replace("{top}", &margins.top.to_string())
        .replace("{bottom}", &margins.bottom.to_string())
        .replace("{left}", &margins.left.to_string())
        .replace("{right}", &margins.right.to_string());
    content.push_str(&page_def);

    if images.is_empty() {
        content.push_str("No images to display.\n");
        return content;
    }

    // Chunk images into groups of 4 (3 normal + 1 rotated)
    for (_, chunk) in images.chunks(4).enumerate() {
        // RECTO
        let mut recto_cells = String::new();
        // First row: up to 3 images
        for img in chunk.iter().take(3) {
            recto_cells.push_str(
                &IMAGE_CELL
                    .replace("{path}", &img.recto_path)
                    .replace("{width}", &config.image_width.to_string()),
            );
        }
        // Second row: rotated image if present
        if let Some(img) = chunk.get(3) {
            recto_cells.push_str(
                &ROTATED_IMAGE_CELL
                    .replace("{angle}", &config.rotation_angle.to_string())
                    .replace("{path}", &img.recto_path)
                    .replace("{width}", &config.image_width.to_string()),
            );
        }
        content.push_str(
            &GRID_DEFINITION
                .replace("{column-gutter}", &config.column_gutter.to_string())
                .replace("{row-gutter}", &config.row_gutter.to_string())
                .replace("{cells}", &recto_cells),
        );

        // VERSO
        let mut verso_cells = String::new();
        // First row: up to 3 images, reversed order
        for img in chunk.iter().take(3).rev() {
            verso_cells.push_str(
                &IMAGE_CELL
                    .replace("{path}", &img.verso_path)
                    .replace("{width}", &config.image_width.to_string()),
            );
        }
        // Second row: rotated image if present (negative angle)
        if let Some(img) = chunk.get(3) {
            verso_cells.push_str(
                &ROTATED_IMAGE_CELL
                    .replace("{angle}", &format!("-{}", config.rotation_angle))
                    .replace("{path}", &img.verso_path)
                    .replace("{width}", &config.image_width.to_string()),
            );
        }

        content.push_str(
            &GRID_DEFINITION
                .replace("{column-gutter}", &config.column_gutter.to_string())
                .replace("{row-gutter}", &config.row_gutter.to_string())
                .replace("{cells}", &verso_cells),
        );
    }

    content
}

pub fn generate_pdf(
    images: &[RectoVersoImagePair],
) -> Result<Vec<u8>, Box<dyn std::error::Error + Sync + Send>> {
    let margins = PageMargins::default();
    let config = GridConfig::default();
    generate_pdf_with_config(images, &margins, &config)
}

pub fn generate_pdf_with_config(
    images: &[RectoVersoImagePair],
    margins: &PageMargins,
    config: &GridConfig,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Sync + Send>> {
    if images.is_empty() {
        return Err("No images provided for PDF generation".into());
    }

    // Validate margins and grid configuration
    validate_margins_and_grid(margins, config)?;

    let typst_content = generate_typst_content(images, margins, config);

    let template = TypstEngine::builder()
        .main_file(typst_content)
        .with_file_system_resolver("./".to_owned())
        .build();

    // Compile the document
    let result = template.compile();
    let document = result
        .output
        .map_err(|error| format!("Typst compilation failed: {}", error))?;

    // Export to PDF
    let pdf_data = typst_pdf::pdf(&document, &typst_pdf::PdfOptions::default())
        .map_err(|e| format!("PDF export failed: {:?}", e))?;

    Ok(pdf_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_generated_content_should_be_correct() {
        let images = vec![
            RectoVersoImagePair {
                recto_path: "recto/devils4.png".to_string(),
                verso_path: "verso/devils4.png".to_string(),
            },
            RectoVersoImagePair {
                recto_path: "recto/uglylove.png".to_string(),
                verso_path: "verso/uglylove.png".to_string(),
            },
            RectoVersoImagePair {
                recto_path: "recto/yoyo.png".to_string(),
                verso_path: "verso/yoyo.png".to_string(),
            },
            RectoVersoImagePair {
                recto_path: "recto/dragon.png".to_string(),
                verso_path: "verso/dragon.png".to_string(),
            },
        ];
        const EXPECTED: &str = r#"#set page(margin: (
 top: 0cm,
 bottom: 0cm,
 left: 2cm,
 right: 2cm
))

#grid(
  columns: (1fr, 1fr, 1fr),
  rows: (auto, auto),
  column-gutter: 3cm,
  row-gutter: 0.7cm,
  align: center,


image("recto/devils4.png", width: 5.5cm),
image("recto/uglylove.png", width: 5.5cm),
image("recto/yoyo.png", width: 5.5cm),
grid.cell(rotate(75deg, image("recto/dragon.png", width: 5.5cm), reflow: true), colspan: 3),

)
#grid(
  columns: (1fr, 1fr, 1fr),
  rows: (auto, auto),
  column-gutter: 3cm,
  row-gutter: 0.7cm,
  align: center,


image("verso/yoyo.png", width: 5.5cm),
image("verso/uglylove.png", width: 5.5cm),
image("verso/devils4.png", width: 5.5cm),
grid.cell(rotate(-75deg, image("verso/dragon.png", width: 5.5cm), reflow: true), colspan: 3),

)
"#;

        let margins = PageMargins::default();
        let config = GridConfig::default();

        let content = generate_typst_content(&images, &margins, &config);

        println!("Generated Typst content:\n{}", content);

        assert_eq!(content, EXPECTED);
    }

    #[test]
    fn test_validate_margins_valid() {
        let margins = PageMargins::default();
        let config = GridConfig::default();
        
        // Should pass with default values
        let result = validate_margins_and_grid(&margins, &config);
        if let Err(e) = &result {
            println!("Validation error: {}", e);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_margins_overflow() {
        let margins = PageMargins {
            top: 0.0,
            bottom: 0.0,
            left: 15.0,  // Very large left margin
            right: 15.0, // Very large right margin
        };
        let config = GridConfig::default();
        
        // Should fail - content would overflow significantly
        let result = validate_margins_and_grid(&margins, &config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("significantly exceeds page width"));
    }

    #[test]
    fn test_validate_margins_negative() {
        let margins = PageMargins {
            top: -1.0,  // Negative margin
            bottom: 0.0,
            left: 2.0,
            right: 2.0,
        };
        let config = GridConfig::default();
        
        // Should fail - negative margins not allowed
        let result = validate_margins_and_grid(&margins, &config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cannot be negative"));
    }
}
