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

    pub async fn generate_pdf(&mut self) -> Result<Vec<u8>, Error> {
        let image_pairs = split_pages_from_input_pdfs(&self.input_files, self.id).await;
        let image_pairs = match image_pairs {
            Ok(pairs) => pairs,
            Err(_e) => return Err(Error),
        };
        let pdf = generate_pdf(&image_pairs);
        match pdf {
            Ok(data) => Ok(data),
            Err(_e) => Err(Error),
        }
    }
}
