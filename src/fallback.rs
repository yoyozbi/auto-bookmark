use cfg_if::cfg_if;

cfg_if! {
if #[cfg(feature = "ssr")] {
    use axum::{
        body::Body,
        extract::State,
        response::IntoResponse,
        http::{Request, Response, StatusCode, Uri, HeaderMap, HeaderValue},
    };
    use axum::response::Response as AxumResponse;
    use tower_http::services::ServeDir;
    use tower::ServiceExt;
    use leptos::{view, prelude::*};
    use crate::app::AppState;

    #[component]
    fn NotFound() -> impl IntoView {
        view! {
            <p>404 Not Found</p>
        }
    }

    pub async fn file_and_error_handler(uri: Uri, State(options): State<AppState>, req: Request<Body>) -> AxumResponse {
        let root = options.leptos_options.site_root.clone();
        let res = get_static_file(uri.clone(), &root).await.unwrap();

        if res.status() == StatusCode::OK {
            res.into_response()
        } else if looks_like_asset(uri.path()) {
            StatusCode::NOT_FOUND.into_response()
        } else {
            let handler = leptos_axum::render_app_to_stream(NotFound);
            handler(req).await.into_response()
        }
    }

    fn looks_like_asset(path: &str) -> bool {
        matches!(
            std::path::Path::new(path).extension().and_then(|e| e.to_str()),
            Some("wasm" | "js" | "css" | "ico" | "png" | "jpg" | "svg" | "woff" | "woff2" | "ttf")
        )
    }

    async fn get_static_file(uri: Uri, root: &str) -> Result<Response<Body>, (StatusCode, String)> {
        let req = Request::builder().uri(uri.clone()).body(Body::empty()).unwrap();
        // `ServeDir` implements `tower::Service` so we can call it with `tower::ServiceExt::oneshot`
        // This path is relative to the cargo root
        match ServeDir::new(root).oneshot(req).await {
            Ok(res) => {
                // Convert the response body to axum::body::Body
                let (mut parts, body) = res.into_parts();
                let body = Body::new(body);

                // Set correct MIME type for WASM files
                if uri.path().ends_with(".wasm") {
                    parts.headers.insert(
                        axum::http::header::CONTENT_TYPE,
                        HeaderValue::from_static("application/wasm")
                    );
                }

                Ok(Response::from_parts(parts, body))
            },
            Err(err) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Something went wrong: {err}"),
            )),
        }
    }


}
}
