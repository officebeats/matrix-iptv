pub mod api;
pub mod app;
pub mod cache;
#[cfg(all(not(target_arch = "wasm32"), feature = "chromecast"))]
pub mod cast;
pub mod config;
pub mod doh;
pub mod errors;
pub mod flex_id;
pub mod handlers;
#[cfg(not(target_arch = "wasm32"))]
pub mod matrix_rain;
#[cfg(not(target_arch = "wasm32"))]
pub mod onboarding;
pub mod parser;
#[cfg(not(target_arch = "wasm32"))]
pub mod player;
pub mod preprocessing;
pub mod scores;
pub mod setup;
pub mod sports;
pub mod state;
pub mod ui;

// Wasm module
#[cfg(target_arch = "wasm32")]
pub mod wasm_client;

#[cfg(test)]
mod tests {
    use crate::app::{App, CurrentScreen};

    #[test]
    fn test_app_new() {
        let app = App::new();
        assert_eq!(app.current_screen, CurrentScreen::Home);
    }

    // test_update_account_logic removed to prevent overwriting user config.json
    // TODO: Refactor AppConfig to accept a custom path for testing.
}
