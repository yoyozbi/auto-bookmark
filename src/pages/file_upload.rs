use gloo_net::http::Request;
use leptos::html::Input;
use leptos::prelude::*;
use leptos::reactive::spawn_local;
use web_sys::{FormData, HtmlInputElement};

#[component]
pub fn FileUpload() -> impl IntoView {
    let (status, set_status) = signal(String::new());
    let (uploading, set_uploading) = signal(false);
    let file_input: NodeRef<Input> = NodeRef::new();

    let upload = move |_| {
        spawn_local(async move {
            set_uploading.set(true);
            set_status.set("Uploading...".to_string());

            let input = file_input.get().unwrap();
            let input_el: HtmlInputElement = input;
            let files = input_el.files().unwrap();

            if files.length() == 0 {
                set_status.set("No files selected".to_string());
                set_uploading.set(false);
                return;
            }

            let form_data = FormData::new().unwrap();
            for i in 0..files.length() {
                let file = files.get(i).unwrap();
                form_data.append_with_blob("files", &file).unwrap();
            }

            let resp = Request::post("/api/upload")
                .body(form_data)
                .unwrap()
                .send()
                .await;

            match resp {
                Ok(response) if response.ok() => {
                    set_status.set("Upload successful!".to_string());
                }
                _ => {
                    set_status.set("Upload failed".to_string());
                }
            }

            set_uploading.set(false);
        });
    };

    view! {
        <div>
            <h3>"Upload Files"</h3>
            <input
                node_ref=file_input
                type="file"
                multiple=true
            />
            <button
                on:click=upload
                disabled=move || uploading.get()
            >
                "Upload"
            </button>
            <p>{move || status.get()}</p>
        </div>
    }
}
