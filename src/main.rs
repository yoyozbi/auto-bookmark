use cfg_if::cfg_if;
cfg_if! {
    if #[cfg(feature = "ssr")] {
use auto_bookmark::{app::*, upload_route::file_upload_routes, generation::GenerationStatus};
    use axum::Router;
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use auto_bookmark::fallback::file_and_error_handler;

use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::{Duration, SystemTime};
use tokio::signal;

/// Cleanup task that removes downloaded requests older than 5 minutes
async fn cleanup_old_downloads(app_state: AppState) {
    // When on debug, the timer is every 30s
    #[cfg(debug_assertions)]
    let time =
        Duration::from_secs(30); // 30 seconds
    #[cfg(not(debug_assertions))]
    let time = Duration::from_mins(5); // 5 minutes

    #[cfg(debug_assertions)]
    let no_activity_duration = Duration::from_mins(2); // 2 minutes
    #[cfg(not(debug_assertions))]
    let no_activity_duration = Duration::from_mins(30); // 30 minutes

    let mut interval = tokio::time::interval(time);
    loop {
        interval.tick().await;

        let mut requests = app_state.requests.lock().await;
        let now = SystemTime::now();
        let initial_count = requests.len();

        // Remove requests that were downloaded
        requests.retain(|req| {
            if matches!(req.status(), GenerationStatus::Downloaded)
                    && let Some(downloaded_at) = req.downloaded_at
                    && let Ok(elapsed) = now.duration_since(downloaded_at)
                    && elapsed > time{
                log!("Cleaning up old downloaded request: {}", req.id);
                return false; // Remove this request
            }
            true // Keep this request
        });

        for (i, req) in requests.clone().iter().enumerate() {
            // Also remove requests that have had no activity for a long time
            if let Ok(elapsed) = now.duration_since(req.created_at) && elapsed > no_activity_duration {
                log!("Cleaning up inactive request: {}", req.id);
                req.delete_files().await.ok();
                requests.remove(i);
            }
        }

        // Clear uploaded files that are not linked to anything
        let files_in_use: Vec<String> = requests
            .iter()
            .flat_map(|req| req.input_files.clone())
            .collect();


        let files = tokio::fs::read_dir("uploads/").await;
        let file_paths = match files {
            Ok(mut dir) => {
                let mut paths = Vec::new();
                while let Ok(Some(entry)) = dir.next_entry().await {
                    paths.push(entry.path());
                }
                paths
            }
            Err(_) => Vec::new(),
        };

        for path in file_paths {
            if let Some(path_str) = path.to_str()
                    && !files_in_use.contains(&path_str.to_string()){
                log!("Removing unlinked uploaded file: {}", path_str);
                tokio::fs::remove_file(path).await.ok();
            }
        }


        let cleaned_count = initial_count - requests.len();
        if cleaned_count > 0 {
            log!("Cleaned up {} old downloaded requests", cleaned_count);
        }
    }
}

async fn shutdown_signal()  {
    match signal::ctrl_c().await {
        Ok(()) => {},
        Err(err) => {
            eprint!("Unable to listen for shutdown signal: {}", err)
        }
    }
}

#[tokio::main]
async fn main() {
    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let app_state = AppState {
        leptos_options: conf.leptos_options,
        requests: Arc::new(Mutex::new(Vec::new()))
    };
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);

    // Start the cleanup background task
    tokio::spawn(cleanup_old_downloads(app_state.clone()));

    let app = Router::new()
        .leptos_routes(&app_state, routes, {
            let leptos_options = app_state.leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .merge(file_upload_routes())
        .fallback(file_and_error_handler)
        .with_state(app_state);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}
    }
    else {
        pub fn main() {
            // no client-side main function
            // unless we want this to work with e.g., Trunk for pure client-side testing
            // see lib.rs for hydration function instead
        }
    }
}
