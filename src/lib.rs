pub mod api;
pub mod app;
pub mod cache;
pub mod config;
pub mod flex_id;
pub mod parser;
pub mod errors;
#[cfg(not(target_arch = "wasm32"))]
pub mod player;
#[cfg(all(not(target_arch = "wasm32"), feature = "chromecast"))]
pub mod cast;
pub mod setup;
pub mod sports;
pub mod ui;
pub mod preprocessing;
#[cfg(not(target_arch = "wasm32"))]
pub mod matrix_rain;
pub mod scores;
pub mod handlers;
pub mod state;

// Wasm module
#[cfg(target_arch = "wasm32")]
pub mod wasm_client;

#[cfg(test)]
mod tests {
    use crate::app::{App, CurrentScreen};
    use crate::config::Account;

    #[test]
    fn test_app_new() {
        let app = App::new();
        assert_eq!(app.current_screen, CurrentScreen::Home);
    }

    // test_update_account_logic removed to prevent overwriting user config.json
    // TODO: Refactor AppConfig to accept a custom path for testing.
}
