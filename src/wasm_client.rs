use crate::app::{App, CurrentScreen, InputMode, LoginField};
use crate::config::Account;
use crate::player::Player;
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct WasmClient {
    app: Rc<RefCell<App>>,
    terminal: Rc<RefCell<Terminal<TestBackend>>>,
    player: Player,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WasmClient {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();
        let backend = TestBackend::new(120, 30);
        let terminal = Terminal::new(backend).unwrap();

        let app = Rc::new(RefCell::new(App::new()));
        // Try load config (it uses local storage internally now)
        if let Ok(cfg) = crate::config::AppConfig::load() {
            app.borrow_mut().config = cfg;
        }

        Self {
            app,
            terminal: Rc::new(RefCell::new(terminal)),
            player: Player::new(),
        }
    }

    pub fn draw(&mut self) -> String {
        let mut app = self.app.borrow_mut();
        let mut term = self.terminal.borrow_mut();

        let _ = term.draw(|f| crate::ui::ui(f, &mut app));

        let buffer = term.backend().buffer();
        let mut s = String::new();
        for y in 0..buffer.area.height {
            for x in 0..buffer.area.width {
                let cell = buffer.get(x, y);
                s.push_str(cell.symbol());
            }
            s.push('\n');
        }
        s
    }

    pub fn handle_key(&mut self, key: String) {
        let mut app = self.app.borrow_mut();

        if app.input_mode == InputMode::Normal {
            if key == "q" {
                app.should_quit = true;
            }
        }

        match app.current_screen {
            CurrentScreen::Home => {
                match key.as_str() {
                    "n" => app.current_screen = CurrentScreen::Login,
                    "s" => app.current_screen = CurrentScreen::Settings,
                    "j" | "ArrowDown" => app.next_account(),
                    "k" | "ArrowUp" => app.previous_account(),
                    "Enter" => {
                        // Mock login for UI test
                        // Allow mock login even if empty for testing purposes
                        if app.config.accounts.is_empty() {
                            // If empty, just create a mock one so we can proceed
                            app.config.accounts.push(crate::config::Account {
                                name: "Mock Account".into(),
                                base_url: "http://mock.local".into(),
                                username: "mock".into(),
                                password: "mock".into(),
                                epg_url: None,
                                last_refreshed: None,
                                total_channels: None,
                                total_movies: None,
                                total_series: None,
                            });
                        }

                        if !app.config.accounts.is_empty() {
                            // Just switch to categories directly for test
                            app.current_screen = CurrentScreen::Categories;
                            app.selected_category_index = 0;
                            app.category_list_state.select(Some(0));
                            // Mock categories
                            app.categories = vec![
                                crate::api::Category {
                                    category_id: "1".into(),
                                    category_name: "Mock Category 1".into(),
                                    parent_id: serde_json::Value::Number(0.into()),
                                },
                                crate::api::Category {
                                    category_id: "2".into(),
                                    category_name: "Mock Category 2".into(),
                                    parent_id: serde_json::Value::Number(0.into()),
                                },
                            ];
                        }
                    }
                    _ => {}
                }
            }
            CurrentScreen::Login => {
                match app.input_mode {
                    InputMode::Normal => match key.as_str() {
                        "Escape" => app.current_screen = CurrentScreen::Home,
                        "j" | "ArrowDown" | "Tab" => {
                            app.login_field_focus = match app.login_field_focus {
                                LoginField::Name => LoginField::Url,
                                LoginField::Url => LoginField::Username,
                                LoginField::Username => LoginField::Password,
                                LoginField::Password => LoginField::EpgUrl,
                                LoginField::EpgUrl => LoginField::Name,
                            };
                        }
                        "k" | "ArrowUp" => {
                            app.login_field_focus = match app.login_field_focus {
                                LoginField::Name => LoginField::EpgUrl,
                                LoginField::Url => LoginField::Name,
                                LoginField::Username => LoginField::Url,
                                LoginField::Password => LoginField::Username,
                                LoginField::EpgUrl => LoginField::Password,
                            };
                        }
                        "Enter" => app.toggle_input_mode(),
                        _ => {}
                    },
                    InputMode::Editing => {
                        // Define closure to help update input
                        let update_input = |app: &mut App, val: String| match app.login_field_focus
                        {
                            LoginField::Name => app.input_name = tui_input::Input::new(val),
                            LoginField::Url => app.input_url = tui_input::Input::new(val),
                            LoginField::Username => app.input_username = tui_input::Input::new(val),
                            LoginField::Password => app.input_password = tui_input::Input::new(val),
                            LoginField::EpgUrl => app.input_epg_url = tui_input::Input::new(val),
                        };

                        let get_input = |app: &App| -> String {
                            match app.login_field_focus {
                                LoginField::Name => app.input_name.value().into(),
                                LoginField::Url => app.input_url.value().into(),
                                LoginField::Username => app.input_username.value().into(),
                                LoginField::Password => app.input_password.value().into(),
                                LoginField::EpgUrl => app.input_epg_url.value().into(),
                            }
                        };

                        match key.as_str() {
                            "Escape" => app.toggle_input_mode(),
                            "Enter" => {
                                app.toggle_input_mode();
                                app.login_field_focus = match app.login_field_focus {
                                    LoginField::Name => LoginField::Url,
                                    LoginField::Url => LoginField::Username,
                                    LoginField::Username => LoginField::Password,
                                    LoginField::Password => LoginField::EpgUrl,
                                    LoginField::EpgUrl => {
                                        let name = app.input_name.value().to_string();
                                        let url = app.input_url.value().to_string();
                                        let user = app.input_username.value().to_string();
                                        let pass = app.input_password.value().to_string();
                                        let epg = app.input_epg_url.value().to_string();
                                        let epg_opt = if epg.is_empty() { None } else { Some(epg) };

                                        if !name.is_empty() && !url.is_empty() {
                                            let acc = Account {
                                                name,
                                                base_url: url,
                                                username: user,
                                                password: pass,
                                                epg_url: epg_opt,
                                                last_refreshed: None,
                                                total_channels: None,
                                                total_movies: None,
                                                total_series: None,
                                            };
                                            app.config.add_account(acc);
                                            app.current_screen = CurrentScreen::Home;
                                            app.input_name = tui_input::Input::default();
                                            app.input_url = tui_input::Input::default();
                                            app.input_username = tui_input::Input::default();
                                            app.input_password = tui_input::Input::default();
                                            app.input_epg_url = tui_input::Input::default();
                                            // Return any field
                                            LoginField::Name
                                        } else {
                                            LoginField::EpgUrl
                                        }
                                    }
                                };
                            }
                            "Backspace" => {
                                let val = get_input(&app);
                                if !val.is_empty() {
                                    update_input(&mut app, val[0..val.len() - 1].to_string());
                                }
                            }
                            k if k.len() == 1 => {
                                let val = get_input(&app);
                                update_input(&mut app, format!("{}{}", val, k));
                            }
                            _ => {}
                        }
                    }
                }
            }
            CurrentScreen::Categories
            | CurrentScreen::VodCategories
            | CurrentScreen::VodStreams
            | CurrentScreen::Settings
            | CurrentScreen::Streams => {
                // Simplified handler for other screens for now
                match key.as_str() {
                    "Escape" => app.current_screen = CurrentScreen::Home,
                    "Tab" => {
                        if app.current_screen == CurrentScreen::Categories {
                            app.current_screen = CurrentScreen::VodCategories;
                            app.selected_vod_category_index = 0;
                            app.vod_category_list_state.select(Some(0));
                            // Mock VOD categories
                            app.vod_categories = vec![
                                crate::api::Category {
                                    category_id: "v1".into(),
                                    category_name: "Action Movies".into(),
                                    parent_id: serde_json::Value::Number(0.into()),
                                },
                                crate::api::Category {
                                    category_id: "v2".into(),
                                    category_name: "Comedy Movies".into(),
                                    parent_id: serde_json::Value::Number(0.into()),
                                },
                            ];
                        } else if app.current_screen == CurrentScreen::VodCategories {
                            app.current_screen = CurrentScreen::Categories;
                            app.selected_category_index = 0;
                            app.category_list_state.select(Some(0));
                        }
                    }
                    "j" | "ArrowDown" => match app.current_screen {
                        CurrentScreen::Categories => app.next_category(),
                        CurrentScreen::Streams => app.next_stream(),
                        CurrentScreen::VodCategories => app.next_vod_category(),
                        CurrentScreen::VodStreams => app.next_vod_stream(),
                        CurrentScreen::Settings => app.next_setting(),
                        _ => {}
                    },
                    "k" | "ArrowUp" => match app.current_screen {
                        CurrentScreen::Categories => app.previous_category(),
                        CurrentScreen::Streams => app.previous_stream(),
                        CurrentScreen::VodCategories => app.previous_vod_category(),
                        CurrentScreen::VodStreams => app.previous_vod_stream(),
                        CurrentScreen::Settings => app.previous_setting(),
                        _ => {}
                    },
                    _ => {}
                }
            }
            _ => {}
        }
    }
}
