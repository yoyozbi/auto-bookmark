use std::path::Path;
use typst_as_lib::TypstEngine;

const PAGE_DEFINITION: &str = r#"#set page(margin: (
 top: {top}cm,
 bottom: {bottom}cm,
 left: {left}cm,
 right: {right}cm
))

"#;

const GRID_DEFINITION: &str = r#"
#pad(left: {pad-left}cm, top: {pad-top}cm)[
#grid(
  columns: (1fr, 1fr, 1fr),
  rows: (auto, auto),
  column-gutter: {column-gutter}cm,
  row-gutter: {row-gutter}cm,
  align: center,


{cells}
)]
"#;

const PDF_PAGE_CELL: &str = r#"image("{pdf_path}", width: {width}cm, page: {page}),
"#;
const ROTATED_PDF_PAGE_CELL: &str = r#"grid.cell(rotate({angle}deg, image("{pdf_path}", width: {width}cm, page: {page}), reflow: true), colspan: 3),
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
    pub top_offset: f64,
    pub left_offset: f64,
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            column_gutter: 3.0,
            row_gutter: 0.7,
            image_width: 5.5,
            rotation_angle: 75.0,
            top_offset: 0.0,
            left_offset: 0.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PdfPagePair {
    pub pdf_path: String,
    pub recto_page: usize,
    pub verso_page: usize,
}

fn generate_typst_content(
    pages: &[PdfPagePair],
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

    if pages.is_empty() {
        content.push_str("No pages to display.\n");
        return content;
    }

    // Chunk pages into groups of 4 (3 normal + 1 rotated)
    for chunk in pages.chunks(4) {
        // RECTO
        let mut recto_cells = String::new();
        // First row: up to 3 pages
        for page_pair in chunk.iter().take(3) {
            recto_cells.push_str(
                &PDF_PAGE_CELL
                    .replace("{pdf_path}", &page_pair.pdf_path)
                    .replace("{page}", &page_pair.recto_page.to_string())
                    .replace("{width}", &config.image_width.to_string()),
            );
        }
        // Second row: rotated page if present
        if let Some(page_pair) = chunk.get(3) {
            recto_cells.push_str(
                &ROTATED_PDF_PAGE_CELL
                    .replace("{angle}", &config.rotation_angle.to_string())
                    .replace("{pdf_path}", &page_pair.pdf_path)
                    .replace("{page}", &page_pair.recto_page.to_string())
                    .replace("{width}", &config.image_width.to_string()),
            );
        }

        // For recto (front) pages: apply positive offsets as left/top padding 
        // Positive = shift front pages right/down, Negative = no padding on front
        let mut pad_left_value = if config.left_offset.is_sign_positive() {
            config.left_offset.to_string()
        } else {
            0.to_string()
        };

        let mut pad_top_value = if config.top_offset.is_sign_positive() {
            config.top_offset.to_string()
        } else {
            0.to_string()
        };

        content.push_str(
            &GRID_DEFINITION
                .replace("{column-gutter}", &config.column_gutter.to_string())
                .replace("{row-gutter}", &config.row_gutter.to_string())
                .replace("{cells}", &recto_cells)
                .replace("{pad-left}", &pad_left_value)
                .replace("{pad-top}", &pad_top_value),
        );

        // VERSO
        let mut verso_cells = String::new();
        // First row: up to 3 pages, reversed order
        for page_pair in chunk.iter().take(3).rev() {
            verso_cells.push_str(
                &PDF_PAGE_CELL
                    .replace("{pdf_path}", &page_pair.pdf_path)
                    .replace("{page}", &page_pair.verso_page.to_string())
                    .replace("{width}", &config.image_width.to_string()),
            );
        }
        // Second row: rotated page if present (negative angle)
        if let Some(page_pair) = chunk.get(3) {
            verso_cells.push_str(
                &ROTATED_PDF_PAGE_CELL
                    .replace("{angle}", &format!("-{}", config.rotation_angle))
                    .replace("{pdf_path}", &page_pair.pdf_path)
                    .replace("{page}", &page_pair.verso_page.to_string())
                    .replace("{width}", &config.image_width.to_string()),
            );
        }

        // For verso (back) pages: apply negative offsets as left/top padding
        // Negative = shift back pages right/down (front pages appear left/up), Positive = no padding on back  
        pad_left_value = if config.left_offset.is_sign_negative() {
            config.left_offset.abs().to_string()
        } else {
            0.to_string()
        };

        pad_top_value = if config.top_offset.is_sign_negative() {
            config.top_offset.abs().to_string()
        } else {
            0.to_string()
        };

        content.push_str(
            &GRID_DEFINITION
                .replace("{column-gutter}", &config.column_gutter.to_string())
                .replace("{row-gutter}", &config.row_gutter.to_string())
                .replace("{cells}", &verso_cells)
                .replace("{pad-left}", &pad_left_value)
                .replace("{pad-top}", &pad_top_value),
        );
    }

    content
}

pub fn generate_pdf_with_config(
    pages: &[PdfPagePair],
    margins: &PageMargins,
    config: &GridConfig,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Sync + Send>> {
    if pages.is_empty() {
        return Err("No pages provided for PDF generation".into());
    }

    let typst_content = generate_typst_content(pages, margins, config);

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

/// Creates PdfPagePair instances from a PDF file path with the given number of pages
/// Pages are paired as: (1,2), (3,4), (5,6), etc.
pub fn create_pdf_page_pairs(
    pdf_path: &Path,
    page_count: usize,
) -> Result<Vec<PdfPagePair>, Box<dyn std::error::Error + Sync + Send>> {
    if !page_count.is_multiple_of(2) {
        return Err(format!(
            "PDF has odd number of pages ({}). Expected even number for recto-verso pairs.",
            page_count
        )
        .into());
    }

    let pdf_path_str = pdf_path.to_string_lossy().to_string();
    let pairs = (0..page_count)
        .step_by(2)
        .map(|i| PdfPagePair {
            pdf_path: pdf_path_str.clone(),
            recto_page: i + 1, // Typst uses 1-based page numbering
            verso_page: i + 2,
        })
        .collect();

    Ok(pairs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_generated_content_should_be_correct() {
        let pages = vec![
            PdfPagePair {
                pdf_path: "document.pdf".to_string(),
                recto_page: 1,
                verso_page: 2,
            },
            PdfPagePair {
                pdf_path: "document.pdf".to_string(),
                recto_page: 3,
                verso_page: 4,
            },
            PdfPagePair {
                pdf_path: "document.pdf".to_string(),
                recto_page: 5,
                verso_page: 6,
            },
            PdfPagePair {
                pdf_path: "document.pdf".to_string(),
                recto_page: 7,
                verso_page: 8,
            },
        ];
        const EXPECTED: &str = r#"#set page(margin: (
 top: 0cm,
 bottom: 0cm,
 left: 2cm,
 right: 2cm
))


#pad(left: 0cm, top: 0cm)[
#grid(
  columns: (1fr, 1fr, 1fr),
  rows: (auto, auto),
  column-gutter: 3cm,
  row-gutter: 0.7cm,
  align: center,


image("document.pdf", width: 5.5cm, page: 1),
image("document.pdf", width: 5.5cm, page: 3),
image("document.pdf", width: 5.5cm, page: 5),
grid.cell(rotate(75deg, image("document.pdf", width: 5.5cm, page: 7), reflow: true), colspan: 3),

)]

#pad(left: 0cm, top: 0cm)[
#grid(
  columns: (1fr, 1fr, 1fr),
  rows: (auto, auto),
  column-gutter: 3cm,
  row-gutter: 0.7cm,
  align: center,


image("document.pdf", width: 5.5cm, page: 6),
image("document.pdf", width: 5.5cm, page: 4),
image("document.pdf", width: 5.5cm, page: 2),
grid.cell(rotate(-75deg, image("document.pdf", width: 5.5cm, page: 8), reflow: true), colspan: 3),

)]
"#;

        let margins = PageMargins::default();
        let config = GridConfig::default();

        let content = generate_typst_content(&pages, &margins, &config);

        assert_eq!(content, EXPECTED);
    }
}
