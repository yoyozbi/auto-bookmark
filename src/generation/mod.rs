use extract_pdf_pages::split_pages_from_input_pdfs;
use generate_pdf::generate_pdf;
use itertools::Itertools;
use std::path::Path;
use uuid::Uuid;

mod extract_pdf_pages;
mod generate_pdf;

pub struct GenerationRequest<P1: AsRef<Path> + Sync + Sized + Clone + Send> {
    pub id: Uuid,
    pub input_files: Vec<P1>,
}

impl<P1: AsRef<Path> + Sync + Sized + Clone + Send> GenerationRequest<P1> {
    pub fn new(input_files: Vec<P1>) -> GenerationRequest<P1> {
        GenerationRequest {
            id: Uuid::new_v4(),
            input_files,
        }
    }

    pub async fn generate_pdf(
        &mut self,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        println!("Generating pdfs");
        let image_pairs = split_pages_from_input_pdfs(&self.input_files, self.id).await;
        let image_pairs = match image_pairs {
            Ok(pairs) => pairs,
            Err(_e) => {
                self.delete_files(None).await?;
                return Err(format!("Failed to create image pairs: {}", _e).into());
            }
        };
        let pdf = generate_pdf(&image_pairs);

        let images = image_pairs
            .iter()
            .map(|f| vec![f.recto_path.clone(), f.verso_path.clone()])
            .flatten()
            .collect_vec();

        self.delete_files(Some(images)).await?;

        match pdf {
            Ok(data) => Ok(data),
            Err(_e) => Err(format!("Failed to generate PDF: {}", _e).into()),
        }
    }

    async fn delete_files(
        &mut self,
        additional_file_paths: Option<Vec<String>>,
    ) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
        let mut all_files: Vec<String> = self
            .input_files
            .iter()
            .map(|p| p.as_ref().to_string_lossy().to_string())
            .collect();

        if let Some(mut files) = additional_file_paths {
            all_files.append(&mut files);
        }

        for file in all_files.iter() {
            tokio::fs::remove_file(file)
                .await
                .map_err(|e| format!("Error deleting file: {}", e))?;
        }
        Ok(())
    }
}
