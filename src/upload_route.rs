use cfg_if::cfg_if;
cfg_if! {
    if #[cfg(feature = "ssr")] {
        use crate::app::AppState;
        use crate::generation::GenerationRequest;
        use uuid::Uuid;
        use std::error::Error;

        use axum::{
            Json,
            extract::{Multipart, State, Path},
            http::StatusCode,
            http::{header, HeaderMap},
            routing::post,
            Router,
        };

        use tokio::fs;

        use axum::response::IntoResponse;

        #[axum::debug_handler]
        pub async fn create_upload_request(State(app_state): State<AppState>) -> impl IntoResponse {
            let mut requests = app_state.requests.lock().await;
            let request = GenerationRequest::default();
            println!("Create upload request: {:#?}", request);
            requests.push(request.clone());
            Json(request)
        }

        // Handler for multiple file upload
        #[axum::debug_handler]
        pub async fn upload_file(
                    State(app_state): State<AppState>,
                    Path(request_id): Path<Uuid>,
                    mut multipart: Multipart,
                ) -> impl IntoResponse
        {
            let mut requests = app_state.requests.lock().await;
            println!("Upload file: {:#?}", requests);

            let request = requests.iter_mut().find(|f| f.id == request_id);

            if request.is_none() {
                return Err(StatusCode::NOT_FOUND);
            }

            let mut file_path = String::new();
            let mut failed = false;

            // Create uploads directory if it doesn't exist
            fs::create_dir_all("uploads").await.map_err(|e| {
                println!("Error {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            while let Some(field) = multipart.next_field().await.map_err(|e| {
                println!("Error next field: {:#?}", e.source());
                StatusCode::BAD_REQUEST
            })? {
                if field.name() != Some("file") {
                    continue;
                }

                // Extract file_name before any await
                let file_name = match field.file_name() {
                    Some(name) => name.to_string(),
                    None => {
                        println!("Unknown file");
                        failed = true;
                        break;
                    }
                };

                // Move file_name into a new scope to avoid crossing await boundary
                let file_path_owned = {
                    let timestamp = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis();
                    format!("uploads/{}_{}", timestamp, file_name)
                };

                match field.bytes().await {
                    Ok(data) => {

                        match fs::write(&file_path_owned, &data).await {
                            Ok(_) => {
                                file_path = file_path_owned;
                            }
                            Err(e) => {
                                println!("Failed to write {}", e);
                                failed = true;
                            }
                        }
                    }
                    Err(_) => {
                        println!("No data");
                        failed = true;
                    }
                }
                break;
            }

            if failed {
                return Err(StatusCode::BAD_REQUEST);
            }

            request.unwrap().add_file(file_path);
            Ok(())
        }

        pub async fn generate_pdf(
            Path(request_id): Path<Uuid>,
            State(app_state): State<AppState>,
        ) -> impl IntoResponse
        {
            let mut requests = app_state.requests.lock().await;
            let request = requests.iter().find(|f| f.id == request_id);

            if request.is_none() {
                return Err(StatusCode::NOT_FOUND);
            }

            let request = request.unwrap();
            let result = request.generate_pdf().await;
            requests.retain(|f| f.id != request_id);

            match result {
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
                },
                Err(err) => {
                    println!("Failed to generate pdf: {}", err);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }

        // Add this route to your Axum router
        pub fn file_upload_routes() -> Router<AppState> {
            Router::new()
                .route("/api/upload", post(create_upload_request))
                .route("/api/upload/{request_id}/file", post(upload_file))
                .route("/api/upload/{request_id}/result", post(generate_pdf))
        }
    }
}
