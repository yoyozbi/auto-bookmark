use std::path::{Path, PathBuf};
use tokio::task::JoinSet;
use uuid::Uuid;

#[cfg(feature = "ssr")]
use lopdf::Document;

use crate::generation::generate_pdf::{PdfPagePair, create_pdf_page_pairs};

#[derive(Debug)]
pub enum ExtractionError {
    InvalidPdfPath(String),
    OddPageCount(usize),
    IoError(std::io::Error),
    ExtractionFailed(String),
    PdfProcessingError(String),
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
            ExtractionError::PdfProcessingError(msg) => write!(f, "PDF processing error: {}", msg),
        }
    }
}

impl std::error::Error for ExtractionError {}

impl From<std::io::Error> for ExtractionError {
    fn from(error: std::io::Error) -> Self {
        ExtractionError::IoError(error)
    }
}

/// Get the number of pages in a PDF file using lopdf for accurate parsing
fn get_pdf_page_count<P: AsRef<Path> + std::fmt::Debug>(
    pdf_path: P,
) -> Result<usize, ExtractionError> {
    // Load and parse PDF using lopdf
    let document = Document::load(pdf_path.as_ref())
        .map_err(|e| ExtractionError::PdfProcessingError(format!("Failed to load PDF: {}", e)))?;

    let pages = document.get_pages();
    let page_count = pages.len();

    Ok(page_count)
}

/// Copy a PDF file to the output directory and create page pairs
/// This replaces the old image extraction approach
async fn process_pdf_file<P: AsRef<Path>>(
    pdf_path: P,
    output_dir: P,
    request_id: Uuid,
) -> Result<Vec<PdfPagePair>, ExtractionError> {
    let pdf_path = pdf_path.as_ref();

    if !pdf_path.exists() {
        return Err(ExtractionError::InvalidPdfPath(
            pdf_path.to_string_lossy().to_string(),
        ));
    }

    // Get page count
    let page_count = get_pdf_page_count(pdf_path)?;

    if page_count % 2 != 0 {
        return Err(ExtractionError::OddPageCount(page_count));
    }

    // Create output directory
    tokio::fs::create_dir_all(&output_dir).await?;

    // Copy PDF to output directory with request ID
    let file_stem = pdf_path
        .file_stem()
        .unwrap_or_else(|| std::ffi::OsStr::new("document"))
        .to_string_lossy();

    let output_pdf_name = format!("{}_{}.pdf", file_stem, request_id);
    let output_pdf_path = output_dir.as_ref().join(&output_pdf_name);

    tokio::fs::copy(pdf_path, &output_pdf_path).await?;

    // Create page pairs
    let pairs = create_pdf_page_pairs(&output_pdf_path, page_count)
        .map_err(|e| ExtractionError::PdfProcessingError(e.to_string()))?;

    Ok(pairs)
}

/// Process multiple PDF files and create page pairs for each
pub async fn split_pages_from_input_pdfs(
    pdf_files: &[String],
    request_id: Uuid,
) -> Result<Vec<PdfPagePair>, ExtractionError> {
    if pdf_files.is_empty() {
        return Ok(Vec::new());
    }

    // Create output directory
    let output_dir = PathBuf::from("output").join(request_id.to_string());
    tokio::fs::create_dir_all(&output_dir).await?;

    let mut all_pairs = Vec::new();
    let mut join_set = JoinSet::new();

    // Process all PDFs concurrently
    for pdf_path in pdf_files {
        let pdf_path = pdf_path.clone();
        let output_dir = output_dir.clone();

        join_set.spawn(async move {
            process_pdf_file(pdf_path.as_str(), output_dir.to_str().unwrap(), request_id).await
        });
    }

    // Collect results
    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(Ok(mut pairs)) => all_pairs.append(&mut pairs),
            Ok(Err(e)) => return Err(e),
            Err(e) => return Err(ExtractionError::ExtractionFailed(e.to_string())),
        }
    }

    if all_pairs.is_empty() {
        return Err(ExtractionError::ExtractionFailed(
            "No valid page pairs created".to_string(),
        ));
    }

    Ok(all_pairs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_get_pdf_page_count_basic() {
        use lopdf::{Document, Object, dictionary};
        
        let temp_dir = TempDir::new().unwrap();
        let pdf_path = temp_dir.path().join("test.pdf");
        
        // Create a simple PDF with 2 pages using lopdf
        let mut doc = Document::with_version("1.4");
        
        // Add two pages to the document
        let pages_id = doc.new_object_id();
        let page1_id = doc.new_object_id();
        let page2_id = doc.new_object_id();
        
        // Create pages object
        doc.objects.insert(pages_id, Object::Dictionary(dictionary! {
            "Type" => "Pages",
            "Kids" => vec![Object::Reference(page1_id), Object::Reference(page2_id)],
            "Count" => 2,
        }));
        
        // Create page 1
        doc.objects.insert(page1_id, Object::Dictionary(dictionary! {
            "Type" => "Page",
            "Parent" => Object::Reference(pages_id),
            "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
        }));
        
        // Create page 2
        doc.objects.insert(page2_id, Object::Dictionary(dictionary! {
            "Type" => "Page",
            "Parent" => Object::Reference(pages_id),
            "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
        }));
        
        // Create catalog
        let catalog_id = doc.new_object_id();
        doc.objects.insert(catalog_id, Object::Dictionary(dictionary! {
            "Type" => "Catalog",
            "Pages" => Object::Reference(pages_id),
        }));
        
        // Set catalog as root
        doc.trailer.set("Root", Object::Reference(catalog_id));
        
        // Save the PDF
        doc.save(&pdf_path).unwrap();

        let result = get_pdf_page_count(pdf_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
    }

    #[test]
    fn test_create_pairs_even_pages() {
        let pdf_path = Path::new("test.pdf");
        let pairs = create_pdf_page_pairs(pdf_path, 4).unwrap();

        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0].recto_page, 1);
        assert_eq!(pairs[0].verso_page, 2);
        assert_eq!(pairs[1].recto_page, 3);
        assert_eq!(pairs[1].verso_page, 4);
    }

    #[test]
    fn test_create_pairs_odd_pages_error() {
        let pdf_path = Path::new("test.pdf");
        let result = create_pdf_page_pairs(pdf_path, 3);

        assert!(result.is_err());
    }
}
