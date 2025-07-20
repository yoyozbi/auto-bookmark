const PAGE_DEFINITION: &str = r#"#set page(margin: (
 top: {top}cm,
 bottom: {bottom}cm,
 left: {left}cm,
 right: {right}cm
))

"#;

const GRID_DEFINITION: &str = r#"#grid(
  columns: (auto, auto, auto),
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

#[cfg(feature = "ssr")]
use {
    typst::{
        foundations::{Bytes, Datetime},
        syntax::{FileId, Source},
        text::{Font, FontBook},
        utils::LazyHash,
        Library, World,
    },
    typst_pdf::{pdf, PdfOptions},
};

#[derive(Clone, Debug)]
pub(crate) struct ImageInfo {
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

#[derive(Default, Clone, Debug)]
pub struct DocumentMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
}

#[cfg(feature = "ssr")]
pub struct SimpleTypstWorld {
    main: Source,
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
}

#[cfg(feature = "ssr")]
impl SimpleTypstWorld {
    pub fn new(content: String) -> Self {
        Self {
            main: Source::detached(content),
            library: LazyHash::new(Library::default()),
            book: LazyHash::new(FontBook::new()),
            fonts: Vec::new(),
        }
    }
}

#[cfg(feature = "ssr")]
impl World for SimpleTypstWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    fn main(&self) -> FileId {
        self.main.id()
    }

    fn source(&self, id: FileId) -> Result<Source, typst::diag::FileError> {
        if id == self.main.id() {
            Ok(self.main.clone())
        } else {
            Err(typst::diag::FileError::NotFound(
                id.vpath().as_rootless_path().into(),
            ))
        }
    }

    fn file(&self, _id: FileId) -> Result<Bytes, typst::diag::FileError> {
        Err(typst::diag::FileError::NotFound(std::path::PathBuf::new()))
    }

    fn font(&self, id: usize) -> Option<Font> {
        self.fonts.get(id).cloned()
    }

    fn today(&self, _offset: Option<i64>) -> Option<Datetime> {
        None
    }
}

//TODO: Refactory this to use structs for each kind of things
fn generate_typst_content(
    images: &[ImageInfo],
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

#[cfg(feature = "ssr")]
pub fn generate_pdf(images: &[ImageInfo]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let margins = PageMargins::default();
    let config = GridConfig::default();
    generate_pdf_with_config(images, &margins, &config, &DocumentMetadata::default())
}

#[cfg(feature = "ssr")]
pub fn generate_pdf_with_config(
    images: &[ImageInfo],
    margins: &PageMargins,
    config: &GridConfig,
    _metadata: &DocumentMetadata,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    if images.is_empty() {
        return Err("No images provided for PDF generation".into());
    }

    // Generate Typst content
    let typst_content = generate_typst_content(images, margins, config);
    println!("Typst content: {}", typst_content);

    // Create Typst world
    let world = SimpleTypstWorld::new(typst_content);

    // Compile the document
    let result = typst::compile(&world);
    let document = result.output.map_err(|errors| {
        let error_messages: Vec<String> = errors
            .iter()
            .map(|error| format!("{}", error.message))
            .collect();
        format!("Typst compilation failed: {}", error_messages.join("; "))
    })?;

    // Export to PDF
    let pdf_data = pdf(&document, &PdfOptions::default())
        .map_err(|e| format!("PDF export failed: {:?}", e))?;

    Ok(pdf_data)
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_generated_content_should_be_correct() {
        let images = vec![
            ImageInfo {
                recto_path: "recto/devils4.png".to_string(),
                verso_path: "verso/devils4.png".to_string(),
            },
            ImageInfo {
                recto_path: "recto/uglylove.png".to_string(),
                verso_path: "verso/uglylove.png".to_string(),
            },
            ImageInfo {
                recto_path: "recto/yoyo.png".to_string(),
                verso_path: "verso/yoyo.png".to_string(),
            },
            ImageInfo {
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
