use std::{
    iter::Map,
    path::{Path, PathBuf},
};

#[cfg(feature = "ssr")]
use crate::generation::GenerationRequest;
#[cfg(feature = "ssr")]
use axum::{
    extract::Multipart,
    http::StatusCode,
    http::{header, HeaderMap},
    routing::post,
    Router,
};
use leptos::config::LeptosOptions;
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use tokio::fs;

// Handler for multiple file upload
#[cfg(feature = "ssr")]
pub async fn upload_file(mut multipart: Multipart) -> Result<(HeaderMap, Vec<u8>), StatusCode> {
    let mut file_paths = Vec::new();
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
                        file_paths.push(file_path.clone());
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

    let mut request = GenerationRequest::new(file_paths);

    return match request.generate_pdf().await {
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
