use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    Downloaded,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct GenerationRequest {
    pub id: Uuid,
    #[serde(skip_serializing, default)]
    pub input_files: Vec<String>,
    pub top_offset: f64,
    pub left_offset: f64,
    status: GenerationStatus,

    pub created_at: std::time::SystemTime,

    #[serde(skip_serializing, default)]
    #[cfg(feature = "ssr")]
    generated_data: Option<Vec<u8>>,

    #[serde(skip_serializing, default)]
    #[cfg(feature = "ssr")]
    pub downloaded_at: Option<std::time::SystemTime>,
}

impl Default for GenerationRequest {
    fn default() -> Self {
        GenerationRequest {
            id: Uuid::new_v4(),
            input_files: Vec::new(),
            top_offset: 0.0,
            left_offset: 0.0,
            status: GenerationStatus::Pending,
            created_at: std::time::SystemTime::now(),
            #[cfg(feature = "ssr")]
            generated_data: None,
            #[cfg(feature = "ssr")]
            downloaded_at: None,
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

    pub fn set_downloaded_now(&mut self) {
        self.downloaded_at = Some(std::time::SystemTime::now());
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
        use crate::generation::generate_pdf::{GridConfig, PageMargins, PdfPagePair, generate_pdf_with_config};

        if self.input_files.is_empty() {
            return Err("No files provided".into());
        }

        let page_pairs: Vec<PdfPagePair> = self.input_files.iter()
            .map(|path| PdfPagePair {
                pdf_path: path.clone(),
                recto_page: 1,
                verso_page: 2,
            })
            .collect();

        let margins_config = PageMargins::default();
        let grid_config = GridConfig {
            left_offset: self.left_offset,
            top_offset: self.top_offset,
            ..Default::default()
        };
        let result = generate_pdf_with_config(&page_pairs, &margins_config, &grid_config);

        self.delete_files().await?;

        match result {
            Ok(data) => Ok(data),
            Err(e) => Err(format!("Failed to generate PDF: {}", e).into()),
        }
    }

    pub async fn delete_files(&self) -> Result<(), Box<dyn std::error::Error + Sync + Send>> {
        for file in self.input_files.iter() {
            tokio::fs::remove_file(file)
                .await
                .map_err(|e| format!("Error deleting file ({}): {}", file, e))?;
        }
        Ok(())
    }
}
