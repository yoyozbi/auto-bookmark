use itertools::Itertools;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use tokio::task::JoinSet;

use crate::generation::RectoVersoImagePair;
use crate::generation::pdf_wrapper::{ImageExtractionError, PdfDocumentWrapper};

const OUTPUT_DIR: &str = "output";

#[derive(Debug, Clone)]
struct RectoVersoPair {
    recto_path: String,
    verso_path: String,
}

#[derive(Debug)]
pub enum ExtractionError {
    InvalidPdfPath(String),
    OddPageCount(usize),
    IoError(std::io::Error),
    ExtractionFailed(String),
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
        }
    }
}

impl std::error::Error for ExtractionError {}

impl From<std::io::Error> for ExtractionError {
    fn from(error: std::io::Error) -> Self {
        ExtractionError::IoError(error)
    }
}

impl From<ImageExtractionError> for ExtractionError {
    fn from(value: ImageExtractionError) -> Self {
        match value {
            super::pdf_wrapper::ImageExtractionError::Parsing => {
                ExtractionError::ExtractionFailed("Unknown parsing error".to_string())
            }

            super::pdf_wrapper::ImageExtractionError::Reading(e) => ExtractionError::IoError(e),

            super::pdf_wrapper::ImageExtractionError::Writing(e) => ExtractionError::IoError(e),
        }
    }
}

struct PdfImageExtractor {
    request_id: Uuid,
}

impl PdfImageExtractor {
    fn new(request_id: Uuid) -> Self {
        Self { request_id }
    }

    async fn extract_recto_verso_pairs<P: AsRef<Path>>(
        &self,
        pdf_path: P,
    ) -> Result<Vec<RectoVersoPair>, ExtractionError> {
        let pdf_path = pdf_path.as_ref();

        if !pdf_path.exists() {
            return Err(ExtractionError::InvalidPdfPath(
                pdf_path.to_string_lossy().to_string(),
            ));
        }

        let mut doc_wrapper = PdfDocumentWrapper::load_from_file(pdf_path).await?;
        let page_count = doc_wrapper.page_count();

        if page_count % 2 != 0 {
            return Err(ExtractionError::OddPageCount(page_count));
        }

        self.create_output_directory()?;

        let mut pairs = Vec::new();

        // The wrapper method takes care of checking
        //  if the number of images doesn't align with the number of page
        let images = doc_wrapper
            .extract_images_from_pages(self.get_output_directory())
            .await?;
        for pair in images.chunks(2) {
            pairs.push(RectoVersoPair {
                recto_path: pair[0].to_str().unwrap().to_string(),
                verso_path: pair[1].to_str().unwrap().to_string(),
            });
        }

        Ok(pairs)
    }

    fn create_output_directory(&self) -> Result<(), ExtractionError> {
        let output_dir = self.get_output_directory();
        if !output_dir.exists() {
            fs::create_dir_all(&output_dir)?;
        }
        Ok(())
    }

    fn get_output_directory(&self) -> PathBuf {
        let mut output = PathBuf::from(OUTPUT_DIR);
        output.push(self.request_id.to_string());
        output
    }
}

pub async fn split_pages_from_input_pdfs<P1: AsRef<Path> + Sync + Sized>(
    input_pdfs: &[P1],
    request_id: Uuid,
) -> Result<Vec<RectoVersoImagePair>, ExtractionError> {
    let tasks: JoinSet<_> = input_pdfs
        .iter()
        .map(|pdf_path| {
            let pdf_path = pdf_path.as_ref().to_path_buf();
            async move {
                let extractor = PdfImageExtractor::new(request_id);
                if !pdf_path.exists() {
                    return Err(ExtractionError::InvalidPdfPath(String::from(
                        pdf_path.to_str().unwrap_or("unknown"),
                    )));
                }

                extractor.extract_recto_verso_pairs(pdf_path).await
            }
        })
        .collect();

    let result = tasks.join_all().await;

    let values: Vec<_> = result.into_iter().try_collect()?;

    Ok(values
        .into_iter()
        .flatten()
        .map(|f| RectoVersoImagePair {
            recto_path: f.recto_path,
            verso_path: f.verso_path,
        })
        .collect_vec())
}
