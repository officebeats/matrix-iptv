use crate::app::{App, AsyncAction, CurrentScreen, Pane, InputMode, LoginField, Guide, SettingsState};
use crate::api::{XtreamClient, get_id_str};
use crate::{preprocessing, player};
#[cfg(feature = "chromecast")]
use crate::cast;
use crate::config::Account;
use tokio::sync::mpsc;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, Event, KeyModifiers};
use tui_input::backend::crossterm::EventHandler;
use std::io;

pub enum InputResult {
    Ok,
    Quit,
    Continue,
    UpdateRequested,
}

pub async fn handle_key_event(
    app: &mut App,
    key: KeyEvent,
    tx: &mpsc::Sender<AsyncAction>,
    player: &player::Player,
) -> io::Result<InputResult> {
    // Only process key press events, not release (Windows sends both)
    if key.kind != KeyEventKind::Press {
        return Ok(InputResult::Continue);
    }

    // Global Search Triggers - Checked at the absolute start for maximum reliability
    // Supports: Ctrl+Space, Alt+Space, Ctrl+F, Ctrl+P, F3
    let is_ctrl_space = (key.code == KeyCode::Char(' ') || key.code == KeyCode::Char('\0') || key.code == KeyCode::Null) && key.modifiers.contains(KeyModifiers::CONTROL);
    let is_ctrl_f = key.code == KeyCode::Char('f') || key.code == KeyCode::Char('F') || key.code == KeyCode::Char('\x06');
    let is_ctrl_p = key.code == KeyCode::Char('p') || key.code == KeyCode::Char('P') || key.code == KeyCode::Char('\x10');
    let is_f3 = key.code == KeyCode::F(3);

    if is_ctrl_space || (key.modifiers.contains(KeyModifiers::CONTROL) && (is_ctrl_f || is_ctrl_p)) || is_f3 {
        let on_home = app.current_screen == CurrentScreen::Home;
        app.previous_screen = Some(app.current_screen.clone());
        app.current_screen = CurrentScreen::GlobalSearch;
        app.search_mode = true;
        app.search_query.clear();
        app.update_search();
        // Force screensaver off
        app.show_matrix_rain = false;
        app.matrix_rain_screensaver_mode = false;

        // "Value Prop": If searching from home screen and no data is loaded, boot-up the highlighted account
        if on_home && app.global_all_streams.is_empty() {
             if let Some(acc) = app.config.accounts.get(app.selected_account_index) {
                 let client = crate::api::XtreamClient::new(
                     acc.base_url.clone(),
                     acc.username.clone(),
                     acc.password.clone(),
                 );
                 app.current_client = Some(client.clone());
                 let tx = tx.clone();
                 let pms = app.config.processing_modes.clone();
                 let stream_favs = app.config.favorites.streams.clone();
                 let vod_favs = app.config.favorites.vod_streams.clone();
                 let acc_name = acc.name.clone();
                 
                 tokio::spawn(async move {
                     if let Ok((true, _, _)) = client.authenticate().await {
                         // Load Live
                         if let Ok(mut streams) = client.get_live_streams("ALL").await {
                             crate::preprocessing::preprocess_streams(&mut streams, &stream_favs, &pms, true, &acc_name);
                             let _ = tx.send(AsyncAction::TotalChannelsLoaded(streams)).await;
                         }
                         // Load VOD
                         if let Ok(mut streams) = client.get_vod_streams_all().await {
                             crate::preprocessing::preprocess_streams(&mut streams, &vod_favs, &pms, false, &acc_name);
                             let _ = tx.send(AsyncAction::TotalMoviesLoaded(streams)).await;
                         }
                     }
                 });
             }
        }

        return Ok(InputResult::Continue);
    }

    // Priority 1: Help Popup
    if app.show_help {
        if matches!(
            key.code,
            KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q')
        ) {
            app.show_help = false;
        }
        return Ok(InputResult::Continue);
    }

    // Priority 2: Guide Popups
    if app.show_guide.is_some() {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter => {
                app.show_guide = None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.guide_scroll = app.guide_scroll.saturating_add(1)
            }
            KeyCode::Up | KeyCode::Char('k') => {
                app.guide_scroll = app.guide_scroll.saturating_sub(1)
            }
            _ => {}
        }
        return Ok(InputResult::Continue);
    }
    
    // Priority 3: Matrix Rain Screensaver
    if app.show_matrix_rain && app.matrix_rain_screensaver_mode {
        app.show_matrix_rain = false;
        app.matrix_rain_screensaver_mode = false;
        app.matrix_rain_start_time = None;
        return Ok(InputResult::Continue);
    }
    
    // Priority 5: Play Details Popup
    if app.show_play_details {
        match key.code {
            KeyCode::Enter => {
                if let Some(url) = app.pending_play_url.take() {
                    let title = app.pending_play_title.take().unwrap_or_default();
                    app.state_loading = true;
                    app.player_error = None;
                    app.loading_message = Some(format!("Preparing: {}...", title));
                    let tx = tx.clone();
                    let player = player.clone();
                    let use_default = app.config.use_default_mpv;
                    tokio::spawn(async move {
                        let _ = tx.send(AsyncAction::LoadingMessage("Connecting...".to_string())).await;
                        match player.play(&url, use_default) {
                            Ok(_) => {
                                match player.wait_for_playback(10000).await {
                                    Ok(true) => { let _ = tx.send(AsyncAction::PlayerStarted).await; }
                                    _ => { let _ = tx.send(AsyncAction::PlayerFailed("Failed to start".to_string())).await; }
                                }
                            }
                            Err(e) => { let _ = tx.send(AsyncAction::PlayerFailed(e.to_string())).await; }
                        }
                    });
                }
                app.show_play_details = false;
            }
            KeyCode::Esc | KeyCode::Backspace => {
                app.show_play_details = false;
                app.pending_play_url = None;
                app.pending_play_title = None;
            }
            _ => {}
        }
        return Ok(InputResult::Continue);
    }

    // Priority 6: Welcome Popup (FTUE)
    if app.show_welcome_popup {
        app.show_welcome_popup = false;
        return Ok(InputResult::Continue);
    }

    // Priority 7: Cast Device Picker
    if app.show_cast_picker {
        match key.code {
            KeyCode::Esc => {
                app.show_cast_picker = false;
                app.cast_discovering = false;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if !app.cast_devices.is_empty() {
                    let len = app.cast_devices.len();
                    app.selected_cast_device_index = (app.selected_cast_device_index + 1) % len;
                    app.cast_device_list_state.select(Some(app.selected_cast_device_index));
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if !app.cast_devices.is_empty() {
                    let len = app.cast_devices.len();
                    if app.selected_cast_device_index == 0 {
                        app.selected_cast_device_index = len - 1;
                    } else {
                        app.selected_cast_device_index -= 1;
                    }
                    app.cast_device_list_state.select(Some(app.selected_cast_device_index));
                }
            }
            KeyCode::Char('r') => {
                // Refresh/rescan for devices
                #[cfg(feature = "chromecast")]
                if !app.cast_discovering {
                    app.cast_discovering = true;
                    app.cast_devices.clear();
                    let tx = tx.clone();
                    tokio::spawn(async move {
                        match cast::CastManager::discover_devices(5).await {
                            Ok(devices) => {
                                let _ = tx.send(AsyncAction::CastDevicesDiscovered(devices)).await;
                            }
                            Err(e) => {
                                let _ = tx.send(AsyncAction::CastFailed(format!("Discovery failed: {}", e))).await;
                            }
                        }
                    });
                }
                #[cfg(not(feature = "chromecast"))]
                {
                    let _ = tx.send(AsyncAction::CastFailed("Chromecast support not enabled. Rebuild with --features chromecast".to_string()));
                }
            }
            KeyCode::Enter => {
                #[cfg(feature = "chromecast")]
                if !app.cast_devices.is_empty() && app.selected_cast_device_index < app.cast_devices.len() {
                    let device = app.cast_devices[app.selected_cast_device_index].clone();
                    let device_name = device.name.clone();
                    
                    // Get the pending URL to cast
                    if let Some(url) = app.pending_play_url.take() {
                        let tx = tx.clone();
                        tokio::spawn(async move {
                            let mut manager = cast::CastManager::new();
                            match manager.cast_to_device(&device, &url, None) {
                                Ok(_) => {
                                    let _ = tx.send(AsyncAction::CastStarted(device_name)).await;
                                }
                                Err(e) => {
                                    let _ = tx.send(AsyncAction::CastFailed(e.to_string())).await;
                                }
                            }
                        });
                    }
                    app.show_cast_picker = false;
                }
            }
            _ => {}
        }
        return Ok(InputResult::Continue);
    }

    // Priority 5: Global Error Overlay Dismissal
    if app.login_error.is_some() && app.current_screen != CurrentScreen::Login {
        if key.code == KeyCode::Esc {
            app.login_error = None;
            return Ok(InputResult::Continue);
        }
    }

    // GLOBAL KEYS
    if app.input_mode == InputMode::Normal && !app.search_mode {
        if let KeyCode::Char('q') | KeyCode::Char('Q') = key.code {
            app.should_quit = true;
            return Ok(InputResult::Quit);
        }
        // Refresh Playlist
        if matches!(key.code, KeyCode::Char('r') | KeyCode::Char('R')) {
            if let Some(client) = app.current_client.clone() {
                let tx = tx.clone();
                app.state_loading = true;
                app.loading_message = Some("Refreshing playlist...".to_string());

                app.categories.clear();
                app.all_categories.clear();
                app.streams.clear();
                app.all_streams.clear();
                app.vod_categories.clear();
                app.all_vod_categories.clear();
                app.vod_streams.clear();
                app.all_vod_streams.clear();

                tokio::spawn(async move {
                    let (auth_success, ui, si) = match client.authenticate().await {
                        Ok(r) => r,
                        Err(e) => {
                            let _ = tx.send(AsyncAction::Error(format!("Refresh failed: {}", e))).await;
                            let _ = tx.send(AsyncAction::PlaylistRefreshed(None, None)).await;
                            return;
                        }
                    };

                    if !auth_success {
                        let _ = tx.send(AsyncAction::Error("Refresh authentication failed".to_string())).await;
                        let _ = tx.send(AsyncAction::PlaylistRefreshed(None, None)).await;
                        return;
                    }

                    let _ = tx.send(AsyncAction::PlaylistRefreshed(ui, si)).await;
                });
            }
        }

        // Quick Mode Switch
        if matches!(key.code, KeyCode::Char('m') | KeyCode::Char('M')) {
            if app.current_screen != CurrentScreen::Settings || app.settings_state != SettingsState::PlaylistModeSelection {
                if app.current_screen != CurrentScreen::Settings {
                    app.previous_screen = Some(app.current_screen.clone());
                }
                app.current_screen = CurrentScreen::Settings;
                app.settings_state = SettingsState::PlaylistModeSelection;
                
                // Pre-select current mode
                let modes = crate::config::PlaylistMode::all();
                let idx = modes.iter().position(|m| *m == app.config.playlist_mode).unwrap_or(0);
                app.playlist_mode_list_state.select(Some(idx));
                return Ok(InputResult::Continue);
            }
        }
    }

    if app.should_quit {
        return Ok(InputResult::Quit);
    }

    // SCREEN SPECIFIC
    let account_name = app.config.accounts.get(app.selected_account_index)
                        .map(|a| a.name.clone()).unwrap_or_default();
    match app.current_screen {
        CurrentScreen::Home => {
            match key.code {
                KeyCode::Char('n') => {
                    app.current_screen = CurrentScreen::Login;
                    app.previous_screen = Some(CurrentScreen::Home);
                    app.input_name = tui_input::Input::default();
                    app.input_url = tui_input::Input::default();
                    app.input_username = tui_input::Input::default();
                    app.input_password = tui_input::Input::default();
                    app.input_epg_url = tui_input::Input::default();
                    app.login_error = None;
                    app.editing_account_index = None;
                    app.input_mode = InputMode::Editing; // Auto-start in editing mode
                }
                KeyCode::Char('e') => {
                    if !app.config.accounts.is_empty() {
                        app.editing_account_index = Some(app.selected_account_index);
                        let acc = &app.config.accounts[app.selected_account_index];
                        app.input_name = tui_input::Input::new(acc.name.clone());
                        app.input_url = tui_input::Input::new(acc.base_url.clone());
                        app.input_username = tui_input::Input::new(acc.username.clone());
                        app.input_password = tui_input::Input::new(acc.password.clone());
                        app.input_epg_url = tui_input::Input::new(acc.epg_url.clone().unwrap_or_default());

                        app.current_screen = CurrentScreen::Login;
                        app.previous_screen = Some(CurrentScreen::Home);
                        app.input_mode = InputMode::Editing; // Auto-start in editing mode
                    }
                }
                KeyCode::Char('d') => {
                    if !app.config.accounts.is_empty() {
                        app.config.remove_account(app.selected_account_index);
                        if app.selected_account_index >= app.config.accounts.len() && !app.config.accounts.is_empty() {
                            app.selected_account_index = app.config.accounts.len() - 1;
                            app.account_list_state.select(Some(app.selected_account_index));
                        } else if app.config.accounts.is_empty() {
                            app.selected_account_index = 0;
                            app.account_list_state.select(None);
                        }
                    }
                }
                KeyCode::Char('x') => app.current_screen = CurrentScreen::Settings,
                KeyCode::Char('s') | KeyCode::Char('S') => {
                    app.previous_screen = Some(CurrentScreen::Home);
                    app.current_screen = CurrentScreen::SportsDashboard;
                    app.active_pane = Pane::Categories;
                    app.sports_matches.clear();
                    app.sports_category_list_state.select(Some(0));
                }
                KeyCode::Char('j') | KeyCode::Down => app.next_account(),
                KeyCode::Char('k') | KeyCode::Up => app.previous_account(),
                KeyCode::Char('1') => {
                    app.show_guide = Some(Guide::WhatIsApp);
                    app.guide_scroll = 0;
                }
                KeyCode::Char('2') => {
                    app.show_guide = Some(Guide::HowToGetPlaylists);
                    app.guide_scroll = 0;
                }
                KeyCode::Char('3') => {
                    app.show_guide = Some(Guide::WhatIsIptv);
                    app.guide_scroll = 0;
                }
                KeyCode::Enter => {
                    if !app.config.accounts.is_empty() {
                        let acc = &app.config.accounts[app.selected_account_index];
                        let base_url = acc.base_url.clone();
                        let username = acc.username.clone();
                        let password = acc.password.clone();
                        let now = chrono::Utc::now().timestamp();
                        let needs_refresh = acc.last_refreshed.map(|last| now - last > (5 * 3600)).unwrap_or(true);

                        app.state_loading = true;
                        if needs_refresh {
                            app.loading_message = Some("Refreshing playlist (Data > 5h old)...".to_string());
                        } else {
                            app.loading_message = Some("Loading playlist...".to_string());
                        }

                        app.login_error = None;
                        let tx = tx.clone();
                        let dns_provider = app.config.dns_provider;
                        tokio::spawn(async move {
                            match XtreamClient::new_with_doh(base_url, username, password, dns_provider).await {
                                Ok(client) => match client.authenticate().await {
                                    Ok((true, ui, si)) => {
                                        let _ = tx.send(AsyncAction::LoginSuccess(client, ui, si)).await;
                                    }
                                    Ok((false, _, _)) => {
                                        let _ = tx.send(AsyncAction::LoginFailed("Authentication failed".to_string())).await;
                                    }
                                    Err(e) => {
                                        let _ = tx.send(AsyncAction::LoginFailed(e.to_string())).await;
                                    }
                                },
                                Err(e) => {
                                    let _ = tx.send(AsyncAction::LoginFailed(format!("Connection error: {}", e))).await;
                                }
                            }
                        });
                    }
                }
                _ => {}
            }
        }
        CurrentScreen::ContentTypeSelection => {
            match key.code {
                KeyCode::Char('1') => {
                    app.current_screen = CurrentScreen::Categories;
                    app.active_pane = Pane::Categories;
                    app.search_mode = false;
                    app.search_query.clear();
                    app.update_search();
                }
                KeyCode::Char('2') => {
                    app.current_screen = CurrentScreen::VodCategories;
                    app.active_pane = Pane::Categories;
                    app.search_mode = false;
                    app.search_query.clear();
                    app.update_search();
                }
                KeyCode::Char('3') => {
                    app.current_screen = CurrentScreen::SeriesCategories;
                    app.active_pane = Pane::Categories;
                    app.search_mode = false;
                    app.search_query.clear();
                    app.update_search();
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    app.selected_content_type_index = (app.selected_content_type_index + 1) % 3;
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    if app.selected_content_type_index == 0 {
                        app.selected_content_type_index = 2;
                    } else {
                        app.selected_content_type_index -= 1;
                    }
                }
                KeyCode::Char('s') | KeyCode::Char('S') => {
                    app.current_screen = CurrentScreen::SportsDashboard;
                    app.active_pane = Pane::Categories;
                    app.sports_matches.clear();
                    app.sports_category_list_state.select(Some(0));
                }
                KeyCode::Enter => {
                    match app.selected_content_type_index {
                        0 => {
                            app.current_screen = CurrentScreen::Categories;
                            app.active_pane = Pane::Categories;
                            app.search_mode = false;
                            app.search_query.clear();
                            app.update_search();
                        }
                        1 => {
                            app.current_screen = CurrentScreen::VodCategories;
                            app.active_pane = Pane::Categories;
                            app.search_mode = false;
                            app.search_query.clear();
                            app.update_search();
                        }
                        2 => {
                            app.current_screen = CurrentScreen::SeriesCategories;
                            app.active_pane = Pane::Categories;
                            app.search_mode = false;
                            app.search_query.clear();
                            app.update_search();
                        }
                        _ => {}
                    }
                }
                KeyCode::Esc | KeyCode::Backspace => {
                    app.current_screen = CurrentScreen::Home;
                    app.current_client = None; 
                }
                KeyCode::Char('R') => {
                    // Manual refresh - force full playlist rescan
                    if let Some(client) = &app.current_client {
                        app.loading_message = Some("Refreshing playlist...".to_string());
                        let client = client.clone();
                        let tx = tx.clone();
                        tokio::spawn(async move {
                            match client.authenticate().await {
                                Ok((_, ui, si)) => {
                                    let _ = tx.send(AsyncAction::PlaylistRefreshed(ui, si)).await;
                                }
                                Err(e) => {
                                    let _ = tx.send(AsyncAction::Error(format!("Refresh failed: {}", e))).await;
                                }
                            }
                        });
                    }
                }
                _ => {}
            }
        }
        CurrentScreen::Login => {
            if app.show_save_confirmation {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                        app.save_account();
                        app.show_save_confirmation = false;
                        app.current_screen = app.previous_screen.take().unwrap_or(CurrentScreen::Home);
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') => {
                        app.show_save_confirmation = false;
                        app.current_screen = app.previous_screen.take().unwrap_or(CurrentScreen::Home);
                        app.input_name = tui_input::Input::default();
                        app.input_url = tui_input::Input::default();
                        app.input_username = tui_input::Input::default();
                        app.input_password = tui_input::Input::default();
                        app.input_epg_url = tui_input::Input::default();
                        app.editing_account_index = None;
                    }
                    KeyCode::Esc => {
                        app.show_save_confirmation = false;
                    }
                    _ => {}
                }
            } else {
                match app.input_mode {
                    InputMode::Normal => {
                        match key.code {
                            KeyCode::Esc => {
                                let mut changed = false;
                                let (orig_name, orig_url, orig_user, orig_pass, orig_epg) = if let Some(idx) = app.editing_account_index {
                                    if let Some(acc) = app.config.accounts.get(idx) {
                                        (acc.name.clone(), acc.base_url.clone(), acc.username.clone(), acc.password.clone(), acc.epg_url.clone().unwrap_or_default())
                                    } else {
                                        ("".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string())
                                    }
                                } else {
                                    ("".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string())
                                };

                                if app.input_name.value() != orig_name || app.input_url.value() != orig_url || app.input_username.value() != orig_user || app.input_password.value() != orig_pass || app.input_epg_url.value() != orig_epg {
                                    changed = true;
                                }

                                if app.editing_account_index.is_none() && app.input_name.value().is_empty() && app.input_url.value().is_empty() && app.input_username.value().is_empty() && app.input_password.value().is_empty() && app.input_epg_url.value().is_empty() {
                                    changed = false;
                                }

                                if changed {
                                    app.show_save_confirmation = true;
                                } else {
                                    let return_screen = app.previous_screen.take().unwrap_or(CurrentScreen::Home);
                                    if return_screen == CurrentScreen::Settings {
                                        app.settings_state = SettingsState::ManageAccounts;
                                    }
                                    app.current_screen = return_screen;
                                    app.input_name = tui_input::Input::default();
                                    app.input_url = tui_input::Input::default();
                                    app.input_username = tui_input::Input::default();
                                    app.input_password = tui_input::Input::default();
                                    app.input_epg_url = tui_input::Input::default();
                                    app.editing_account_index = None;
                                    app.login_error = None;
                                    app.input_mode = InputMode::Normal; // Reset on exit
                                }
                            }
                            KeyCode::Char('j') | KeyCode::Down | KeyCode::Tab => {
                                app.login_field_focus = match app.login_field_focus {
                                    LoginField::Name => LoginField::Url,
                                    LoginField::Url => LoginField::Username,
                                    LoginField::Username => LoginField::Password,
                                    LoginField::Password => LoginField::EpgUrl,
                                    LoginField::EpgUrl => LoginField::Name,
                                };
                            }
                            KeyCode::Char('k') | KeyCode::Up => {
                                app.login_field_focus = match app.login_field_focus {
                                    LoginField::Name => LoginField::EpgUrl,
                                    LoginField::Url => LoginField::Name,
                                    LoginField::Username => LoginField::Url,
                                    LoginField::Password => LoginField::Username,
                                    LoginField::EpgUrl => LoginField::Password,
                                };
                            }
                            KeyCode::Enter => app.toggle_input_mode(),
                            _ => {}
                        }
                    }
                    InputMode::Editing => {
                        match key.code {
                            KeyCode::Esc => {
                                app.input_mode = InputMode::Normal;
                                // Return to previous screen
                                let return_screen = app.previous_screen.take().unwrap_or(CurrentScreen::Home);
                                if return_screen == CurrentScreen::Settings {
                                    app.settings_state = SettingsState::ManageAccounts;
                                }
                                app.current_screen = return_screen;
                                app.input_name = tui_input::Input::default();
                                app.input_url = tui_input::Input::default();
                                app.input_username = tui_input::Input::default();
                                app.input_password = tui_input::Input::default();
                                app.input_epg_url = tui_input::Input::default();
                                app.editing_account_index = None;
                                app.login_error = None;
                            }
                            KeyCode::Tab => {
                                // Move to next field without leaving editing mode
                                app.login_field_focus = match app.login_field_focus {
                                    LoginField::Name => LoginField::Url,
                                    LoginField::Url => LoginField::Username,
                                    LoginField::Username => LoginField::Password,
                                    LoginField::Password => LoginField::EpgUrl,
                                    LoginField::EpgUrl => LoginField::Name,
                                };
                            }
                            KeyCode::BackTab => {
                                // Move to previous field
                                app.login_field_focus = match app.login_field_focus {
                                    LoginField::Name => LoginField::EpgUrl,
                                    LoginField::Url => LoginField::Name,
                                    LoginField::Username => LoginField::Url,
                                    LoginField::Password => LoginField::Username,
                                    LoginField::EpgUrl => LoginField::Password,
                                };
                            }
                            KeyCode::Enter => {
                                // On Enter from last field, submit the form
                                if app.login_field_focus == LoginField::EpgUrl {
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
                                            server_timezone: None,
                                        };
                                        if let Some(idx) = app.editing_account_index {
                                            app.config.update_account(idx, acc);
                                        } else {
                                            app.config.add_account(acc);
                                        }

                                        app.current_screen = CurrentScreen::Home;
                                        app.input_name = tui_input::Input::default();
                                        app.input_url = tui_input::Input::default();
                                        app.input_username = tui_input::Input::default();
                                        app.input_password = tui_input::Input::default();
                                        app.input_epg_url = tui_input::Input::default();
                                        app.login_error = None;
                                        app.editing_account_index = None;
                                        app.input_mode = InputMode::Normal;
                                    } else {
                                        app.login_error = Some("Name and URL required".to_string());
                                    }
                                } else {
                                    // Move to next field on Enter
                                    app.login_field_focus = match app.login_field_focus {
                                        LoginField::Name => LoginField::Url,
                                        LoginField::Url => LoginField::Username,
                                        LoginField::Username => LoginField::Password,
                                        LoginField::Password => LoginField::EpgUrl,
                                        LoginField::EpgUrl => LoginField::Name,
                                    };
                                }
                            }
                            _ => {
                                // Handle Ctrl+V paste
                                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('v') {
                                    #[cfg(not(target_arch = "wasm32"))]
                                    {
                                        if let Ok(mut clipboard) = arboard::Clipboard::new() {
                                            if let Ok(text) = clipboard.get_text() {
                                                // Paste into the currently focused field
                                                match app.login_field_focus {
                                                    LoginField::Name => {
                                                        let current = app.input_name.value().to_string();
                                                        app.input_name = tui_input::Input::new(current + &text);
                                                    }
                                                    LoginField::Url => {
                                                        let current = app.input_url.value().to_string();
                                                        app.input_url = tui_input::Input::new(current + &text);
                                                    }
                                                    LoginField::Username => {
                                                        let current = app.input_username.value().to_string();
                                                        app.input_username = tui_input::Input::new(current + &text);
                                                    }
                                                    LoginField::Password => {
                                                        let current = app.input_password.value().to_string();
                                                        app.input_password = tui_input::Input::new(current + &text);
                                                    }
                                                    LoginField::EpgUrl => {
                                                        let current = app.input_epg_url.value().to_string();
                                                        app.input_epg_url = tui_input::Input::new(current + &text);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    match app.login_field_focus {
                                        LoginField::Name => { app.input_name.handle_event(&Event::Key(key)); }
                                        LoginField::Url => { app.input_url.handle_event(&Event::Key(key)); }
                                        LoginField::Username => { app.input_username.handle_event(&Event::Key(key)); }
                                        LoginField::Password => { app.input_password.handle_event(&Event::Key(key)); }
                                        LoginField::EpgUrl => { app.input_epg_url.handle_event(&Event::Key(key)); }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        CurrentScreen::Categories | CurrentScreen::Streams => {
            if app.search_mode {
                match key.code {
                    KeyCode::Esc => {
                        app.search_mode = false;
                        app.search_query.clear();
                        app.update_search();
                    }
                    KeyCode::Enter => {
                        app.search_mode = false;
                    }
                    KeyCode::Backspace => {
                        app.search_query.pop();
                        app.update_search();
                    }
                    KeyCode::Char(c) => {
                        app.search_query.push(c);
                        app.update_search();
                    }
                    _ => {}
                }
            } else {
                match key.code {
                    KeyCode::Char('/') | KeyCode::Char('f') => {
                        app.search_mode = true;
                        app.search_query.clear();
                        app.update_search();
                    }
                    KeyCode::Esc | KeyCode::Backspace => {
                        if app.active_pane == Pane::Streams && !app.streams.is_empty() {
                            app.active_pane = Pane::Categories;
                            app.streams.clear();
                            app.all_streams.clear();
                            app.selected_stream_index = 0;
                            app.stream_list_state.select(None);
                            app.search_mode = false;
                            app.search_query.clear();
                        } else {
                            app.streams.clear();
                            app.all_streams.clear();
                            app.selected_stream_index = 0;
                            app.selected_category_index = 0;
                            app.stream_list_state.select(None);
                            app.category_list_state.select(None);
                            app.search_mode = false;
                            app.search_query.clear();
                            app.current_screen = CurrentScreen::ContentTypeSelection;
                        }
                    }
                    KeyCode::Tab => {
                        match app.active_pane {
                            Pane::Categories => { if !app.streams.is_empty() { app.active_pane = Pane::Streams; } }
                            Pane::Streams => { app.active_pane = Pane::Categories; }
                            _ => {}
                        }
                    }
                    KeyCode::Left | KeyCode::Char('c') => { app.active_pane = Pane::Categories; }
                    KeyCode::Right | KeyCode::Char('s') => { if !app.streams.is_empty() { app.active_pane = Pane::Streams; } }
                    KeyCode::Char('j') | KeyCode::Down => match app.active_pane {
                        Pane::Categories => app.next_category(),
                        Pane::Streams => app.next_stream(),
                        _ => {}
                    },
                    KeyCode::Char('k') | KeyCode::Up => match app.active_pane {
                        Pane::Categories => app.previous_category(),
                        Pane::Streams => app.previous_stream(),
                        _ => {}
                    },
                    KeyCode::Char('v') => match app.active_pane {
                        Pane::Categories => {
                            if !app.categories.is_empty() {
                                let id = app.categories[app.selected_category_index].category_id.clone();
                                app.config.toggle_favorite_category(id);
                                app.categories.sort_by(|a, b| {
                                    let a_fav = app.config.favorites.categories.contains(&a.category_id);
                                    let b_fav = app.config.favorites.categories.contains(&b.category_id);
                                    if a.category_id == "ALL" { return std::cmp::Ordering::Less; }
                                    if b.category_id == "ALL" { return std::cmp::Ordering::Greater; }
                                    match (a_fav, b_fav) {
                                        (true, false) => std::cmp::Ordering::Less,
                                        (false, true) => std::cmp::Ordering::Greater,
                                        _ => a.category_name.cmp(&b.category_name),
                                    }
                                });
                                app.all_categories = app.categories.clone();
                            }
                        }
                        Pane::Streams => {
                            if !app.streams.is_empty() {
                                let stream = &app.streams[app.selected_stream_index];
                                let id = get_id_str(&stream.stream_id);
                                app.config.toggle_favorite_stream(id);
                                app.streams.sort_by(|a, b| {
                                    let a_id = get_id_str(&a.stream_id);
                                    let b_id = get_id_str(&b.stream_id);
                                    let a_fav = app.config.favorites.streams.contains(&a_id);
                                    let b_fav = app.config.favorites.streams.contains(&b_id);
                                    match (a_fav, b_fav) {
                                        (true, false) => std::cmp::Ordering::Less,
                                        (false, true) => std::cmp::Ordering::Greater,
                                        _ => a.name.cmp(&b.name),
                                    }
                                });
                                app.all_streams = app.streams.clone();
                            }
                        }
                        _ => {}
                    },
                    KeyCode::Enter => {
                        match app.active_pane {
                            Pane::Categories => {
                                if !app.categories.is_empty() {
                                    let cat_id = app.categories[app.selected_category_index].category_id.clone();
                                    if cat_id == "ALL" && !app.global_all_streams.is_empty() {
                                        app.all_streams = app.global_all_streams.clone();
                                        app.streams = app.all_streams.clone();
                                        app.current_screen = CurrentScreen::Streams;
                                        app.active_pane = Pane::Streams;
                                        app.selected_stream_index = 0;
                                        app.stream_list_state.select(Some(0));
                                    } else if let Some(client) = &app.current_client {
                                        let client = client.clone();
                                        let tx = tx.clone();
                                        let pms = app.config.processing_modes.clone();
                                        let favs = app.config.favorites.streams.clone();
                                        let account_name = account_name.clone();
                                        app.state_loading = true;
                                        app.loading_message = Some("Initializing Request...".to_string());
                                        tokio::spawn(async move {
                                            let _ = tx.send(AsyncAction::LoadingMessage("Fetching Live Streams...".to_string())).await;
                                            match client.get_live_streams(&cat_id).await {
                                                Ok(mut streams) => {
                                                    let _ = tx.send(AsyncAction::LoadingMessage(format!("Processing {} Streams...", streams.len()))).await;
                                                    preprocessing::preprocess_streams(&mut streams, &favs, &pms, true, &account_name);
                                                    let _ = tx.send(AsyncAction::StreamsLoaded(streams, cat_id)).await;
                                                }
                                                Err(e) => { let _ = tx.send(AsyncAction::Error(e.to_string())).await; }
                                            }
                                        });
                                    }
                                }
                            }
                            Pane::Streams => {
                                if !app.streams.is_empty() {
                                    let stream = &app.streams[app.selected_stream_index];
                                    if let Some(client) = &app.current_client {
                                        let id = get_id_str(&stream.stream_id);
                                        let url = client.get_stream_url(&id, "ts");
                                        app.state_loading = true;
                                        app.player_error = None;
                                        app.loading_message = Some(format!("Preparing Live Stream: {}...", stream.name));
                                        let tx = tx.clone();
                                        let player = player.clone();
                                        let stream_url = url.clone();
                                        let use_default = app.config.use_default_mpv;
                                        tokio::spawn(async move {
                                            let _ = tx.send(AsyncAction::LoadingMessage("Connecting to stream server...".to_string())).await;
                                            match player.play(&stream_url, use_default) {
                                                Ok(_) => {
                                                    let _ = tx.send(AsyncAction::LoadingMessage("Handshaking with player...".to_string())).await;
                                                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                                                    let _ = tx.send(AsyncAction::LoadingMessage("Buffering video stream...".to_string())).await;
                                                    match player.wait_for_playback(10000).await {
                                                        Ok(true) => { let _ = tx.send(AsyncAction::PlayerStarted).await; }
                                                        Ok(false) => { let _ = tx.send(AsyncAction::PlayerFailed("Stream failed to start - MPV exited unexpectedly".to_string())).await; }
                                                        Err(e) => { let _ = tx.send(AsyncAction::PlayerFailed(format!("Playback error: {}", e))).await; }
                                                    }
                                                }
                                                Err(e) => { let _ = tx.send(AsyncAction::PlayerFailed(e.to_string())).await; }
                                            }
                                        });
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    KeyCode::Char('x') => { app.current_screen = CurrentScreen::Settings }
                    KeyCode::Char('g') => {
                        // Add to group - open group picker
                        if app.active_pane == Pane::Streams && !app.streams.is_empty() {
                            let stream = &app.streams[app.selected_stream_index];
                            let stream_id = get_id_str(&stream.stream_id);
                            app.pending_stream_for_group = Some(stream_id);
                            app.selected_group_index = 0;
                            app.group_list_state.select(Some(0));
                            app.previous_screen = Some(app.current_screen.clone());
                            app.current_screen = CurrentScreen::GroupPicker;
                        }
                    }
                    KeyCode::Char('G') => {
                        // Open group management
                        app.previous_screen = Some(app.current_screen.clone());
                        app.selected_group_index = 0;
                        app.group_list_state.select(if app.config.favorites.groups.is_empty() { None } else { Some(0) });
                        app.current_screen = CurrentScreen::GroupManagement;
                    }
                    #[cfg(feature = "chromecast")]
                    KeyCode::Char('C') => {
                        // Cast to Chromecast - open device picker
                        if app.active_pane == Pane::Streams && !app.streams.is_empty() {
                            let stream = &app.streams[app.selected_stream_index];
                            if let Some(client) = &app.current_client {
                                let id = get_id_str(&stream.stream_id);
                                let url = client.get_stream_url(&id, "ts");
                                app.pending_play_url = Some(url);
                                app.pending_play_title = Some(stream.name.clone());
                                app.show_cast_picker = true;
                                app.cast_discovering = true;
                                app.selected_cast_device_index = 0;
                                app.cast_device_list_state.select(None);
                                
                                // Start device discovery
                                let tx = tx.clone();
                                tokio::spawn(async move {
                                    match cast::CastManager::discover_devices(5).await {
                                        Ok(devices) => {
                                            let _ = tx.send(AsyncAction::CastDevicesDiscovered(devices)).await;
                                        }
                                        Err(e) => {
                                            let _ = tx.send(AsyncAction::CastFailed(format!("Discovery failed: {}", e))).await;
                                        }
                                    }
                                });
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        CurrentScreen::VodCategories => {
            if app.search_mode {
                match key.code {
                    KeyCode::Esc => {
                        app.search_mode = false;
                        app.search_query.clear();
                        app.update_search();
                    }
                    KeyCode::Enter => { app.search_mode = false; }
                    KeyCode::Backspace => { app.search_query.pop(); app.update_search(); }
                    KeyCode::Char(c) => { app.search_query.push(c); app.update_search(); }
                    _ => {}
                }
            } else {
                match key.code {
                    KeyCode::Char('/') | KeyCode::Char('f') => {
                        app.search_mode = true;
                        app.active_pane = Pane::Categories;
                        app.search_query.clear();
                        app.update_search();
                    }
                    KeyCode::Esc | KeyCode::Backspace => {
                        app.vod_streams.clear();
                        app.all_vod_streams.clear();
                        app.selected_vod_category_index = 0;
                        app.selected_vod_stream_index = 0;
                        app.vod_category_list_state.select(None);
                        app.vod_stream_list_state.select(None);
                        app.search_mode = false;
                        app.search_query.clear();
                        app.current_screen = CurrentScreen::ContentTypeSelection;
                    }
                    KeyCode::Char('j') | KeyCode::Down => app.next_vod_category(),
                    KeyCode::Char('k') | KeyCode::Up => app.previous_vod_category(),
                    KeyCode::Enter => {
                        if !app.vod_categories.is_empty() {
                            let cat_id = app.vod_categories[app.selected_vod_category_index].category_id.clone();
                            if cat_id == "ALL" && !app.global_all_vod_streams.is_empty() {
                                app.all_vod_streams = app.global_all_vod_streams.clone();
                                let mut display_streams = app.all_vod_streams.clone();
                                display_streams.truncate(1000);
                                app.vod_streams = display_streams;
                                app.current_screen = CurrentScreen::VodStreams;
                                app.active_pane = Pane::Streams;
                                app.selected_vod_stream_index = 0;
                                app.vod_stream_list_state.select(Some(0));
                            } else if let Some(client) = &app.current_client {
                                let client = client.clone();
                                let tx = tx.clone();
                                let pms = app.config.processing_modes.clone();
                                let favs = app.config.favorites.vod_streams.clone();
                                let account_name = account_name.clone();
                                app.state_loading = true;
                                tokio::spawn(async move {
                                    if cat_id == "ALL" {
                                        match client.get_vod_streams_all().await {
                                            Ok(mut streams) => {
                                                preprocessing::preprocess_streams(&mut streams, &favs, &pms, false, &account_name);
                                                let _ = tx.send(AsyncAction::VodStreamsLoaded(streams, cat_id)).await;
                                            }
                                            Err(e) => { let _ = tx.send(AsyncAction::Error(e.to_string())).await; }
                                        }
                                    } else {
                                        match client.get_vod_streams(&cat_id).await {
                                            Ok(mut streams) => {
                                                preprocessing::preprocess_streams(&mut streams, &favs, &pms, false, &account_name);
                                                let _ = tx.send(AsyncAction::VodStreamsLoaded(streams, cat_id)).await;
                                            }
                                            Err(e) => { let _ = tx.send(AsyncAction::Error(e.to_string())).await; }
                                        }
                                    }
                                });
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        CurrentScreen::VodStreams => {
            if app.search_mode {
                match key.code {
                    KeyCode::Esc => { app.search_mode = false; app.search_query.clear(); app.update_search(); }
                    KeyCode::Enter => { app.search_mode = false; }
                    KeyCode::Backspace => { app.search_query.pop(); app.update_search(); }
                    KeyCode::Char(c) => { app.search_query.push(c); app.update_search(); }
                    _ => {}
                }
            } else {
                match key.code {
                    KeyCode::Char('/') | KeyCode::Char('f') => { app.search_mode = true; app.active_pane = Pane::Streams; app.search_query.clear(); app.update_search(); }
                    KeyCode::Esc | KeyCode::Backspace => {
                        app.vod_streams.clear();
                        app.all_vod_streams.clear();
                        app.selected_vod_stream_index = 0;
                        app.vod_stream_list_state.select(None);
                        app.current_screen = CurrentScreen::VodCategories;
                    }
                    KeyCode::Char('j') | KeyCode::Down => app.next_vod_stream(),
                    KeyCode::Char('k') | KeyCode::Up => app.previous_vod_stream(),
                    KeyCode::Left => {
                        app.vod_streams.clear();
                        app.all_vod_streams.clear();
                        app.selected_vod_stream_index = 0;
                        app.vod_stream_list_state.select(None);
                        app.current_screen = CurrentScreen::VodCategories;
                    }
                    KeyCode::Enter => {
                        if !app.vod_streams.is_empty() {
                            let stream = &app.vod_streams[app.selected_vod_stream_index];
                            if let Some(client) = &app.current_client {
                                let id = get_id_str(&stream.stream_id);
                                let extension = stream.container_extension.as_deref().unwrap_or("mp4");
                                let url = client.get_vod_url(&id, extension);
                                app.pending_play_url = Some(url);
                                app.pending_play_title = Some(stream.name.clone());
                                app.show_play_details = true;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        CurrentScreen::SeriesCategories => {
            if app.search_mode {
                match key.code {
                    KeyCode::Esc => { app.search_mode = false; app.search_query.clear(); app.series_categories = app.all_series_categories.clone(); app.series_streams = app.all_series_streams.clone(); }
                    KeyCode::Enter => { app.search_mode = false; }
                    KeyCode::Backspace => { app.search_query.pop(); app.update_search(); }
                    KeyCode::Char(c) => { app.search_query.push(c); app.update_search(); }
                    _ => {}
                }
            } else {
                match key.code {
                    KeyCode::Char('/') | KeyCode::Char('f') => { app.search_mode = true; app.search_query.clear(); app.update_search(); }
                    KeyCode::Esc | KeyCode::Backspace => {
                        app.series_streams.clear();
                        app.all_series_streams.clear();
                        app.selected_series_category_index = 0;
                        app.selected_series_stream_index = 0;
                        app.series_category_list_state.select(None);
                        app.series_stream_list_state.select(None);
                        app.search_mode = false;
                        app.search_query.clear();
                        app.current_screen = CurrentScreen::ContentTypeSelection;
                    }
                    KeyCode::Char('j') | KeyCode::Down => app.next_series_category(),
                    KeyCode::Char('k') | KeyCode::Up => app.previous_series_category(),
                    KeyCode::Enter => {
                        if !app.series_categories.is_empty() {
                            let cat_id = app.series_categories[app.selected_series_category_index].category_id.clone();
                            if cat_id == "ALL" && !app.global_all_series_streams.is_empty() {
                                app.all_series_streams = app.global_all_series_streams.clone();
                                let mut display_streams = app.all_series_streams.clone();
                                display_streams.truncate(1000);
                                app.series_streams = display_streams;
                                app.current_screen = CurrentScreen::SeriesStreams;
                                app.active_pane = Pane::Streams;
                                app.selected_series_stream_index = 0;
                                app.series_stream_list_state.select(Some(0));
                            } else if let Some(client) = &app.current_client {
                                let client = client.clone();
                                let tx = tx.clone();
                                let pms = app.config.processing_modes.clone();
                                let favs = app.config.favorites.vod_streams.clone();
                                app.state_loading = true;
                                app.active_pane = Pane::Streams;
                                let acc_name_cloned = account_name.clone();
                                tokio::spawn(async move {
                                    match client.get_series_streams(&cat_id).await {
                                        Ok(mut streams) => {
                                            preprocessing::preprocess_streams(&mut streams, &favs, &pms, false, &acc_name_cloned);
                                            let _ = tx.send(AsyncAction::SeriesStreamsLoaded(streams, cat_id)).await;
                                        }
                                        Err(e) => { let _ = tx.send(AsyncAction::Error(e.to_string())).await; }
                                    }
                                });
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        CurrentScreen::SeriesStreams => {
             if app.search_mode {
                match key.code {
                    KeyCode::Esc => { app.search_mode = false; app.search_query.clear(); app.series_categories = app.all_series_categories.clone(); app.series_streams = app.all_series_streams.clone(); }
                    KeyCode::Enter => { app.search_mode = false; }
                    KeyCode::Backspace => { app.search_query.pop(); app.update_search(); }
                    KeyCode::Char(c) => { app.search_query.push(c); app.update_search(); }
                    _ => {}
                }
            } else {
                 match key.code {
                    KeyCode::Char('/') | KeyCode::Char('f') => { app.search_mode = true; app.search_query.clear(); app.update_search(); }
                    KeyCode::Esc | KeyCode::Backspace | KeyCode::Left => {
                        match app.active_pane {
                            Pane::Episodes => { app.series_episodes.clear(); app.selected_series_episode_index = 0; app.series_episode_list_state.select(None); app.active_pane = Pane::Streams; app.search_mode = false; app.search_query.clear(); }
                            Pane::Streams => { app.series_streams.clear(); app.all_series_streams.clear(); app.selected_series_stream_index = 0; app.series_stream_list_state.select(None); app.active_pane = Pane::Categories; app.search_mode = false; app.search_query.clear(); }
                            Pane::Categories => { app.series_streams.clear(); app.all_series_streams.clear(); app.selected_series_stream_index = 0; app.series_stream_list_state.select(None); app.search_mode = false; app.search_query.clear(); app.current_screen = CurrentScreen::SeriesCategories; }
                        }
                    }
                    KeyCode::Char('j') | KeyCode::Down => match app.active_pane {
                        Pane::Categories => app.next_series_category(),
                        Pane::Streams => app.next_series_stream(),
                        Pane::Episodes => app.next_series_episode(),
                    },
                    KeyCode::Char('k') | KeyCode::Up => match app.active_pane {
                        Pane::Categories => app.previous_series_category(),
                        Pane::Streams => app.previous_series_stream(),
                        Pane::Episodes => app.previous_series_episode(),
                    },
                    KeyCode::Enter => match app.active_pane {
                        Pane::Categories => {
                             if !app.series_categories.is_empty() {
                                let cat_id = app.series_categories[app.selected_series_category_index].category_id.clone();
                                if cat_id == "ALL" && !app.global_all_series_streams.is_empty() {
                                    app.all_series_streams = app.global_all_series_streams.clone();
                                    app.series_streams = app.all_series_streams.clone();
                                    app.active_pane = Pane::Streams;
                                    app.selected_series_stream_index = 0;
                                    app.series_stream_list_state.select(Some(0));
                                } else if let Some(client) = &app.current_client {
                                    let client = client.clone();
                                    let tx = tx.clone();
                                    let pms = app.config.processing_modes.clone();
                                    let favs = app.config.favorites.vod_streams.clone();
                                    app.state_loading = true;
                                    app.active_pane = Pane::Streams;
                                    let acc_name_cloned = account_name.clone();
                                    tokio::spawn(async move {
                                        match client.get_series_streams(&cat_id).await {
                                            Ok(mut streams) => {
                                                preprocessing::preprocess_streams(&mut streams, &favs, &pms, false, &acc_name_cloned);
                                                let _ = tx.send(AsyncAction::SeriesStreamsLoaded(streams, cat_id)).await;
                                            }
                                            Err(e) => { let _ = tx.send(AsyncAction::Error(e.to_string())).await; }
                                        }
                                    });
                                }
                            }
                        }
                        Pane::Streams => {
                            if !app.series_streams.is_empty() {
                                let stream = &app.series_streams[app.selected_series_stream_index];
                                if let Some(client) = &app.current_client {
                                    let id = get_id_str(&stream.stream_id);
                                    app.state_loading = true;
                                    app.active_pane = Pane::Episodes;
                                    let tx = tx.clone();
                                    let client = client.clone();
                                    tokio::spawn(async move {
                                        match client.get_series_info(&id).await {
                                            Ok(info) => { let _ = tx.send(AsyncAction::SeriesInfoLoaded(info)).await; }
                                            Err(e) => { let _ = tx.send(AsyncAction::Error(e.to_string())).await; }
                                        }
                                    });
                                }
                            }
                        }
                        Pane::Episodes => {
                             if !app.series_episodes.is_empty() {
                                let episode = &app.series_episodes[app.selected_series_episode_index];
                                if let Some(client) = &app.current_client {
                                    let id = episode.id.as_ref().map(|v| get_id_str(v)).unwrap_or_default();
                                    if !id.is_empty() {
                                        let ext = episode.container_extension.as_deref().unwrap_or("mp4");
                                        let url = client.get_series_url(&id, ext);
                                        app.pending_play_url = Some(url);
                                        app.pending_play_title = episode.title.clone();
                                        app.show_play_details = true;
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        CurrentScreen::GlobalSearch => {
            if app.search_mode {
                match key.code {
                    KeyCode::Esc => { 
                        app.search_mode = false; 
                        app.search_query.clear(); 
                        app.global_search_results.clear(); 
                        app.current_screen = app.previous_screen.clone().unwrap_or(CurrentScreen::Home);
                    }
                    KeyCode::Enter => { app.search_mode = false; }
                    KeyCode::Backspace => { app.search_query.pop(); app.update_search(); }
                    KeyCode::Char(c) => { app.search_query.push(c); app.update_search(); }
                    _ => {}
                }
            } else {
                match key.code {
                    KeyCode::Char('/') | KeyCode::Char('f') => { app.search_mode = true; app.search_query.clear(); app.update_search(); }
                    KeyCode::Esc | KeyCode::Backspace => {
                        app.search_mode = false;
                        app.search_query.clear();
                        app.global_search_results.clear();
                        app.current_screen = app.previous_screen.clone().unwrap_or(CurrentScreen::Home);
                    }
                    KeyCode::Char('j') | KeyCode::Down => app.next_global_search_result(),
                    KeyCode::Char('k') | KeyCode::Up => app.previous_global_search_result(),
                    KeyCode::Enter => {
                        if !app.global_search_results.is_empty() {
                            let stream = &app.global_search_results[app.selected_stream_index];
                            if let Some(client) = &app.current_client {
                                let id = get_id_str(&stream.stream_id);
                                let extension = stream.container_extension.as_deref().unwrap_or("ts");
                                
                                let url = match stream.stream_type.as_str() {
                                    "movie" => client.get_vod_url(&id, extension),
                                    "series" => client.get_series_url(&id, extension),
                                    _ => client.get_stream_url(&id, extension),
                                };

                                if stream.stream_type == "movie" || stream.stream_type == "series" {
                                    app.pending_play_url = Some(url);
                                    app.pending_play_title = Some(stream.name.clone());
                                    app.show_play_details = true;
                                } else {
                                    app.state_loading = true;
                                    app.player_error = None;
                                    app.loading_message = Some(format!("Preparing: {}...", stream.name));
                                    let tx = tx.clone();
                                    let player = player.clone();
                                    let stream_url = url.clone();
                                    let use_default = app.config.use_default_mpv;
                                    tokio::spawn(async move {
                                        let _ = tx.send(AsyncAction::LoadingMessage("Connecting to stream...".to_string())).await;
                                        match player.play(&stream_url, use_default) {
                                            Ok(_) => {
                                                let _ = tx.send(AsyncAction::LoadingMessage("Buffering...".to_string())).await;
                                                match player.wait_for_playback(10000).await {
                                                    Ok(true) => { let _ = tx.send(AsyncAction::PlayerStarted).await; }
                                                    Ok(false) => { let _ = tx.send(AsyncAction::PlayerFailed("Failed to start - MPV exited unexpectedly".to_string())).await; }
                                                    Err(e) => { let _ = tx.send(AsyncAction::PlayerFailed(format!("Playback error: {}", e))).await; }
                                                }
                                            }
                                            Err(e) => { let _ = tx.send(AsyncAction::PlayerFailed(e.to_string())).await; }
                                        }
                                    });
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        CurrentScreen::Settings => {
            match app.settings_state {
                SettingsState::Main => match key.code {
                    KeyCode::Esc | KeyCode::Backspace => app.current_screen = CurrentScreen::Home,
                    KeyCode::Char('j') | KeyCode::Down => app.next_setting(),
                    KeyCode::Char('k') | KeyCode::Up => app.previous_setting(),
                    KeyCode::Enter => {
                         match app.selected_settings_index {
                            0 => { 
                                app.settings_state = SettingsState::ManageAccounts;
                                // Initialize list selection so user can navigate
                                if !app.config.accounts.is_empty() {
                                    app.account_list_state.select(Some(0));
                                    app.selected_account_index = 0;
                                }
                            }
                            1 => { 
                                app.current_screen = CurrentScreen::TimezoneSettings;
                                // Pre-select current timezone in list
                                let current_tz = app.config.timezone.clone().unwrap_or_else(|| "UTC".to_string());
                                let idx = app.timezone_list.iter().position(|t| t == &current_tz).unwrap_or(0);
                                app.timezone_list_state.select(Some(idx));
                            }
                            2 => { 
                                // Open Playlist Mode Selection dropdown
                                app.settings_state = SettingsState::PlaylistModeSelection;
                                // Pre-select current mode
                                let modes = crate::config::PlaylistMode::all();
                                let idx = modes.iter().position(|m| *m == app.config.playlist_mode).unwrap_or(0);
                                app.playlist_mode_list_state.select(Some(idx));
                            }
                            3 => { 
                                // Open DNS selection dropdown
                                app.settings_state = SettingsState::DnsSelection;
                                // Pre-select current DNS provider
                                let providers = crate::config::DnsProvider::all();
                                let idx = providers.iter().position(|p| *p == app.config.dns_provider).unwrap_or(0);
                                app.dns_list_state.select(Some(idx));
                            }
                            4 => { 
                                // Open Video Mode selection dropdown
                                app.settings_state = SettingsState::VideoModeSelection;
                                // Pre-select current video mode (0 = Enhanced, 1 = MPV Default)
                                let idx = if app.config.use_default_mpv { 1 } else { 0 };
                                app.video_mode_list_state.select(Some(idx));
                            }
                            5 => { 
                                // Open Auto-Refresh selection
                                app.settings_state = SettingsState::AutoRefreshSelection;
                                // Options: 0=Off, 1=6h, 2=12h, 3=24h, 4=48h
                                let idx = match app.config.auto_refresh_hours {
                                    0 => 0,
                                    6 => 1,
                                    12 => 2,
                                    24 => 3,
                                    48 => 4,
                                    _ => 2, // Default to 12h
                                };
                                app.auto_refresh_list_state.select(Some(idx));
                            }
                            6 => { 
                                // Enable Matrix Rain Screensaver
                                app.show_matrix_rain = true;
                                app.matrix_rain_screensaver_mode = true;
                                app.matrix_rain_start_time = None;
                                app.matrix_rain_columns.clear();
                            }
                            7 => { 
                                app.state_loading = true;
                                app.loading_message = Some("Checking for updates...".to_string());
                                let tx = tx.clone();
                                tokio::spawn(async move {
                                    crate::setup::check_for_updates(tx, true).await;
                                });
                            }
                            8 => { app.settings_state = SettingsState::About; }
                            _ => {}
                        }
                    }
                    _ => {}
                }
                SettingsState::ManageAccounts => match key.code {
                    KeyCode::Esc | KeyCode::Backspace => { app.settings_state = SettingsState::Main; }
                    KeyCode::Char('j') | KeyCode::Down => app.next_account(),
                    KeyCode::Char('k') | KeyCode::Up => app.previous_account(),
                    KeyCode::Char('a') => {
                        // Add new playlist - clear form and open Login screen
                        app.input_name = tui_input::Input::default();
                        app.input_url = tui_input::Input::default();
                        app.input_username = tui_input::Input::default();
                        app.input_password = tui_input::Input::default();
                        app.input_epg_url = tui_input::Input::default();
                        app.input_server_timezone = tui_input::Input::default();
                        app.editing_account_index = None; // None = adding new
                        app.previous_screen = Some(CurrentScreen::Settings);
                        app.current_screen = CurrentScreen::Login;
                        app.login_field_focus = LoginField::Name;
                        app.input_mode = InputMode::Normal; // Start in navigation mode
                        app.login_error = None;
                    }
                    KeyCode::Enter => {
                        // Open edit form for selected playlist
                        if !app.config.accounts.is_empty() && app.selected_account_index < app.config.accounts.len() {
                            let account = &app.config.accounts[app.selected_account_index];
                            app.input_name = tui_input::Input::new(account.name.clone());
                            app.input_url = tui_input::Input::new(account.base_url.clone());
                            app.input_username = tui_input::Input::new(account.username.clone());
                            app.input_password = tui_input::Input::new(account.password.clone());
                            app.input_epg_url = tui_input::Input::new(account.epg_url.clone().unwrap_or_default());
                            app.input_server_timezone = tui_input::Input::new(account.server_timezone.clone().unwrap_or_default());
                            app.editing_account_index = Some(app.selected_account_index);
                            app.previous_screen = Some(CurrentScreen::Settings); // Return to Settings on Esc
                            app.current_screen = CurrentScreen::Login;
                            app.login_field_focus = LoginField::Name;
                        }
                    }
                    KeyCode::Char('d') | KeyCode::Delete => {
                        // Delete selected playlist
                        if !app.config.accounts.is_empty() && app.selected_account_index < app.config.accounts.len() {
                            app.config.accounts.remove(app.selected_account_index);
                            let _ = app.config.save();
                            if app.selected_account_index > 0 {
                                app.selected_account_index -= 1;
                            }
                            app.account_list_state.select(Some(app.selected_account_index.min(app.config.accounts.len().saturating_sub(1))));
                        }
                    }
                    _ => {}
                }
                SettingsState::About => match key.code {
                    KeyCode::Esc | KeyCode::Backspace => { app.settings_state = SettingsState::Main; app.about_scroll = 0; }
                    KeyCode::Down | KeyCode::Char('j') => { app.about_scroll = app.about_scroll.saturating_add(1) }
                    KeyCode::Up | KeyCode::Char('k') => { app.about_scroll = app.about_scroll.saturating_sub(1) }
                    _ => {}
                }
                SettingsState::DnsSelection => match key.code {
                    KeyCode::Esc | KeyCode::Backspace => { app.settings_state = SettingsState::Main; }
                    KeyCode::Up | KeyCode::Char('k') => {
                        let providers = crate::config::DnsProvider::all();
                        if let Some(idx) = app.dns_list_state.selected() {
                            let new_idx = if idx == 0 { providers.len() - 1 } else { idx - 1 };
                            app.dns_list_state.select(Some(new_idx));
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        let providers = crate::config::DnsProvider::all();
                        if let Some(idx) = app.dns_list_state.selected() {
                            let new_idx = if idx >= providers.len() - 1 { 0 } else { idx + 1 };
                            app.dns_list_state.select(Some(new_idx));
                        }
                    }
                    KeyCode::Enter => {
                        let providers = crate::config::DnsProvider::all();
                        if let Some(idx) = app.dns_list_state.selected() {
                            if idx < providers.len() {
                                app.config.dns_provider = providers[idx];
                                let _ = app.config.save();
                            }
                        }
                        app.settings_state = SettingsState::Main;
                        app.refresh_settings_options();
                    }
                    _ => {}
                }
                SettingsState::VideoModeSelection => match key.code {
                    KeyCode::Esc | KeyCode::Backspace => { app.settings_state = SettingsState::Main; }
                    KeyCode::Up | KeyCode::Char('k') => {
                        if let Some(idx) = app.video_mode_list_state.selected() {
                            let new_idx = if idx == 0 { 1 } else { idx - 1 };
                            app.video_mode_list_state.select(Some(new_idx));
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if let Some(idx) = app.video_mode_list_state.selected() {
                            let new_idx = if idx >= 1 { 0 } else { idx + 1 };
                            app.video_mode_list_state.select(Some(new_idx));
                        }
                    }
                    KeyCode::Enter => {
                        if let Some(idx) = app.video_mode_list_state.selected() {
                            app.config.use_default_mpv = idx == 1; // 0 = Enhanced, 1 = MPV Default
                            let _ = app.config.save();
                        }
                        app.settings_state = SettingsState::Main;
                        app.refresh_settings_options();
                    }
                    _ => {}
                }
                SettingsState::PlaylistModeSelection => match key.code {
                    KeyCode::Esc | KeyCode::Backspace => { app.settings_state = SettingsState::Main; }
                    KeyCode::Up | KeyCode::Char('k') => {
                        let modes = crate::config::ProcessingMode::all();
                        // modes.len() items + 1 Done button = modes.len() + 1 total items
                        let total_items = modes.len() + 1;
                        if let Some(idx) = app.playlist_mode_list_state.selected() {
                            let new_idx = if idx == 0 { total_items - 1 } else { idx - 1 };
                            app.playlist_mode_list_state.select(Some(new_idx));
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        let modes = crate::config::ProcessingMode::all();
                        let total_items = modes.len() + 1;
                        if let Some(idx) = app.playlist_mode_list_state.selected() {
                            let new_idx = if idx >= total_items - 1 { 0 } else { idx + 1 };
                            app.playlist_mode_list_state.select(Some(new_idx));
                        }
                    }
                    KeyCode::Char(' ') | KeyCode::Enter => {
                        let modes = crate::config::ProcessingMode::all();
                        if let Some(idx) = app.playlist_mode_list_state.selected() {
                            if idx < modes.len() {
                                // Toggle Selection
                                if let Some(mode) = modes.get(idx) {
                                    if app.config.processing_modes.contains(mode) {
                                        app.config.processing_modes.retain(|m| m != mode);
                                    } else {
                                        app.config.processing_modes.push(*mode);
                                    }
                                    let _ = app.config.save(); // Optional: Auto-save on toggle?
                                }
                            } else {
                                // Clicked "APPLY & SAVE"
                                let _ = app.config.save();

                                // Exit settings back to wherever we were
                                let return_screen = app.previous_screen.take().unwrap_or(CurrentScreen::Home);
                                app.current_screen = return_screen.clone();
                                app.settings_state = SettingsState::Main;
                                app.refresh_settings_options();

                                // Trigger Refresh
                                if app.current_client.is_some() {
                                    if let Some(client) = app.current_client.clone() {
                                        let tx = tx.clone();
                                        app.state_loading = true;
                                        app.loading_message = Some("Applying filter matrix...".to_string());
                                        
                                        tokio::spawn(async move {
                                            if let Ok((true, ui, si)) = client.authenticate().await {
                                                let _ = tx.send(AsyncAction::PlaylistRefreshed(ui, si)).await;
                                            }
                                        });
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
                SettingsState::AutoRefreshSelection => match key.code {
                    KeyCode::Esc | KeyCode::Backspace => { app.settings_state = SettingsState::Main; }
                    KeyCode::Up | KeyCode::Char('k') => {
                        if let Some(idx) = app.auto_refresh_list_state.selected() {
                            let new_idx = if idx == 0 { 4 } else { idx - 1 };
                            app.auto_refresh_list_state.select(Some(new_idx));
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if let Some(idx) = app.auto_refresh_list_state.selected() {
                            let new_idx = if idx >= 4 { 0 } else { idx + 1 };
                            app.auto_refresh_list_state.select(Some(new_idx));
                        }
                    }
                    KeyCode::Enter => {
                        if let Some(idx) = app.auto_refresh_list_state.selected() {
                            app.config.auto_refresh_hours = match idx {
                                0 => 0,
                                1 => 6,
                                2 => 12,
                                3 => 24,
                                4 => 48,
                                _ => 12,
                            };
                            let _ = app.config.save();
                        }
                        app.settings_state = SettingsState::Main;
                        app.refresh_settings_options();
                    }
                    _ => {}
                }
            }
        }
        CurrentScreen::TimezoneSettings => {
            match key.code {
                KeyCode::Esc | KeyCode::Backspace => {
                    app.current_screen = CurrentScreen::Settings;
                }
                KeyCode::Enter => {
                    // Save selected timezone
                    if let Some(idx) = app.timezone_list_state.selected() {
                        if idx < app.timezone_list.len() {
                            app.config.timezone = Some(app.timezone_list[idx].clone());
                            let _ = app.config.save();
                            app.cached_user_timezone = app.config.get_user_timezone();
                        }
                    }
                    app.current_screen = CurrentScreen::Settings;
                    app.refresh_settings_options();
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    app.previous_timezone();
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    app.next_timezone();
                }
                _ => {}
            }
        }
        CurrentScreen::GroupManagement => {
            match key.code {
                KeyCode::Esc | KeyCode::Backspace => {
                    app.current_screen = app.previous_screen.take().unwrap_or(CurrentScreen::ContentTypeSelection);
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if !app.config.favorites.groups.is_empty() && app.selected_group_index > 0 {
                        app.selected_group_index -= 1;
                        app.group_list_state.select(Some(app.selected_group_index));
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if !app.config.favorites.groups.is_empty() && app.selected_group_index < app.config.favorites.groups.len() - 1 {
                        app.selected_group_index += 1;
                        app.group_list_state.select(Some(app.selected_group_index));
                    }
                }
                KeyCode::Char('n') => {
                    // Create new group with default name
                    let new_name = format!("Group {}", app.config.favorites.groups.len() + 1);
                    app.config.create_group(new_name, Some("📁".to_string()));
                    app.selected_group_index = app.config.favorites.groups.len().saturating_sub(1);
                    app.group_list_state.select(Some(app.selected_group_index));
                }
                KeyCode::Char('d') | KeyCode::Delete => {
                    // Delete selected group
                    if !app.config.favorites.groups.is_empty() {
                        app.config.delete_group(app.selected_group_index);
                        if app.selected_group_index > 0 && app.selected_group_index >= app.config.favorites.groups.len() {
                            app.selected_group_index = app.config.favorites.groups.len().saturating_sub(1);
                        }
                        app.group_list_state.select(if app.config.favorites.groups.is_empty() { None } else { Some(app.selected_group_index) });
                    }
                }
                KeyCode::Enter => {
                    // View group contents (shows as a synthetic category)
                    if !app.config.favorites.groups.is_empty() {
                        // For now, just go back - full group viewing can be a future enhancement
                        app.current_screen = app.previous_screen.take().unwrap_or(CurrentScreen::ContentTypeSelection);
                    }
                }
                _ => {}
            }
        }
        CurrentScreen::GroupPicker => {
            let total_options = app.config.favorites.groups.len() + 1; // +1 for "Create New"
            match key.code {
                KeyCode::Esc => {
                    app.pending_stream_for_group = None;
                    app.current_screen = app.previous_screen.take().unwrap_or(CurrentScreen::Streams);
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if app.selected_group_index > 0 {
                        app.selected_group_index -= 1;
                        app.group_list_state.select(Some(app.selected_group_index));
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if app.selected_group_index < total_options - 1 {
                        app.selected_group_index += 1;
                        app.group_list_state.select(Some(app.selected_group_index));
                    }
                }
                KeyCode::Enter => {
                    if let Some(stream_id) = app.pending_stream_for_group.take() {
                        if app.selected_group_index < app.config.favorites.groups.len() {
                            // Add to existing group
                            app.config.add_to_group(app.selected_group_index, stream_id);
                            app.loading_message = Some(format!("Added to {}", app.config.favorites.groups[app.selected_group_index].name));
                        } else {
                            // Create new group and add
                            let new_name = format!("Group {}", app.config.favorites.groups.len() + 1);
                            let idx = app.config.create_group(new_name.clone(), Some("📁".to_string()));
                            app.config.add_to_group(idx, stream_id);
                            app.loading_message = Some(format!("Created {} and added stream", new_name));
                        }
                    }
                    app.current_screen = app.previous_screen.take().unwrap_or(CurrentScreen::Streams);
                }
                _ => {}
            }
        }
        CurrentScreen::UpdatePrompt => {
            match key.code {
                KeyCode::Enter | KeyCode::Char('u') | KeyCode::Char('U') => {
                    return Ok(InputResult::UpdateRequested);
                }
                KeyCode::Esc | KeyCode::Char('l') | KeyCode::Char('L') | KeyCode::Char('q') => {
                    app.current_screen = CurrentScreen::Home;
                }
                _ => {}
            }
        }
        CurrentScreen::SportsDashboard => {
            match key.code {
                KeyCode::Esc | KeyCode::Backspace => {
                    app.current_screen = app.previous_screen.take().unwrap_or(CurrentScreen::Home);
                }
                KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => {
                    app.active_pane = if app.active_pane == Pane::Categories { Pane::Streams } else { Pane::Categories };
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    app.active_pane = if app.active_pane == Pane::Categories { Pane::Streams } else { Pane::Categories };
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if app.active_pane == Pane::Categories {
                        let i = match app.sports_category_list_state.selected() {
                            Some(i) => {
                                if i == 0 { app.sports_categories.len() - 1 } else { i - 1 }
                            }
                            None => 0,
                        };
                        app.sports_category_list_state.select(Some(i));
                        app.selected_sports_category_index = i;
                        app.sports_matches.clear(); // Trigger refresh
                    } else {
                        let i = match app.sports_list_state.selected() {
                            Some(i) => {
                                if i == 0 { if app.sports_matches.is_empty() { 0 } else { app.sports_matches.len() - 1 } } else { i - 1 }
                            }
                            None => 0,
                        };
                        app.sports_list_state.select(Some(i));
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if app.active_pane == Pane::Categories {
                        let i = match app.sports_category_list_state.selected() {
                            Some(i) => {
                                if i >= app.sports_categories.len() - 1 { 0 } else { i + 1 }
                            }
                            None => 0,
                        };
                        app.sports_category_list_state.select(Some(i));
                        app.selected_sports_category_index = i;
                        app.sports_matches.clear(); // Trigger refresh
                    } else {
                        let i = match app.sports_list_state.selected() {
                            Some(i) => {
                                if app.sports_matches.is_empty() { 0 } else if i >= app.sports_matches.len() - 1 { 0 } else { i + 1 }
                            }
                            None => 0,
                        };
                        app.sports_list_state.select(Some(i));
                    }
                }
                KeyCode::Enter => {
                    if app.active_pane == Pane::Categories {
                        app.active_pane = Pane::Streams;
                    } else if !app.current_sports_streams.is_empty() {
                        let stream = &app.current_sports_streams[0]; // Play first link
                        let url = stream.embed_url.clone();
                        let title = app.sports_matches[app.sports_list_state.selected().unwrap_or(0)].title.clone();
                        
                        app.state_loading = true;
                        app.loading_message = Some(format!("Preparing: {}...", title));
                        
                        let tx = tx.clone();
                        let player = player.clone();
                        let use_default = app.config.use_default_mpv;
                        tokio::spawn(async move {
                            match player.play(&url, use_default) {
                                Ok(_) => {
                                    match player.wait_for_playback(10000).await {
                                        Ok(true) => { let _ = tx.send(AsyncAction::PlayerStarted).await; }
                                        _ => { let _ = tx.send(AsyncAction::PlayerFailed("Failed to start".to_string())).await; }
                                    }
                                }
                                Err(e) => { let _ = tx.send(AsyncAction::PlayerFailed(e.to_string())).await; }
                            }
                        });
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
    Ok(InputResult::Continue)
}
