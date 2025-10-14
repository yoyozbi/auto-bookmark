use leptos::html::Input;

use leptos::prelude::*;
use leptos::reactive::spawn_local;
use uuid::Uuid;

use crate::generation::GenerationStatus;
use crate::i18n::*;
use crate::utils::{
    api::get_files_from_input,
    status_handler::{StatusHandler, get_status_indicator_class, get_status_text},
    upload_workflow::{UploadWorkflow, validate_files},
};

#[component]
pub fn FileUpload() -> impl IntoView {
    let i18n = use_i18n();
    
    // Signals for component state
    let (status, set_status) = signal(String::new());
    let (uploading, set_uploading) = signal(false);
    let (current_request_id, set_current_request_id) = signal::<Option<Uuid>>(None);
    let (download_url, set_download_url) = signal(String::new());
    let (show_download, set_show_download) = signal(false);
    let (generation_status, set_generation_status) = signal::<Option<GenerationStatus>>(None);
    let (auto_polling, set_auto_polling) = signal(false);
    let (top_offset, set_top_offset) = signal(0.0);
    let (left_offset, set_left_offset) = signal(0.0);

    let file_input: NodeRef<Input> = NodeRef::new();

    // Initialize handlers
    let status_handler = StatusHandler::new(
        set_status,
        set_generation_status,
        set_download_url,
        set_show_download,
        set_uploading,
    );

    let upload_workflow = UploadWorkflow::new(status_handler.clone(), set_current_request_id);

    // Start auto-polling function
    let start_auto_polling = {
        let status_handler = status_handler.clone();
        move |request_id: Uuid| {
            set_auto_polling.set(true);

            // Clone signals for the polling task
            let status_handler = status_handler.clone();
            let current_request_id = current_request_id;
            let generation_status = generation_status;
            let set_auto_polling = set_auto_polling;

            spawn_local(async move {
                // Wait 3 seconds before first check
                gloo_timers::future::sleep(std::time::Duration::from_secs(3)).await;

                loop {
                    // Check if we should still be polling
                    if current_request_id.get_untracked() != Some(request_id) {
                        set_auto_polling.set(false);
                        break;
                    }

                    let current_status = generation_status.get_untracked();
                    if !matches!(
                        current_status,
                        Some(GenerationStatus::Pending) | Some(GenerationStatus::Generating)
                    ) {
                        set_auto_polling.set(false);
                        break;
                    }

                    // Check status
                    match crate::utils::api::ApiClient::check_status(request_id).await {
                        Ok(status) => {
                            status_handler.handle_generation_status(status.clone(), request_id);

                            // Stop polling if completed
                            if matches!(
                                status,
                                GenerationStatus::Success | GenerationStatus::Failure(_)
                            ) {
                                set_auto_polling.set(false);
                                break;
                            }
                        }
                        Err(error) => {
                            status_handler.set_error(&error);
                            set_auto_polling.set(false);
                            break;
                        }
                    }

                    // Wait 10 seconds before next check
                    gloo_timers::future::sleep(std::time::Duration::from_secs(10)).await;
                }
            });
        }
    };

    // Event handlers
    let upload_handler = {
        let workflow = upload_workflow.clone();
        let status_handler = status_handler.clone();
        move |_| match get_files_from_input(&file_input) {
            Ok(files) => {
                if let Err(error) = validate_files(&files) {
                    status_handler.set_error(&error);
                    return;
                }
                workflow.execute_with_offsets_and_polling(
                    files,
                    top_offset.get(),
                    left_offset.get(),
                    start_auto_polling.clone(),
                );
            }
            Err(error) => {
                status_handler.set_error(&error);
            }
        }
    };

    let cleanup_handler = {
        let status_handler = status_handler.clone();
        move |_| {
            if let Some(request_id) = current_request_id.get() {
                set_auto_polling.set(false);
                let status_handler = status_handler.clone();
                leptos::reactive::spawn_local(async move {
                    match crate::utils::api::ApiClient::cleanup_request(request_id).await {
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
    };

    let reset_handler = {
        let status_handler = status_handler.clone();
        move |_| {
            set_auto_polling.set(false);
            status_handler.reset();
            set_current_request_id.set(None);
            crate::utils::api::clear_file_input(&file_input);
        }
    };

    view! {
        <div class="file-upload-container">
            <h3>{t!(i18n, file_upload_title)}</h3>

            <div class="upload-section">
                <input
                    node_ref=file_input
                    type="file"
                    multiple=true
                    accept=".pdf"
                    disabled={move || uploading.get()}
                />

                <div class="offset-controls">
                    <div class="offset-input">
                        <label for="top-offset">{t!(i18n, file_upload_top_offset_label)}</label>
                        <input
                            id="top-offset"
                            type="number"
                            step="0.1"
                            min="0"
                            value={move || top_offset.get()}
                            on:input=move |ev| {
                                if let Ok(value) = event_target_value(&ev).parse::<f64>() {
                                    set_top_offset.set(value);
                                }
                            }
                            disabled={move || uploading.get()}
                        />
                    </div>
                    <div class="offset-input">
                        <label for="left-offset">{t!(i18n, file_upload_left_offset_label)}</label>
                        <input
                            id="left-offset"
                            type="number"
                            step="0.1"
                            min="0"
                            value={move || left_offset.get()}
                            on:input=move |ev| {
                                if let Ok(value) = event_target_value(&ev).parse::<f64>() {
                                    set_left_offset.set(value);
                                }
                            }
                            disabled={move || uploading.get()}
                        />
                    </div>
                </div>

                <div class="button-group">
                    <button
                        on:click=upload_handler
                        disabled={move || uploading.get() || auto_polling.get()}
                        class="upload-btn"
                    >
                        {move || {
                            if uploading.get() {
                                t!(i18n, file_upload_uploading_files)
                            } else {
                                t!(i18n, file_upload_upload_generate_pdf)
                            }
                        }}
                    </button>

                    <button
                        on:click=reset_handler
                        disabled={move || uploading.get()}
                        class="reset-btn"
                    >
                        {t!(i18n, file_upload_reset)}
                    </button>
                </div>
            </div>

            // Unified status banner
            <div class="status-banner" class:hidden={move || {
                !uploading.get() &&
                !auto_polling.get() &&
                !show_download.get() &&
                status.get().is_empty()
            }}>
                {move || {
                    if uploading.get() {
                        // Upload progress state
                        view! {
                            <div class="banner-content upload-state">
                                <div class="banner-header">
                                    <div class="spinner-ring"></div>
                                    <h4>{t!(i18n, file_upload_uploading_files_title)}</h4>
                                </div>
                                <div class="progress-bar">
                                    <div class="progress-fill"></div>
                                </div>
                                <p class="banner-text">{t!(i18n, file_upload_uploading_files_message)}</p>
                            </div>
                        }.into_any()
                    } else if show_download.get() {
                        // Download ready state
                        let cleanup_handler = cleanup_handler.clone();
                        view! {
                            <div class="banner-content success-state">
                                <div class="banner-header">
                                    <div class="status-indicator success">{format!("✅ {}", t!(i18n, file_upload_success))}</div>
                                    <h4>{t!(i18n, file_upload_pdf_generated_successfully)}</h4>
                                </div>
                                <div class="banner-actions">
                                    <a
                                        href={move || download_url.get()}
                                        target="_blank"
                                        class="download-link"
                                    >
                                        {format!("📄 {}", t!(i18n, file_upload_download_pdf))}
                                    </a>
                                    <button
                                        on:click=cleanup_handler
                                        class="cleanup-btn"
                                    >
                                        {t!(i18n, file_upload_clean_up)}
                                    </button>
                                </div>
                            </div>
                        }.into_any()
                    } else if auto_polling.get() || !status.get().is_empty() {
                        // Processing/polling state
                        view! {
                            <div class="banner-content processing-state">
                                <div class="banner-header">
                                    {move || {
                                        if auto_polling.get() {
                                            view! {
                                                <div class="spinner-ring"></div>
                                            }.into_any()
                                        } else {
                                            view! { <div></div> }.into_any()
                                        }
                                    }}
                                    {move || {
                                        if let Some(gen_status) = generation_status.get() {
                                            let class = get_status_indicator_class(&gen_status);
                                            let text = get_status_text(&gen_status, &i18n);
                                            view! { <div class=class>{text}</div> }.into_any()
                                        } else {
                                            view! { <div></div> }.into_any()
                                        }
                                    }}
                                </div>
                                <p class="banner-text">
                                    {move || status.get()}
                                    {move || {
                                        if auto_polling.get() {
                                            view! {
                                                <span class="auto-polling-text">
                                                    {t!(i18n, file_upload_auto_checking)}
                                                </span>
                                            }.into_any()
                                        } else {
                                            view! { <span></span> }.into_any()
                                        }
                                    }}
                                </p>
                            </div>
                        }.into_any()
                    } else {
                        view! { <div></div> }.into_any()
                    }
                }}
            </div>
        </div>
    }
}
