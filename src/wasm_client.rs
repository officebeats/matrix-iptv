use crate::app::{App, CurrentScreen, InputMode, LoginField};
use crate::config::Account;
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;


#[wasm_bindgen]
extern "C" {
    fn playStream(url: &str);
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct WasmClient {
    app: Rc<RefCell<App>>,
    terminal: Rc<RefCell<Terminal<TestBackend>>>,
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
        }
    }

    pub fn draw(&mut self) -> String {
        let mut app = self.app.borrow_mut();
        let mut term = self.terminal.borrow_mut();

        let _ = term.draw(|f| crate::ui::ui(f, &mut app));
        app.loading_tick = app.loading_tick.wrapping_add(1);

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
                        if let Some(acc) = app.get_selected_account() {
                            let base_url = acc.base_url.clone();
                            let username = acc.username.clone();
                            let password = acc.password.clone();
                            
                            // Auto-inject localhost:8081 for POC testing if not already present
                            let proxy_url = if !base_url.contains("localhost") {
                                format!("http://localhost:8081/{}", base_url)
                            } else {
                                base_url
                            };

                            web_sys::console::log_1(&format!("[Wasm] Authenticating via proxy: {}", proxy_url).into());

                            let client = crate::api::XtreamClient::new(proxy_url, username, password);
                            app.current_client = Some(client.clone());
                            app.state_loading = true;

                            let app_rc = self.app.clone();
                            
                            // Spawn async login
                            wasm_bindgen_futures::spawn_local(async move {
                                let result = client.authenticate().await;
                                {
                                    let mut app = app_rc.borrow_mut();
                                    app.state_loading = false;
                                }
                                
                                match result {
                                    Ok((true, user_info, server_info)) => {
                                        web_sys::console::log_1(&"[Wasm] Login Successful. Fetching categories...".into());
                                        let mut app = app_rc.borrow_mut();
                                        app.cached_user_timezone = server_info.as_ref()
                                            .and_then(|i| i.timezone.clone())
                                            .unwrap_or_else(|| "UTC".into());
                                        
                                        let client_clone = client.clone();
                                        let app_rc_inner = app_rc.clone();
                                        
                                        // Fetch categories
                                        wasm_bindgen_futures::spawn_local(async move {
                                             let cats_res = client_clone.get_live_categories().await;
                                             let mut app = app_rc_inner.borrow_mut();
                                             match cats_res {
                                                 Ok(cats) => {
                                                     web_sys::console::log_1(&format!("[Wasm] Loaded {} categories.", cats.len()).into());
                                                     app.categories = cats.clone();
                                                     app.all_categories = cats;
                                                     app.current_screen = CurrentScreen::Categories;
                                                 }
                                                 Err(e) => {
                                                     web_sys::console::error_1(&format!("[Wasm] Failed to load categories: {}", e).into());
                                                 }
                                             }
                                        });
                                    },
                                    Ok((false, _, _)) => {
                                         web_sys::console::error_1(&"[Wasm] Login Failed: Invalid Credentials".into());
                                    },
                                    Err(e) => {
                                        web_sys::console::error_1(&format!("[Wasm] Network Error during login: {}", e).into());
                                    }
                                }
                            });
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
                            "Enter" | "Tab" => {
                                // Advance focus but STAY in editing mode
                                app.login_field_focus = match app.login_field_focus {
                                    LoginField::Name => LoginField::Url,
                                    LoginField::Url => LoginField::Username,
                                    LoginField::Username => LoginField::Password,
                                    LoginField::Password => LoginField::EpgUrl,
                                    LoginField::EpgUrl => {
                                        // Final field, try to save
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
                                                account_type: crate::config::AccountType::Xtream,
                                                epg_url: epg_opt,
                                                last_refreshed: None,
                                                total_channels: None,
                                                total_movies: None,
                                                total_series: None,
                                                server_timezone: None,
                                            };
                                            app.config.add_account(acc);
                                            app.toggle_input_mode(); // Done editing
                                            app.current_screen = CurrentScreen::Home;
                                            app.input_name = tui_input::Input::default();
                                            app.input_url = tui_input::Input::default();
                                            app.input_username = tui_input::Input::default();
                                            app.input_password = tui_input::Input::default();
                                            app.input_epg_url = tui_input::Input::default();
                                            LoginField::Name
                                        } else {
                                            // Missing required fields, stay on epg or move back?
                                            // Let's stay and highlight?
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
                             // Fetch VOD categories async?
                             if let Some(client) = &app.current_client {
                                 let client = client.clone();
                                 let app_rc = self.app.clone();
                                 wasm_bindgen_futures::spawn_local(async move {
                                     if let Ok(cats) = client.get_vod_categories().await {
                                         let mut app = app_rc.borrow_mut();
                                         app.vod_categories = cats;
                                     }
                                 });
                             }

                        } else if app.current_screen == CurrentScreen::VodCategories {
                            app.current_screen = CurrentScreen::Categories;
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
                    "Enter" => {
                        match app.current_screen {
                            CurrentScreen::Categories => {
                                if let Some(cat) = app.get_selected_category() {
                                    let id = cat.category_id.clone();
                                    if let Some(client) = &app.current_client {
                                         let client = client.clone();
                                         let app_rc = self.app.clone();
                                         // Show loading?
                                         wasm_bindgen_futures::spawn_local(async move {
                                             if let Ok(streams) = client.get_live_streams(&id).await {
                                                 let mut app = app_rc.borrow_mut();
                                                 app.streams = streams.clone();
                                                 app.all_streams = streams;
                                                 app.current_screen = CurrentScreen::Streams; 
                                             }
                                         });
                                    }
                                }
                            }
                            CurrentScreen::Streams => {
                                if let Some(stream) = app.get_selected_stream() {
                                     let stream_id = match &stream.stream_id {
                                         serde_json::Value::String(s) => s.clone(),
                                         serde_json::Value::Number(n) => n.to_string(),
                                         _ => stream.stream_id.to_string(),
                                     };
                                     let ext = stream.container_extension.clone().unwrap_or_else(|| "m3u8".to_string());
                                     
                                     if let Some(client) = &app.current_client {
                                         let url = client.get_stream_url(&stream_id, &ext);
                                         web_sys::console::log_1(&format!("[Wasm] Playing Stream: {} (URL: {})", stream.name, url).into());
                                         
                                         // Trigger JS playback
                                         playStream(&url);
                                     }
                                }
                            }
                             _ => {}
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}
