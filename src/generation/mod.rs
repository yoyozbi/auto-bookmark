#[cfg(feature = "ssr")]
use extract_pdf_pages::split_pages_from_input_pdfs;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "ssr")]
mod extract_pdf_pages;

#[cfg(feature = "ssr")]
mod generate_pdf;

#[cfg(feature = "ssr")]
pub mod validate_margins;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum GenerationStatus {
    Pending,
    Generating,
    Success,
    Failure(String),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct GenerationRequest {
    pub id: Uuid,
    pub input_files: Vec<String>,
    pub top_offset: f64,
    pub left_offset: f64,
    status: GenerationStatus,
    #[cfg(feature = "ssr")]
    generated_data: Option<Vec<u8>>,
}

impl Default for GenerationRequest {
    fn default() -> Self {
        GenerationRequest {
            id: Uuid::new_v4(),
            input_files: Vec::new(),
            top_offset: 0.0,
            left_offset: 0.0,
            status: GenerationStatus::Pending,
            #[cfg(feature = "ssr")]
            generated_data: None,
        }
    }
}

#[cfg(feature = "ssr")]
impl GenerationRequest {
    pub fn add_file(&mut self, file: String) {
        self.input_files.push(file);
    }

    pub fn status(&self) -> GenerationStatus {
        self.status.clone()
    }

    pub fn set_status(&mut self, status: GenerationStatus) {
        self.status = status;
    }

    pub fn get_generated_data(&self) -> Option<&Vec<u8>> {
        self.generated_data.as_ref()
    }

    pub fn set_generated_data(&mut self, data: Vec<u8>) {
        self.generated_data = Some(data);
    }

    pub fn file_count(&self) -> usize {
        self.input_files.len()
    }

    pub async fn generate_pdf(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        use crate::generation::generate_pdf::{GridConfig, PageMargins, generate_pdf_with_config};

        let page_pairs = split_pages_from_input_pdfs(&self.input_files, self.id).await;
        let page_pairs = match page_pairs {
            Ok(pairs) => pairs,
            Err(_e) => {
                self.delete_files(None).await?;
                return Err(format!("Failed to create page pairs: {}", _e).into());
            }
        };
        let margins_config = PageMargins::default();
        let grid_config = GridConfig {
            left_offset: self.left_offset,
            top_offset: self.top_offset,
            ..Default::default()
        };
        let pdf = generate_pdf_with_config(&page_pairs, &margins_config, &grid_config);

        // Clean up the copied PDF files
        let pdf_files: Vec<String> = page_pairs
            .iter()
            .map(|pair| pair.pdf_path.clone())
            .collect::<std::collections::HashSet<_>>() // Remove duplicates
            .into_iter()
            .collect();

        self.delete_files(Some(pdf_files)).await?;

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
                .map_err(|e| format!("Error deleting file ({}): {}", file, e))?;
        }
        Ok(())
    }
}
