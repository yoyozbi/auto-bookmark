use std::fs;

use std::path::{Path, PathBuf};

#[cfg(feature = "ssr")]
use pdfium_render::prelude::*;

const OUTPUT_DIR: &str = "extracted_images";
const DPI: u32 = 300;
const IMAGE_EXTENSION: &str = "png";

#[derive(Debug, Clone)]
struct RectoVersoPair {
    recto_path: String,
    verso_path: String,
    #[allow(dead_code)]
    pair_index: usize,
}

#[derive(Debug)]
enum ExtractionError {
    InvalidPdfPath(String),
    OddPageCount(usize),
    IoError(std::io::Error),
    ExtractionFailed(String),
    InvalidPageNumber(usize),
}

impl std::fmt::Display for ExtractionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ExtractionError::InvalidPdfPath(path) => write!(f, "Invalid PDF path: {}", path),
            ExtractionError::OddPageCount(count) => {
                write!(
                    f,
                    "PDF has odd number of pages ({}). Expected even number for recto-verso pairs.",
                    count
                )
            }
            ExtractionError::IoError(e) => write!(f, "IO Error: {}", e),
            ExtractionError::ExtractionFailed(msg) => write!(f, "Extraction failed: {}", msg),
            ExtractionError::InvalidPageNumber(page) => write!(f, "Invalid page number: {}", page),
        }
    }
}

impl std::error::Error for ExtractionError {}

impl From<std::io::Error> for ExtractionError {
    fn from(error: std::io::Error) -> Self {
        ExtractionError::IoError(error)
    }
}

struct PdfImageExtractor;

#[cfg(feature = "ssr")]
impl PdfImageExtractor {
    fn new() -> Self {
        Self
    }

    fn extract_recto_verso_pairs<P: AsRef<Path>>(
        &self,
        pdf_path: P,
    ) -> Result<Vec<RectoVersoPair>, ExtractionError> {
        let pdf_path = pdf_path.as_ref();

        if !pdf_path.exists() {
            return Err(ExtractionError::InvalidPdfPath(
                pdf_path.to_string_lossy().to_string(),
            ));
        }

        let page_count = self.get_page_count(pdf_path)?;

        if page_count % 2 != 0 {
            return Err(ExtractionError::OddPageCount(page_count));
        }

        self.create_output_directory()?;

        let mut pairs = Vec::new();
        let pair_count = page_count / 2;

        for pair_idx in 0..pair_count {
            let recto_page = pair_idx * 2 + 1;
            let verso_page = pair_idx * 2 + 2;

            let pair = self.extract_pair(pdf_path, recto_page, verso_page, pair_idx)?;
            pairs.push(pair);
        }

        Ok(pairs)
    }

    fn extract_pair<P: AsRef<Path>>(
        &self,
        pdf_path: P,
        recto_page: usize,
        verso_page: usize,
        pair_index: usize,
    ) -> Result<RectoVersoPair, ExtractionError> {
        let pdf_path = pdf_path.as_ref();
        let pdf_stem = pdf_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("document");

        let recto_filename = format!(
            "{}_page{:03}_recto.{}",
            pdf_stem, recto_page, IMAGE_EXTENSION
        );
        let verso_filename = format!(
            "{}_page{:03}_verso.{}",
            pdf_stem, verso_page, IMAGE_EXTENSION
        );

        let output_dir = PathBuf::from(OUTPUT_DIR);
        let recto_path = output_dir.join(&recto_filename);
        let verso_path = output_dir.join(&verso_filename);

        self.extract_page_as_image(pdf_path, recto_page, &recto_path)?;
        self.extract_page_as_image(pdf_path, verso_page, &verso_path)?;

        Ok(RectoVersoPair {
            recto_path: recto_path.to_string_lossy().to_string(),
            verso_path: verso_path.to_string_lossy().to_string(),
            pair_index,
        })
    }

    fn create_output_directory(&self) -> Result<(), ExtractionError> {
        let output_dir = PathBuf::from(OUTPUT_DIR);
        if !output_dir.exists() {
            fs::create_dir_all(&output_dir)?;
        }
        Ok(())
    }

    fn get_page_count<P: AsRef<Path>>(&self, pdf_path: P) -> Result<usize, ExtractionError> {
        let pdfium = Pdfium::new(
            Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
                .or_else(|_| Pdfium::bind_to_system_library())
                .or_else(|_| Pdfium::bind_to_library(std::env::var("PDFIUM_DEBUG_PATH").unwrap()))
                .map_err(|e| {
                    ExtractionError::ExtractionFailed(format!(
                        "Failed to bind to Pdfium library: {:?}",
                        e
                    ))
                })?,
        );

        let document = pdfium.load_pdf_from_file(&pdf_path, None).map_err(|e| {
            ExtractionError::ExtractionFailed(format!("Failed to load PDF: {:?}", e))
        })?;

        Ok(document.pages().len() as usize)
    }

    fn extract_page_as_image<P1: AsRef<Path>, P2: AsRef<Path>>(
        &self,
        pdf_path: P1,
        page_number: usize,
        output_path: P2,
    ) -> Result<(), ExtractionError> {
        let output_path = output_path.as_ref();
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let pdfium = Pdfium::new(
            Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
                .or_else(|_| Pdfium::bind_to_system_library())
                .or_else(|_| Pdfium::bind_to_library(std::env::var("PDFIUM_DEBUG_PATH").unwrap()))
                .map_err(|e| {
                    ExtractionError::ExtractionFailed(format!(
                        "Failed to bind to Pdfium library: {:?}",
                        e
                    ))
                })?,
        );

        let document = pdfium.load_pdf_from_file(&pdf_path, None).map_err(|e| {
            ExtractionError::ExtractionFailed(format!("Failed to load PDF: {:?}", e))
        })?;

        let page_index = page_number - 1;
        let page = document
            .pages()
            .get(page_index as u16)
            .map_err(|_| ExtractionError::InvalidPageNumber(page_number))?;

        let render_config = PdfRenderConfig::new()
            .set_target_width((8.5 * DPI as f32) as i32)
            .set_maximum_height((11.0 * DPI as f32) as i32);

        let bitmap = page.render_with_config(&render_config).map_err(|e| {
            ExtractionError::ExtractionFailed(format!("Failed to render page: {:?}", e))
        })?;

        bitmap.as_image().save(output_path).map_err(|e| {
            ExtractionError::ExtractionFailed(format!("Failed to save image: {:?}", e))
        })?;

        Ok(())
    }
}

#[cfg(not(feature = "ssr"))]
pub fn generate_pdf_from_input_pdfs<P1: AsRef<Path>>(
    input_pdfs: &[P1],
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    Err("Cannot call this from frontend!".into())
}

#[cfg(feature = "ssr")]
pub fn generate_pdf_from_input_pdfs<P1: AsRef<Path>>(
    input_pdfs: &[P1],
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut all_images = Vec::new();
    let extractor = PdfImageExtractor::new();

    for pdf_path in input_pdfs {
        let pdf_path = pdf_path.as_ref();

        if !pdf_path.exists() {
            return Err(format!("PDF file does not exist: {:?}", pdf_path).into());
        }

        let pairs = extractor
            .extract_recto_verso_pairs(pdf_path)
            .map_err(|e| format!("Failed to extract from {:?}: {}", pdf_path, e))?;

        for pair in pairs {
            all_images.push(super::generate_pdf::ImageInfo {
                recto_path: pair.recto_path,
                verso_path: pair.verso_path,
            });
        }
    }

    if all_images.is_empty() {
        return Err("No images extracted from input PDFs".into());
    }

    let pdf_data = crate::generation::generate_pdf::generate_pdf(&all_images)?;

    Ok(pdf_data)
}
