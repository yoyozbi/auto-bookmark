use leptos::prelude::*;
use leptos::reactive::spawn_local;
use uuid::Uuid;
use web_sys::FileList;

use crate::utils::api::ApiClient;
use crate::utils::status_handler::StatusHandler;

pub struct UploadWorkflow {
    status_handler: StatusHandler,
    set_current_request_id: WriteSignal<Option<Uuid>>,
}

impl UploadWorkflow {
    pub fn new(
        status_handler: StatusHandler,
        set_current_request_id: WriteSignal<Option<Uuid>>,
    ) -> Self {
        Self {
            status_handler,
            set_current_request_id,
        }
    }

    /// Executes the complete upload and generation workflow
    pub fn execute(&self, files: FileList) {
        self.execute_with_calibration(files, 0.0, 0.0);
    }
    
    /// Executes the complete upload and generation workflow with calibration
    pub fn execute_with_calibration(&self, files: FileList, horizontal_cm: f64, vertical_cm: f64) {
        let status_handler = self.status_handler.clone();
        let set_current_request_id = self.set_current_request_id;

        spawn_local(async move {
            // Initialize workflow
            status_handler.set_uploading.set(true);
            status_handler.set_show_download.set(false);
            status_handler.set_generation_status.set(None);
            status_handler.set_info("Starting upload...");

            // Step 1: Create upload request
            let request_id = match ApiClient::create_upload_request().await {
                Ok(id) => {
                    set_current_request_id.set(Some(id));
                    id
                }
                Err(error) => {
                    status_handler.set_error(&error);
                    return;
                }
            };

            // Step 2: Upload each file
            let file_count = files.length();
            for i in 0..file_count {
                if let Some(file) = files.get(i) {
                    status_handler.set_info(&format!("Uploading files: {}/{}", i + 1, file_count));

                    if let Err(error) = ApiClient::upload_file(request_id, file).await {
                        status_handler.set_error(&error);
                        return;
                    }
                }
            }
            
            // Step 2.5: Set calibration offsets if non-zero
            if horizontal_cm != 0.0 || vertical_cm != 0.0 {
                status_handler.set_info("Setting calibration...");
                if let Err(error) = ApiClient::set_calibration(request_id, horizontal_cm, vertical_cm).await {
                    status_handler.set_error(&error);
                    return;
                }
            }

            // Step 3: Start PDF generation
            status_handler.set_info("Starting PDF generation...");
            match ApiClient::start_generation(request_id).await {
                Ok(()) => {
                    status_handler.set_info("PDF generation started. Checking status...");
                    set_current_request_id.set(Some(request_id));

                    // Set initial status to trigger automatic polling
                    status_handler
                        .set_generation_status
                        .set(Some(crate::generation::GenerationStatus::Pending));
                    status_handler.set_uploading.set(false);
                }
                Err(error) => {
                    status_handler.set_error(&error);
                }
            }
        });
    }

    /// Creates a cleanup handler closure
    pub fn create_cleanup_handler(
        &self,
        current_request_id: ReadSignal<Option<Uuid>>,
    ) -> impl Fn(leptos::ev::MouseEvent) + Clone {
        let status_handler = self.status_handler.clone();
        let set_current_request_id = self.set_current_request_id;

        move |_| {
            if let Some(request_id) = current_request_id.get() {
                let status_handler = status_handler.clone();
                spawn_local(async move {
                    match ApiClient::cleanup_request(request_id).await {
                        Ok(()) => {
                            set_current_request_id.set(None);
                            status_handler.set_show_download.set(false);
                            status_handler.set_generation_status.set(None);
                            status_handler.set_info("Request cleaned up");
                        }
                        Err(error) => {
                            status_handler.set_error(&error);
                        }
                    }
                });
            }
        }
    }

    /// Creates a reset handler closure
    pub fn create_reset_handler(
        &self,
        file_input_ref: NodeRef<leptos::html::Input>,
    ) -> impl Fn(leptos::ev::MouseEvent) + Clone {
        let status_handler = self.status_handler.clone();
        let set_current_request_id = self.set_current_request_id;

        move |_| {
            status_handler.reset();
            set_current_request_id.set(None);
            crate::utils::api::clear_file_input(&file_input_ref);
        }
    }
}

impl Clone for UploadWorkflow {
    fn clone(&self) -> Self {
        Self {
            status_handler: self.status_handler.clone(),
            set_current_request_id: self.set_current_request_id,
        }
    }
}

/// Validates files before upload
pub fn validate_files(files: &FileList) -> Result<(), String> {
    if files.length() == 0 {
        return Err("No files selected".to_string());
    }

    // Check each file type
    for i in 0..files.length() {
        if let Some(file) = files.get(i) {
            if !file.type_().starts_with("application/pdf") {
                return Err(format!(
                    "File '{}' is not a PDF file. Only PDF files are allowed.",
                    file.name()
                ));
            }

            // Check file size (limit to 50MB per file)
            const MAX_FILE_SIZE: f64 = 50.0 * 1024.0 * 1024.0; // 50MB
            if file.size() > MAX_FILE_SIZE {
                return Err(format!(
                    "File '{}' is too large. Maximum file size is 50MB.",
                    file.name()
                ));
            }
        }
    }

    Ok(())
}
