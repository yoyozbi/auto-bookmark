use gloo_net::http::Request;
use leptos::leptos_dom::logging::console_log;
use leptos::prelude::*;
use uuid::Uuid;
use web_sys::{FormData, HtmlInputElement};

use crate::generation::{GenerationRequest, GenerationStatus};

pub struct ApiClient;

impl ApiClient {
    /// Creates a new upload request
    pub async fn create_upload_request() -> Result<Uuid, String> {
        Self::create_upload_request_with_offsets(0.0, 0.0).await
    }

    /// Creates a new upload request with offset parameters
    pub async fn create_upload_request_with_offsets(
        top_offset: f64,
        left_offset: f64,
    ) -> Result<Uuid, String> {
        console_log("Creating upload request...");

        let form_data = FormData::new().unwrap();
        form_data
            .append_with_str("top_offset", &top_offset.to_string())
            .unwrap();
        form_data
            .append_with_str("left_offset", &left_offset.to_string())
            .unwrap();

        let req = Request::post("/api/upload").body(form_data);

        let req = match req {
            Ok(request) => request.send().await,
            Err(_) => return Err("Failed to build request".to_string()),
        };

        match req {
            Ok(response) if response.ok() => match response.json::<GenerationRequest>().await {
                Ok(request) => {
                    console_log(&format!("Created request: {}", request.id));
                    Ok(request.id)
                }
                Err(_) => Err("Failed to parse upload response".to_string()),
            },
            _ => Err("Failed to create upload request".to_string()),
        }
    }

    /// Uploads a single file to an existing request
    pub async fn upload_file(request_id: Uuid, file: web_sys::File) -> Result<(), String> {
        let form_data = FormData::new().unwrap();
        form_data.append_with_blob("file", &file).unwrap();

        console_log(&format!("Uploading file: {}", file.name()));

        let upload_req = Request::post(&format!("/api/upload/{}/file", request_id)).body(form_data);

        let upload_req = match upload_req {
            Ok(request) => request.send().await,
            Err(_) => return Err("Failed to build upload request".to_string()),
        };

        match upload_req {
            Ok(response) if response.ok() => {
                console_log(&format!("Successfully uploaded file: {}", file.name()));
                Ok(())
            }
            Ok(response) => {
                let body = response.text().await.unwrap_or_default();
                if body.is_empty() {
                    Err(format!("Failed to upload file: {}", file.name()))
                } else {
                    Err(format!("{}: {}", file.name(), body))
                }
            }
            Err(_) => Err(format!("Failed to upload file: {}", file.name())),
        }
    }

    /// Starts PDF generation for a request
    pub async fn start_generation(request_id: Uuid) -> Result<(), String> {
        let generate_req = Request::post(&format!("/api/upload/{}/generate", request_id))
            .send()
            .await;

        match generate_req {
            Ok(response) if response.status() == 202 => {
                console_log("PDF generation started");
                Ok(())
            }
            _ => Err("Failed to start PDF generation".to_string()),
        }
    }

    /// Checks the status of a generation request
    pub async fn check_status(request_id: Uuid) -> Result<GenerationStatus, String> {
        let response = Request::get(&format!("/api/upload/{}/status", request_id))
            .send()
            .await;

        match response {
            Ok(resp) if resp.ok() => match resp.json::<GenerationStatus>().await {
                Ok(status) => Ok(status),
                Err(_) => Err("Failed to parse status response".to_string()),
            },
            _ => Err("Failed to check status".to_string()),
        }
    }

    /// Cleans up a request from the server
    pub async fn cleanup_request(request_id: Uuid) -> Result<(), String> {
        let cleanup_req = Request::delete(&format!("/api/upload/{}", request_id))
            .send()
            .await;

        match cleanup_req {
            Ok(response) if response.ok() => {
                console_log("Request cleaned up");
                Ok(())
            }
            _ => Err("Failed to cleanup request".to_string()),
        }
    }

    /// Gets the download URL for a completed request
    pub fn get_download_url(request_id: Uuid) -> String {
        format!("/api/upload/{}/download", request_id)
    }
}

/// Helper function to get files from file input element
pub fn get_files_from_input(
    input_ref: &NodeRef<leptos::html::Input>,
) -> Result<web_sys::FileList, String> {
    let input = input_ref.get_untracked().ok_or("File input not found")?;

    let input_el: HtmlInputElement = input;
    let files = input_el.files().ok_or("No files available")?;

    if files.length() == 0 {
        return Err("No files selected".to_string());
    }

    Ok(files)
}

/// Helper function to clear file input
pub fn clear_file_input(input_ref: &NodeRef<leptos::html::Input>) {
    if let Some(input) = input_ref.get_untracked() {
        let input_el: HtmlInputElement = input;
        input_el.set_value("");
    }
}
