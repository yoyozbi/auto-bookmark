#[cfg(feature = "ssr")]
use crate::generation::generate_pdf_from_input_pdfs;
#[cfg(feature = "ssr")]
use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    http::{header, HeaderMap},
    response::Json,
    routing::post,
    Router,
};
use leptos::config::LeptosOptions;
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use tokio::fs;

#[derive(Serialize, Deserialize)]
pub struct UploadResponse {
    pub success: bool,
    pub message: String,
    pub uploaded_files: Vec<UploadedFile>,
    pub failed_files: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct UploadedFile {
    pub original_name: String,
    pub file_path: String,
    pub size: usize,
}

// Handler for multiple file upload
#[cfg(feature = "ssr")]
pub async fn upload_file(mut multipart: Multipart) -> Result<(HeaderMap, Vec<u8>), StatusCode> {
    let mut uploaded_files = Vec::new();
    let mut failed_files = Vec::new();

    // Create uploads directory if it doesn't exist
    fs::create_dir_all("uploads")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        let file_name = match field.file_name() {
            Some(name) => name.to_string(),
            None => {
                failed_files.push("Unknown file (no filename)".to_string());
                continue;
            }
        };

        match field.bytes().await {
            Ok(data) => {
                // Generate unique filename with timestamp to avoid conflicts
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis();
                let file_path = format!("uploads/{}_{}", timestamp, file_name);

                match fs::write(&file_path, &data).await {
                    Ok(_) => {
                        uploaded_files.push(UploadedFile {
                            original_name: file_name,
                            file_path: file_path.clone(),
                            size: data.len(),
                        });
                    }
                    Err(_) => {
                        failed_files.push(file_name);
                    }
                }
            }
            Err(_) => {
                failed_files.push(file_name);
            }
        }
    }

    let success = !uploaded_files.is_empty();
    let file_paths: Vec<String> = uploaded_files.iter().map(|f| f.file_path.clone()).collect();

    return match generate_pdf_from_input_pdfs(&file_paths) {
        Ok(data) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                "application/pdf; charset=utf-8".parse().unwrap(),
            );
            headers.insert(
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"Cargo.pdf\"".parse().unwrap(),
            );
            Ok((headers, data))
        }
        Err(err) => {
            println!("Failed to generate pdf: {}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    };
}

#[cfg(feature = "ssr")]
// Add this route to your Axum router
pub fn file_upload_routes() -> Router<LeptosOptions> {
    Router::new().route("/api/upload", post(upload_file))
}
