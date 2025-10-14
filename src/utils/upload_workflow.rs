use leptos::prelude::*;
use leptos::reactive::spawn_local;
use uuid::Uuid;
use web_sys::FileList;

use crate::i18n::*;
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
        self.execute_with_offsets(files, 0.0, 0.0);
    }

    /// Executes the complete upload and generation workflow with offset parameters
    pub fn execute_with_offsets(&self, files: FileList, top_offset: f64, left_offset: f64) {
        self.execute_with_offsets_and_polling(files, top_offset, left_offset, |_| {});
    }

    /// Executes the complete upload and generation workflow with offset parameters and polling callback
    pub fn execute_with_offsets_and_polling<F>(
        &self,
        files: FileList,
        top_offset: f64,
        left_offset: f64,
        start_polling: F,
    ) where
        F: Fn(Uuid) + Clone + 'static,
    {
        let status_handler = self.status_handler.clone();
        let set_current_request_id = self.set_current_request_id;
        let start_polling = start_polling; // Explicitly capture for move

        spawn_local(async move {
            let i18n = use_i18n();
            
            // Initialize workflow
            status_handler.set_uploading.set(true);
            status_handler.set_show_download.set(false);
            status_handler.set_generation_status.set(None);
            status_handler.set_info(&td!(i18n, workflow_starting_upload));

            // Step 1: Create upload request
            let request_id = match ApiClient::create_upload_request_with_offsets(
                top_offset,
                left_offset,
            )
            .await
            {
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
                    status_handler.set_info(&td!(i18n, workflow_uploading_files, current = (i + 1) as i32, total = file_count as i32));

                    if let Err(error) = ApiClient::upload_file(request_id, file).await {
                        status_handler.set_error(&error);
                        return;
                    }
                }
            }

            // Step 3: Start PDF generation
            status_handler.set_info(&td!(i18n, workflow_starting_pdf_generation));
            match ApiClient::start_generation(request_id).await {
                Ok(()) => {
                    status_handler.set_info(&td!(i18n, workflow_pdf_generation_started));
                    set_current_request_id.set(Some(request_id));

                    // Set initial status to trigger automatic polling
                    status_handler
                        .set_generation_status
                        .set(Some(crate::generation::GenerationStatus::Pending));
                    status_handler.set_uploading.set(false);

                    // Start auto-polling
                    start_polling(request_id);
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
                    let i18n = use_i18n();
                    match ApiClient::cleanup_request(request_id).await {
                        Ok(()) => {
                            set_current_request_id.set(None);
                            status_handler.set_show_download.set(false);
                            status_handler.set_generation_status.set(None);
                            status_handler.set_info(&td!(i18n, workflow_request_cleaned_up));
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
    let i18n = use_i18n();
    
    if files.length() == 0 {
        return Err(td!(i18n, validation_no_files_selected));
    }

    // Check each file type
    for i in 0..files.length() {
        if let Some(file) = files.get(i) {
            if !file.type_().starts_with("application/pdf") {
                let filename = file.name();
                return Err(td!(i18n, validation_not_pdf_file, filename = filename));
            }

            // Check file size (limit to 50MB per file)
            const MAX_FILE_SIZE: f64 = 50.0 * 1024.0 * 1024.0; // 50MB
            if file.size() > MAX_FILE_SIZE {
                let filename = file.name();
                return Err(td!(i18n, validation_file_too_large, filename = filename));
            }
        }
    }

    Ok(())
}
