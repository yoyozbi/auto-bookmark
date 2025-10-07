use leptos::html::Input;
use leptos::prelude::*;
use leptos::reactive::spawn_local;
use uuid::Uuid;

use crate::generation::GenerationStatus;
use crate::utils::{
    api::get_files_from_input,
    status_handler::{StatusHandler, get_status_indicator_class, get_status_text},
    upload_workflow::{UploadWorkflow, validate_files},
};

#[component]
pub fn FileUpload() -> impl IntoView {
    // Signals for component state
    let (status, set_status) = signal(String::new());
    let (uploading, set_uploading) = signal(false);
    let (current_request_id, set_current_request_id) = signal::<Option<Uuid>>(None);
    let (download_url, set_download_url) = signal(String::new());
    let (show_download, set_show_download) = signal(false);
    let (generation_status, set_generation_status) = signal::<Option<GenerationStatus>>(None);
    let (polling, set_polling) = signal(false);
    
    // Calibration offsets (in cm, default to 0.0)
    let (horizontal_offset, set_horizontal_offset) = signal(0.0_f64);
    let (vertical_offset, set_vertical_offset) = signal(0.0_f64);

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

    // Event handlers
    let upload_handler = {
        let workflow = upload_workflow.clone();
        let status_handler = status_handler.clone();
        let file_input = file_input.clone();
        move |_| match get_files_from_input(&file_input) {
            Ok(files) => {
                if let Err(error) = validate_files(&files) {
                    status_handler.set_error(&error);
                    return;
                }
                let h_offset = horizontal_offset.get();
                let v_offset = vertical_offset.get();
                workflow.execute_with_calibration(files, h_offset, v_offset);
            }
            Err(error) => {
                status_handler.set_error(&error);
            }
        }
    };

    // Auto-status check that triggers after generation starts
    let auto_check_status = {
        let status_handler = status_handler.clone();
        move |request_id: Uuid| {
            set_polling.set(true);
            let status_handler = status_handler.clone();
            spawn_local(async move {
                // Wait 3 seconds then check status
                for _ in 0..30 {
                    gloo_net::http::Request::get("data:,").send().await.ok();
                }

                if current_request_id.get_untracked() == Some(request_id) {
                    match crate::utils::api::ApiClient::check_status(request_id).await {
                        Ok(status) => {
                            status_handler.handle_generation_status(status, request_id);
                        }
                        Err(error) => {
                            status_handler.set_error(&error);
                        }
                    }
                    set_polling.set(false);
                }
            });
        }
    };

    // Trigger auto-check when generation starts
    Effect::new({
        let auto_check_status = auto_check_status.clone();
        move |_| {
            if let Some(request_id) = current_request_id.get() {
                let status = generation_status.get();
                if matches!(status, Some(GenerationStatus::Pending)) {
                    auto_check_status(request_id);
                }
            }
        }
    });

    // Check status button handler
    let check_status_handler = {
        let status_handler = status_handler.clone();
        move |_| {
            if let Some(request_id) = current_request_id.get() {
                let status_handler = status_handler.clone();
                spawn_local(async move {
                    match crate::utils::api::ApiClient::check_status(request_id).await {
                        Ok(status) => {
                            status_handler.handle_generation_status(status, request_id);
                        }
                        Err(error) => {
                            status_handler.set_error(&error);
                        }
                    }
                });
            }
        }
    };

    let cleanup_handler = {
        let status_handler = status_handler.clone();
        move |_| {
            if let Some(request_id) = current_request_id.get() {
                set_polling.set(false);
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
            set_polling.set(false);
            status_handler.reset();
            set_current_request_id.set(None);
            crate::utils::api::clear_file_input(&file_input);
        }
    };

    view! {
        <div class="file-upload-container">
            <h3>"Upload PDF Files for Processing"</h3>

            <div class="upload-section">
                <input
                    node_ref=file_input
                    type="file"
                    multiple=true
                    accept=".pdf"
                    disabled={move || uploading.get()}
                />
                
                <div class="calibration-section">
                    <h4>"Printer Calibration (Optional)"</h4>
                    <div class="calibration-inputs">
                        <div class="calibration-input-group">
                            <label for="horizontal-offset">"Horizontal Offset (cm):"</label>
                            <input
                                id="horizontal-offset"
                                type="number"
                                step="0.1"
                                value={move || horizontal_offset.get().to_string()}
                                on:input=move |ev| {
                                    if let Ok(val) = event_target_value(&ev).parse::<f64>() {
                                        set_horizontal_offset.set(val);
                                    }
                                }
                                disabled={move || uploading.get()}
                            />
                        </div>
                        <div class="calibration-input-group">
                            <label for="vertical-offset">"Vertical Offset (cm):"</label>
                            <input
                                id="vertical-offset"
                                type="number"
                                step="0.1"
                                value={move || vertical_offset.get().to_string()}
                                on:input=move |ev| {
                                    if let Ok(val) = event_target_value(&ev).parse::<f64>() {
                                        set_vertical_offset.set(val);
                                    }
                                }
                                disabled={move || uploading.get()}
                            />
                        </div>
                    </div>
                    <small class="help-text">
                        "Adjust these values if your printer misaligns double-sided prints. Use the calibration page to measure offsets."
                    </small>
                </div>

                <div class="button-group">
                    <button
                        on:click=upload_handler
                        disabled={move || uploading.get() || polling.get()}
                        class="upload-btn"
                    >
                        {move || {
                            if uploading.get() {
                                "Uploading Files..."
                            } else {
                                "Upload & Generate PDF"
                            }
                        }}
                    </button>

                    <button
                        on:click=check_status_handler
                        disabled={move || current_request_id.get().is_none() || polling.get()}
                        class="status-btn"
                    >
                        {move || if polling.get() { "Checking..." } else { "Check Status" }}
                    </button>

                    <button
                        on:click=reset_handler
                        disabled={move || uploading.get()}
                        class="reset-btn"
                    >
                        "Reset"
                    </button>
                </div>
            </div>

            <div class="status-section" class:hidden={move || status.get().is_empty()}>
                <p class="status-text">
                    {move || status.get()}
                    {move || {
                        if polling.get() {
                            view! { <span class="spinner">"..."</span> }.into_any()
                        } else if matches!(generation_status.get(), Some(GenerationStatus::Pending) | Some(GenerationStatus::Generating)) {
                            view! { <span class="help-text">" (Click 'Check Status' to update)"</span> }.into_any()
                        } else {
                            view! { <span></span> }.into_any()
                        }
                    }}
                </p>

                {move || {
                    if let Some(gen_status) = generation_status.get() {
                        let class = get_status_indicator_class(&gen_status);
                        let text = get_status_text(&gen_status);
                        view! { <div class=class>{text}</div> }.into_any()
                    } else {
                        view! { <div></div> }.into_any()
                    }
                }}
            </div>

            <div class="download-section" class:hidden={move || !show_download.get()}>
                <h4>"Download Generated PDF"</h4>
                <a
                    href={move || download_url.get()}
                    target="_blank"
                    class="download-link"
                >
                    "📄 Download PDF"
                </a>

                <button
                    on:click=cleanup_handler
                    class="cleanup-btn"
                >
                    "Clean Up Request"
                </button>
            </div>

            <div class="progress-section" class:hidden={move || !uploading.get()}>
                <div class="progress-bar">
                    <div class="progress-fill"></div>
                </div>
                <small>"Uploading files and starting generation..."</small>
            </div>

            <div class="checking-section" class:hidden={move || !polling.get()}>
                <div class="spinner-large">...</div>
                <small>"Checking status..."</small>
            </div>
        </div>
    }
}
