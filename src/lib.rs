pub mod app;

pub mod pages;
pub mod upload_route;

pub mod fallback;

pub mod generation;
pub mod utils;

// Load i18n locales from Cargo.toml metadata
leptos_i18n::load_locales!();

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
