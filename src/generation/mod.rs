use extract_pdf_pages::split_pages_from_input_pdfs;
use generate_pdf::generate_pdf;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

mod extract_pdf_pages;
mod generate_pdf;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct GenerationRequest {
    pub id: Uuid,
    input_files: Vec<String>,
}

impl Default for GenerationRequest {
    fn default() -> Self {
        GenerationRequest {
            id: Uuid::new_v4(),
            input_files: Vec::new(),
        }
    }
}

impl GenerationRequest {
    pub fn add_file(&mut self, file: String) {
        self.input_files.push(file);
    }

    pub async fn generate_pdf(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        println!(
            "Generating pdfs with {} input files",
            self.input_files.len()
        );

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
        &self,
        additional_file_paths: Option<Vec<String>>,
    ) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
        let mut all_files: Vec<String> = self.input_files.clone();

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
