use leptos::prelude::*;
use leptos::reactive::spawn_local;
use uuid::Uuid;

use crate::generation::GenerationStatus;
use crate::i18n::*;
use crate::utils::api::ApiClient;
use leptos_i18n::I18nContext;

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
        let i18n = use_i18n();
        let locale = i18n.get_locale();
        self.set_generation_status.set(Some(status.clone()));

        match status {
            GenerationStatus::Pending => {
                self.set_status.set(format!("{}", td!(locale, status_waiting_to_start)));
            }
            GenerationStatus::Generating => {
                self.set_status.set(format!("{}", td!(locale, status_generating_pdf)));
            }
            GenerationStatus::Success => {
                self.set_status
                    .set(format!("{}", td!(locale, status_pdf_generated_successfully)));
                self.set_download_url
                    .set(ApiClient::get_download_url(request_id));
                self.set_show_download.set(true);
                self.set_uploading.set(false);
            }
            GenerationStatus::Failure(error) => {
                self.set_status
                    .set(format!("{}", td!(locale, status_generation_failed, error = error)));
                self.set_uploading.set(false);
            }
        }
    }

    /// Sets an error status message
    pub fn set_error(&self, message: &str) {
        let i18n = use_i18n();
        let locale = i18n.get_locale();
        self.set_status.set(format!("{}", td!(locale, status_error, message = message)));
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
pub fn get_status_text(status: &GenerationStatus, i18n: &I18nContext<Locale>) -> String {
    let locale = i18n.get_locale();
    match status {
        GenerationStatus::Pending => format!("⏳ {}", td!(locale, status_pending)),
        GenerationStatus::Generating => format!("⚙️ {}", td!(locale, status_generating)),
        GenerationStatus::Success => format!("✅ {}", td!(locale, status_success)),
        GenerationStatus::Failure(_) => format!("❌ {}", td!(locale, status_failed)),
    }
}
