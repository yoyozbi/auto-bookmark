use typst_as_lib::TypstEngine;

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
pub(crate) struct RectoVersoImagePair {
    pub recto_path: String,
    pub verso_path: String,
}

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
  columns: (auto, auto, auto),
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
  columns: (auto, auto, auto),
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
}
