pub mod api;
pub mod app;
pub mod config;
pub mod parser;
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
pub mod handlers;

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

    #[test]
    fn test_update_account_logic() {
        let mut app = App::new();
        app.config.accounts.push(Account {
            name: "Original".to_string(),
            base_url: "x".to_string(),
            username: "u".to_string(),
            password: "p".to_string(),
            epg_url: None,
            last_refreshed: None,
            total_channels: None,
            total_movies: None,
            total_series: None,
            server_timezone: None,
        });

        // Ensure we have at least one
        if app.config.accounts.is_empty() {
            return;
        }

        let new_acc = Account {
            name: "Updated".to_string(),
            base_url: "x".to_string(),
            username: "u".to_string(),
            password: "p".to_string(),
            epg_url: None,
            last_refreshed: None,
            total_channels: None,
            total_movies: None,
            total_series: None,
            server_timezone: None,
        };

        app.config.update_account(0, new_acc);
        assert_eq!(app.config.accounts[0].name, "Updated");
    }
}
