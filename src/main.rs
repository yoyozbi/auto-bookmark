use cfg_if::cfg_if;
cfg_if! {
    if #[cfg(feature = "ssr")] {
use auto_bookmark::{app::*, upload_route::file_upload_routes};
    use axum::Router;
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};

use std::sync::Arc;
use tokio::sync::Mutex;
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

    let app = Router::new()
        .leptos_routes(&app_state, routes, {
            let leptos_options = app_state.leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .merge(file_upload_routes())
        //.fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(app_state);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
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
