use leptos::prelude::*;
use leptos::reactive::spawn_local;
use uuid::Uuid;

use crate::generation::GenerationStatus;
use crate::utils::api::ApiClient;

pub struct StatusHandler {
    pub set_status: WriteSignal<String>,
    pub set_generation_status: WriteSignal<Option<GenerationStatus>>,
    pub set_download_url: WriteSignal<String>,
    pub set_show_download: WriteSignal<bool>,
    pub set_uploading: WriteSignal<bool>,
}

impl StatusHandler {
    pub fn new(
        set_status: WriteSignal<String>,
        set_generation_status: WriteSignal<Option<GenerationStatus>>,
        set_download_url: WriteSignal<String>,
        set_show_download: WriteSignal<bool>,
        set_uploading: WriteSignal<bool>,
    ) -> Self {
        Self {
            set_status,
            set_generation_status,
            set_download_url,
            set_show_download,
            set_uploading,
        }
    }

    /// Updates UI based on generation status
    pub fn handle_generation_status(&self, status: GenerationStatus, request_id: Uuid) {
        self.set_generation_status.set(Some(status.clone()));

        match status {
            GenerationStatus::Pending => {
                self.set_status
                    .set("Status: Waiting to start generation...".to_string());
            }
            GenerationStatus::Generating => {
                self.set_status.set("Status: Generating PDF...".to_string());
            }
            GenerationStatus::Success => {
                self.set_status
                    .set("Status: PDF generated successfully!".to_string());
                self.set_download_url
                    .set(ApiClient::get_download_url(request_id));
                self.set_show_download.set(true);
                self.set_uploading.set(false);
            }
            GenerationStatus::Failure(error) => {
                self.set_status
                    .set(format!("Status: Generation failed: {}", error));
                self.set_uploading.set(false);
            }
        }
    }

    /// Sets an error status message
    pub fn set_error(&self, message: &str) {
        self.set_status.set(format!("Error: {}", message));
        self.set_uploading.set(false);
    }

    /// Sets an info status message
    pub fn set_info(&self, message: &str) {
        self.set_status.set(message.to_string());
    }

    /// Resets all status signals to initial state
    pub fn reset(&self) {
        self.set_status.set(String::new());
        self.set_generation_status.set(None);
        self.set_download_url.set(String::new());
        self.set_show_download.set(false);
        self.set_uploading.set(false);
    }

    /// Checks status once manually
    pub fn check_status_once(&self, request_id: Uuid) {
        let handler = self.clone();
        spawn_local(async move {
            match ApiClient::check_status(request_id).await {
                Ok(status) => {
                    handler.handle_generation_status(status, request_id);
                }
                Err(error) => {
                    handler.set_error(&error);
                }
            }
        });
    }
}

impl Clone for StatusHandler {
    fn clone(&self) -> Self {
        Self {
            set_status: self.set_status,
            set_generation_status: self.set_generation_status,
            set_download_url: self.set_download_url,
            set_show_download: self.set_show_download,
            set_uploading: self.set_uploading,
        }
    }
}

/// Helper function to get status indicator class based on status
pub fn get_status_indicator_class(status: &GenerationStatus) -> &'static str {
    match status {
        GenerationStatus::Pending => "status-indicator pending",
        GenerationStatus::Generating => "status-indicator generating",
        GenerationStatus::Success => "status-indicator success",
        GenerationStatus::Failure(_) => "status-indicator failure",
    }
}

/// Helper function to get status text for display
pub fn get_status_text(status: &GenerationStatus) -> String {
    match status {
        GenerationStatus::Pending => "⏳ Pending".to_string(),
        GenerationStatus::Generating => "⚙️ Generating".to_string(),
        GenerationStatus::Success => "✅ Success".to_string(),
        GenerationStatus::Failure(_) => "❌ Failed".to_string(),
    }
}
