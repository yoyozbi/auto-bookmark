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
#[cfg(feature = "ssr")]
use tokio::fs;

#[cfg(feature = "ssr")]
use axum::response::IntoResponse;

// Handler for multiple file upload
#[cfg(feature = "ssr")]
#[axum::debug_handler]
pub async fn upload_file(mut multipart: Multipart) -> impl IntoResponse {
    use std::error::Error;

    let mut file_paths = Vec::new();
    let mut failed_files = Vec::new();

    // Create uploads directory if it doesn't exist
    println!("Creating directory");
    fs::create_dir_all("uploads").await.map_err(|e| {
        println!("Error {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    println!("Getting fields");
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        println!("Error next field: {:#?}", e.source());
        StatusCode::BAD_REQUEST
    })? {
        println!("field: {:?}", field.name());
        println!("Headers: {:#?}", field.headers());
        println!("Content-type: {:?}", field.content_type());
        if field.name() != Some("files") {
            println!("skipped: {:?}", field.name());
            //continue;
        }

        let file_name = match field.file_name() {
            Some(name) => name.to_string(),
            None => {
                println!("Unknown file");
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

                println!("Writing {}", file_path);

                match fs::write(&file_path, &data).await {
                    Ok(_) => {
                        file_paths.push(file_path.clone());
                    }
                    Err(e) => {
                        println!("Failed to write {}", e);
                        failed_files.push(file_name);
                    }
                }
            }
            Err(_) => {
                println!("No data");
                failed_files.push(file_name);
            }
        }
    }

    println!("Failed files: {:#?}", failed_files);

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
