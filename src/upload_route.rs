use cfg_if::cfg_if;
cfg_if! {
    if #[cfg(feature = "ssr")] {
        use crate::app::AppState;
        use crate::generation::{GenerationRequest, GenerationStatus};
        use uuid::Uuid;
        use std::error::Error;

        use axum::{
            Json,
            extract::{Multipart, State, Path},
            http::StatusCode,
            http::{header, HeaderMap},
            routing::{post, get, delete, put},
            Router,
        };

        use tokio::fs;

        use axum::response::IntoResponse;
        
        #[derive(serde::Deserialize)]
        pub struct CalibrationParams {
            pub horizontal_cm: f64,
            pub vertical_cm: f64,
        }

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
                    Err(e) => {
                        println!("No data: {}", e);
                        failed = true;
                    }
                }
                break;
            }

            if failed {
                println!("File upload failed for request: {}", request_id);
                return Err(StatusCode::BAD_REQUEST);
            }

            let request = request.unwrap();
            request.add_file(file_path.clone());
            println!("Successfully uploaded file {} for request: {}", file_path, request_id);
            Ok(StatusCode::OK)
        }

        pub async fn generate_pdf(
            Path(request_id): Path<Uuid>,
            State(app_state): State<AppState>,
        ) -> impl IntoResponse
        {
            {
                let mut requests = app_state.requests.lock().await;
                let request = requests.iter_mut().find(|f| f.id == request_id);

                if request.is_none() {
                    return Err(StatusCode::NOT_FOUND);
                }

                request.unwrap().set_status(GenerationStatus::Generating);
            }

            let app_state_clone = app_state.clone();
            tokio::spawn(async move {
                println!("Starting PDF generation for request: {}", request_id);
                let result = {
                    let requests = app_state_clone.requests.lock().await;
                    let request = requests.iter().find(|f| f.id == request_id);
                    if let Some(req) = request {
                        req.generate_pdf().await
                    } else {
                        println!("Request {} not found during generation", request_id);
                        return;
                    }
                };

                let mut requests = app_state_clone.requests.lock().await;
                match result {
                    Ok(data) => {
                        println!("PDF generation successful for request: {}, data size: {} bytes", request_id, data.len());
                        if let Some(request) = requests.iter_mut().find(|f| f.id == request_id) {
                            request.set_generated_data(data);
                            request.set_status(GenerationStatus::Success);
                        }
                    },
                    Err(err) => {
                        println!("PDF generation failed for request {}: {}", request_id, err);
                        if let Some(request) = requests.iter_mut().find(|f| f.id == request_id) {
                            request.set_status(GenerationStatus::Failure(err.to_string()));
                        }
                    },
                }
            });

            Ok(StatusCode::ACCEPTED)
        }
        
        pub async fn set_calibration(
            Path(request_id): Path<Uuid>,
            State(app_state): State<AppState>,
            Json(params): Json<CalibrationParams>,
        ) -> impl IntoResponse {
            let mut requests = app_state.requests.lock().await;
            let request = requests.iter_mut().find(|f| f.id == request_id);

            match request {
                Some(req) => {
                    req.set_calibration(params.horizontal_cm, params.vertical_cm);
                    println!("Set calibration for request {}: h={}, v={}", 
                             request_id, params.horizontal_cm, params.vertical_cm);
                    Ok(StatusCode::OK)
                },
                None => Err(StatusCode::NOT_FOUND),
            }
        }

        pub async fn get_status(
            Path(request_id): Path<Uuid>,
            State(app_state): State<AppState>,
        ) -> impl IntoResponse {
            let requests = app_state.requests.lock().await;
            let request = requests.iter().find(|f| f.id == request_id);

            match request {
                Some(req) => Ok(Json(req.status())),
                None => Err(StatusCode::NOT_FOUND),
            }
        }

        pub async fn get_file(
            Path(request_id): Path<Uuid>,
            State(app_state): State<AppState>,
        ) -> impl IntoResponse {
            let requests = app_state.requests.lock().await;
            let request = requests.iter().find(|f| f.id == request_id);

            match request {
                Some(req) if matches!(req.status(), GenerationStatus::Success) => {
                    if let Some(data) = req.get_generated_data() {
                        let mut headers = HeaderMap::new();
                        headers.insert(
                            header::CONTENT_TYPE,
                            "application/pdf".parse().unwrap(),
                        );
                        headers.insert(
                            header::CONTENT_DISPOSITION,
                            "attachment; filename=\"generated.pdf\"".parse().unwrap(),
                        );
                        Ok((headers, data.clone()))
                    } else {
                        println!("Request {} has success status but no generated data", request_id);
                        Err(StatusCode::INTERNAL_SERVER_ERROR)
                    }
                },
                Some(req) => {
                    println!("Request {} found but status is: {:?}", request_id, req.status());
                    Err(StatusCode::NOT_FOUND)
                },
                None => {
                    println!("Request {} not found", request_id);
                    Err(StatusCode::NOT_FOUND)
                },
            }
        }

        pub async fn cleanup_request(
            Path(request_id): Path<Uuid>,
            State(app_state): State<AppState>,
        ) -> impl IntoResponse {
            let mut requests = app_state.requests.lock().await;
            let initial_len = requests.len();
            requests.retain(|r| r.id != request_id);

            if requests.len() < initial_len {
                println!("Cleaned up request: {}", request_id);
                Ok(StatusCode::NO_CONTENT)
            } else {
                println!("Request {} not found for cleanup", request_id);
                Err(StatusCode::NOT_FOUND)
            }
        }

        // Add this route to your Axum router
        pub fn file_upload_routes() -> Router<AppState> {
            Router::new()
                .route("/api/upload", post(create_upload_request))
                .route("/api/upload/{request_id}/file", post(upload_file))
                .route("/api/upload/{request_id}/calibration", put(set_calibration))
                .route("/api/upload/{request_id}/generate", post(generate_pdf))
                .route("/api/upload/{request_id}/status", get(get_status))
                .route("/api/upload/{request_id}/download", get(get_file))
                .route("/api/upload/{request_id}", delete(cleanup_request))
        }
    }
}
