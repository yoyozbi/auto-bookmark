use std::{fmt::Error, path::Path};

use extract_pdf_pages::split_pages_from_input_pdfs;
use generate_pdf::generate_pdf;
use uuid::Uuid;

mod extract_pdf_pages;
mod generate_pdf;

pub struct GenerationRequest<P1: AsRef<Path> + Sync + Sized> {
    pub id: Uuid,
    pub input_files: Vec<P1>,
}

impl<P1: AsRef<Path> + Sync + Sized> GenerationRequest<P1> {
    pub fn new(input_files: Vec<P1>) -> GenerationRequest<P1> {
        GenerationRequest {
            id: Uuid::new_v4(),
            input_files,
        }
    }

    pub async fn generate_pdf(&mut self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let image_pairs = split_pages_from_input_pdfs(&self.input_files, self.id).await;
        let image_pairs = match image_pairs {
            Ok(pairs) => pairs,
            Err(_e) => {
                self.delete_files().await?;
                return Err(format!("Failed to create image pairs: {}", _e).into());
            }
        };
        let pdf = generate_pdf(&image_pairs);
        match pdf {
            Ok(data) => Ok(data),
            Err(_e) => {
                self.delete_files().await?;

                Err(format!("Failed to generate PDF: {}", _e).into())
            }
        }
    }

    async fn delete_files(
        &mut self,
        additionalFilePaths: Option<Vec<P1>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut all_files = self.input_files;

        if let Some(files) = additionalFilePaths {
            all_files.append(files);
        }

        for file in all_files.iter() {
            tokio::fs::remove_file(file)
                .await
                .map_err(|e| format!("Error deleting file: {}", e))?;
        }
        Ok(())
    }
}
