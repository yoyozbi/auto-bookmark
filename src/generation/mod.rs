#[cfg(feature = "ssr")]
use extract_pdf_pages::split_pages_from_input_pdfs;

#[cfg(feature = "ssr")]
use generate_pdf::generate_pdf;

#[cfg(feature = "ssr")]
use itertools::Itertools;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Shared type used by both Typst and HTML generation
#[cfg(feature = "ssr")]
#[derive(Clone, Debug)]
pub struct RectoVersoImagePair {
    pub recto_path: String,
    pub verso_path: String,
}

#[cfg(feature = "ssr")]
mod extract_pdf_pages;

#[cfg(feature = "ssr")]
mod generate_pdf;

#[cfg(feature = "ssr")]
mod generate_pdf_html;

#[cfg(feature = "ssr")]
pub mod calibration;

// Re-export PageMargins and GridConfig for use in calibration
#[cfg(feature = "ssr")]
pub use generate_pdf::{PageMargins, GridConfig};

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
    status: GenerationStatus,
    #[cfg(feature = "ssr")]
    generated_data: Option<Vec<u8>>,
    pub calibration_horizontal_cm: f64,
    pub calibration_vertical_cm: f64,
}

impl Default for GenerationRequest {
    fn default() -> Self {
        GenerationRequest {
            id: Uuid::new_v4(),
            input_files: Vec::new(),
            status: GenerationStatus::Pending,
            #[cfg(feature = "ssr")]
            generated_data: None,
            calibration_horizontal_cm: 0.0,
            calibration_vertical_cm: 0.0,
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
    
    pub fn set_calibration(&mut self, horizontal_cm: f64, vertical_cm: f64) {
        self.calibration_horizontal_cm = horizontal_cm;
        self.calibration_vertical_cm = vertical_cm;
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
        
        // Check if we should use HTML generation (env var USE_HTML_PDF=true)
        let use_html = std::env::var("USE_HTML_PDF")
            .unwrap_or_else(|_| "false".to_string())
            .to_lowercase() == "true";
        
        let pdf = if use_html {
            println!("Using HTML/CSS for PDF generation with calibration: h={}, v={}", 
                     self.calibration_horizontal_cm, self.calibration_vertical_cm);
            use generate_pdf_html::generate_pdf_html_with_calibration;
            let margins = PageMargins::default();
            let config = GridConfig::default();
            generate_pdf_html_with_calibration(
                &image_pairs, 
                &margins, 
                &config,
                self.calibration_horizontal_cm,
                self.calibration_vertical_cm
            )
        } else {
            println!("Using Typst for PDF generation");
            generate_pdf(&image_pairs)
        };

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
