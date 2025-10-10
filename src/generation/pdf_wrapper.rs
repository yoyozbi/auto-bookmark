use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use hayro_interpret::{InterpreterSettings, Pdf};
use hayro_svg::convert;
use tokio::io::Error;
use uuid::Uuid;

#[derive(Debug)]
pub enum ImageExtractionError {
    Reading(Error),
    Writing(Error),
    Parsing,
}

pub struct PdfDocumentWrapper {
    stem: String,
    document: Pdf,
}

impl PdfDocumentWrapper {
    pub async fn load_from_file<T: AsRef<Path>>(
        path: T,
    ) -> Result<PdfDocumentWrapper, ImageExtractionError> {
        let path = path.as_ref();
        let file_data = tokio::fs::read(path)
            .await
            .map_err(|e| ImageExtractionError::Reading(e))?;
        let stem = path
            .file_stem()
            .and_then(|f| Some(f.to_string_lossy().to_string()))
            .unwrap_or(Uuid::new_v4().to_string());

        let data = Arc::new(file_data);
        let pdf = Pdf::new(data).map_err(|e| ImageExtractionError::Parsing)?;
        Ok(Self {
            stem,
            document: pdf,
        })
    }

    pub fn page_count(&mut self) -> usize {
        self.document.pages().len()
    }

    pub async fn extract_images_from_pages<T: AsRef<Path>>(
        &mut self,
        output_dir: T,
    ) -> Result<Vec<PathBuf>, ImageExtractionError> {
        let interpreter_settings = InterpreterSettings::default();
        let mut files = Vec::new();

        for (idx, page) in self.document.pages().iter().enumerate() {
            let svg = convert(page, &interpreter_settings);
            let path = output_dir.as_ref().join(format!("{}_{idx}.svg", self.stem));

            match tokio::fs::write(path.clone(), svg).await {
                Err(e) => return Err(ImageExtractionError::Writing(e)),
                Ok(_) => files.push(path),
            }
        }

        Ok(files)
    }
}
