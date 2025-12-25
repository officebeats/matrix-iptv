use std::{io, time::Duration};

#[cfg(not(target_arch = "wasm32"))]
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind,
        MouseButton, MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

#[cfg(not(target_arch = "wasm32"))]
use ratatui::{backend::CrosstermBackend, Terminal};
#[cfg(not(target_arch = "wasm32"))]
use tokio::sync::mpsc;
#[cfg(not(target_arch = "wasm32"))]
use tui_input::backend::crossterm::EventHandler;

use matrix_iptv_lib::app::{App, AsyncAction, CurrentScreen, Guide, InputMode, LoginField, Pane};

use matrix_iptv_lib::api::{Category, XtreamClient};
use matrix_iptv_lib::config::Account;
use matrix_iptv_lib::{player, setup, ui};

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Optional Direct Play URL (if provided, plays and exits)
    #[arg(short, long)]
    play: Option<String>,

    /// Check configuration and verify login
    #[arg(long)]
    check: bool,
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    use clap::Parser;
    let args = Args::parse();

    // -- CLI MODE --
    if args.check {
        setup::check_and_install_dependencies()?;
        println!("Checking configuration...");
        // Reuse verification logic (simplified)
        // For now just print ok as verifying needs full async client setup which is in TUI logic
        // But we can check if config exists
        let config = matrix_iptv_lib::config::AppConfig::load()?;
        println!("Loaded config for {} accounts.", config.accounts.len());
        return Ok(());
    }

    if let Some(url) = args.play {
        setup::check_and_install_dependencies()?;
        let player = player::Player::new();
        println!("Playing: {}", url);
        player.play(&url)?;
        return Ok(());
    }

    // -- TUI MODE (Default) --

    // Check Dependencies First
    setup::check_and_install_dependencies()?;

    // Setup Terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // App State
    let mut app = App::new();
    let player = player::Player::new();

    // Async Channel
    let (tx, mut rx) = mpsc::channel::<AsyncAction>(32);

    let res = run_app(&mut terminal, &mut app, &player, tx, &mut rx).await;

    // Restore Terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(not(target_arch = "wasm32"))]
async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    player: &player::Player,
    tx: mpsc::Sender<AsyncAction>,
    rx: &mut mpsc::Receiver<AsyncAction>,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui::ui(f, app))?;

        // 1. Check for Async Actions (Non-blocking)
        while let Ok(action) = rx.try_recv() {
            match action {
                AsyncAction::LoginSuccess(client, ui, si) => {
                    app.current_client = Some(client);
                    app.account_info = ui.clone();
                    app.server_info = si.clone();
                    app.provider_timezone = si.and_then(|s| s.timezone);
                    
                    // Reset search state on fresh login/playlist selection
                    app.search_mode = false;
                    app.search_query.clear();

                    // Load persisted counts from config first (last session)
                    if let Some(account) = app.config.accounts.get(app.selected_account_index) {
                        app.total_channels = account.total_channels.unwrap_or(0);
                        app.total_movies = account.total_movies.unwrap_or(0);
                        app.total_series = account.total_series.unwrap_or(0);
                    }

                    // Override with metadata if available and persisted is 0
                    if let Some(info) = &ui {
                        if app.total_channels == 0 {
                            app.total_channels = match &info.total_live_streams {
                                Some(serde_json::Value::Number(n)) => {
                                    n.as_u64().unwrap_or(0) as usize
                                }
                                Some(serde_json::Value::String(s)) => {
                                    s.parse::<usize>().unwrap_or(0)
                                }
                                _ => 0,
                            };
                        }
                        if app.total_movies == 0 {
                            app.total_movies = match &info.total_vod_streams {
                                Some(serde_json::Value::Number(n)) => {
                                    n.as_u64().unwrap_or(0) as usize
                                }
                                Some(serde_json::Value::String(s)) => {
                                    s.parse::<usize>().unwrap_or(0)
                                }
                                _ => 0,
                            };
                        }
                        if app.total_series == 0 {
                            app.total_series = match &info.total_series_streams {
                                Some(serde_json::Value::Number(n)) => {
                                    n.as_u64().unwrap_or(0) as usize
                                }
                                Some(serde_json::Value::String(s)) => {
                                    s.parse::<usize>().unwrap_or(0)
                                }
                                _ => 0,
                            };
                        }
                    }

                    app.state_loading = true; // Now loading categories

                    // Update Last Refreshed Timestamp on successful login
                    let ts_now = chrono::Utc::now().timestamp();
                    if let Some(account) = app.config.accounts.get_mut(app.selected_account_index) {
                        account.last_refreshed = Some(ts_now);
                        let _ = app.config.save();
                    }

                    // Trigger background tasks
                    if let Some(client) = &app.current_client {
                        let client = client.clone();
                        let tx = tx.clone();

                        // 1. Categories (Priority)
                        let c1 = client.clone();
                        let t1 = tx.clone();
                        tokio::spawn(async move {
                            match c1.get_live_categories().await {
                                Ok(cats) => {
                                    let _ = t1.send(AsyncAction::CategoriesLoaded(cats)).await;
                                }
                                Err(e) => {
                                    let _ = t1.send(AsyncAction::Error(e.to_string())).await;
                                }
                            }
                        });

                        // 1.5. VOD Categories
                        let c_vod = client.clone();
                        let t_vod = tx.clone();
                        tokio::spawn(async move {
                            match c_vod.get_vod_categories().await {
                                Ok(cats) => {
                                    let _ = t_vod.send(AsyncAction::VodCategoriesLoaded(cats)).await;
                                }
                                Err(e) => {
                                    let _ = t_vod.send(AsyncAction::Error(format!("VOD Error: {}", e))).await;
                                }
                            }
                        });

                        // 1.6. Series Categories
                        let c_series = client.clone();
                        let t_series = tx.clone();
                        tokio::spawn(async move {
                            match c_series.get_series_categories().await {
                                Ok(cats) => {
                                    let _ = t_series.send(AsyncAction::SeriesCategoriesLoaded(cats)).await;
                                }
                                Err(e) => {
                                    let _ = t_series.send(AsyncAction::Error(format!("Series Error: {}", e))).await;
                                }
                            }
                        });

                        // 2. Background Counts (Live) - Accurate count
                        let c2 = client.clone();
                        let t2 = tx.clone();
                        tokio::spawn(async move {
                            if let Ok(streams) = c2.get_live_streams("ALL").await {
                                let _ = t2
                                    .send(AsyncAction::TotalChannelsLoaded(streams.len()))
                                    .await;
                            }
                        });

                        // 3. Background Counts (VOD) - Accurate count
                        let c3 = client.clone();
                        let t3 = tx.clone();
                        tokio::spawn(async move {
                            if let Ok(streams) = c3.get_vod_streams_all().await {
                                let _ =
                                    t3.send(AsyncAction::TotalMoviesLoaded(streams.len())).await;
                            }
                        });

                        // 4. Background Counts (Series) - Accurate count
                        let c4 = client.clone();
                        let t4 = tx.clone();
                        tokio::spawn(async move {
                            if let Ok(series) = c4.get_series_all().await {
                                let _ = t4.send(AsyncAction::TotalSeriesLoaded(series.len())).await;
                            }
                        });
                    }
                }
                AsyncAction::LoginFailed(e) => {
                    app.login_error = Some(e);
                    app.state_loading = false;
                }
                AsyncAction::CategoriesLoaded(mut cats) => {
                    // Reset search state when fresh categories are loaded
                    app.search_mode = false;
                    app.search_query.clear();
                    // Inject "All Channels" category
                    cats.insert(
                        0,
                        Category {
                            category_id: "ALL".to_string(),
                            category_name: "All Channels".to_string(),
                            parent_id: serde_json::Value::Number(serde_json::Number::from(0)),
                        },
                    );

                    // Sort categories: favorites first, then alphabetically
                    cats.sort_by(|a, b| {
                        let a_fav = app.config.favorites.categories.contains(&a.category_id);
                        let b_fav = app.config.favorites.categories.contains(&b.category_id);
                        
                        // Keep "All Channels" at the top
                        if a.category_id == "ALL" {
                            return std::cmp::Ordering::Less;
                        }
                        if b.category_id == "ALL" {
                            return std::cmp::Ordering::Greater;
                        }
                        
                        match (a_fav, b_fav) {
                            (true, false) => std::cmp::Ordering::Less,  // a is favorite, comes first
                            (false, true) => std::cmp::Ordering::Greater, // b is favorite, comes first
                            _ => a.category_name.cmp(&b.category_name), // Both same, sort alphabetically
                        }
                    });

                    app.all_categories = cats.clone();
                    app.categories = cats;
                    if !app.categories.is_empty() {
                        app.selected_category_index = 0;
                        app.category_list_state.select(Some(0));
                        // If we are coming from Login/Home initially, go to Content Type Selection
                        if app.current_screen != CurrentScreen::Categories && app.current_screen != CurrentScreen::Streams {
                            app.current_screen = CurrentScreen::ContentTypeSelection;
                        }
                    }
                    app.state_loading = false;
                }

                AsyncAction::StreamsLoaded(mut streams) => {
                    // Sort streams: favorites first, then alphabetically
                    streams.sort_by(|a, b| {
                        let a_id = match &a.stream_id {
                            serde_json::Value::Number(n) => n.to_string(),
                            serde_json::Value::String(s) => s.clone(),
                            _ => a.stream_id.to_string(),
                        };
                        let b_id = match &b.stream_id {
                            serde_json::Value::Number(n) => n.to_string(),
                            serde_json::Value::String(s) => s.clone(),
                            _ => b.stream_id.to_string(),
                        };
                        
                        let a_fav = app.config.favorites.streams.contains(&a_id);
                        let b_fav = app.config.favorites.streams.contains(&b_id);
                        
                        match (a_fav, b_fav) {
                            (true, false) => std::cmp::Ordering::Less,
                            (false, true) => std::cmp::Ordering::Greater,
                            _ => a.name.cmp(&b.name),
                        }
                    });
                    
                    app.all_streams = streams.clone();
                    app.streams = streams;
                    app.current_screen = CurrentScreen::Streams;
                    app.state_loading = false;
                    app.selected_stream_index = 0;
                    app.stream_list_state.select(Some(0));
                }

                AsyncAction::VodCategoriesLoaded(mut cats) => {
                    // Reset search state when fresh VOD categories are loaded
                    app.search_mode = false;
                    app.search_query.clear();
                    // Inject "All Movies" category
                    cats.insert(
                        0,
                        Category {
                            category_id: "ALL".to_string(),
                            category_name: "All Movies".to_string(),
                            parent_id: serde_json::Value::Number(serde_json::Number::from(0)),
                        },
                    );

                    // Sort categories: favorites first, then alphabetically
                    cats.sort_by(|a, b| {
                        let a_fav = app.config.favorites.vod_categories.contains(&a.category_id);
                        let b_fav = app.config.favorites.vod_categories.contains(&b.category_id);
                        
                        // Keep "All Movies" at the top
                        if a.category_id == "ALL" {
                            return std::cmp::Ordering::Less;
                        }
                        if b.category_id == "ALL" {
                            return std::cmp::Ordering::Greater;
                        }
                        
                        match (a_fav, b_fav) {
                            (true, false) => std::cmp::Ordering::Less,
                            (false, true) => std::cmp::Ordering::Greater,
                            _ => a.category_name.cmp(&b.category_name),
                        }
                    });

                    app.all_vod_categories = cats.clone();
                    app.vod_categories = cats;
                    if !app.vod_categories.is_empty() {
                        app.selected_vod_category_index = 0;
                        app.vod_category_list_state.select(Some(0));
                        // If we are coming from Login/Home initially, go to Content Type Selection
                        if app.current_screen != CurrentScreen::VodCategories && app.current_screen != CurrentScreen::VodStreams && app.current_screen != CurrentScreen::ContentTypeSelection {
                             app.current_screen = CurrentScreen::ContentTypeSelection;
                        }
                    }
                    app.state_loading = false;
                }
                AsyncAction::VodStreamsLoaded(mut streams) => {
                    // Sort streams: favorites first, then alphabetically
                    streams.sort_by(|a, b| {
                        let a_id = match &a.stream_id {
                            serde_json::Value::Number(n) => n.to_string(),
                            serde_json::Value::String(s) => s.clone(),
                            _ => a.stream_id.to_string(),
                        };
                        let b_id = match &b.stream_id {
                            serde_json::Value::Number(n) => n.to_string(),
                            serde_json::Value::String(s) => s.clone(),
                            _ => b.stream_id.to_string(),
                        };
                        
                        let a_fav = app.config.favorites.vod_streams.contains(&a_id);
                        let b_fav = app.config.favorites.vod_streams.contains(&b_id);
                        
                        match (a_fav, b_fav) {
                            (true, false) => std::cmp::Ordering::Less,
                            (false, true) => std::cmp::Ordering::Greater,
                            _ => a.name.cmp(&b.name),
                        }
                    });
                    
                    app.all_vod_streams = streams.clone();
                    app.vod_streams = streams;
                    app.current_screen = CurrentScreen::VodStreams;
                    app.state_loading = false;
                    app.selected_vod_stream_index = 0;
                    app.vod_stream_list_state.select(Some(0));
                }

                AsyncAction::SeriesCategoriesLoaded(mut cats) => {
                    // Reset search state when fresh Series categories are loaded
                    app.search_mode = false;
                    app.search_query.clear();
                    // Sort categories: favorites first, then alphabetically
                    cats.sort_by(|a, b| {
                        let a_fav = app.config.favorites.vod_categories.contains(&a.category_id);
                        let b_fav = app.config.favorites.vod_categories.contains(&b.category_id);
                        
                        match (a_fav, b_fav) {
                            (true, false) => std::cmp::Ordering::Less,
                            (false, true) => std::cmp::Ordering::Greater,
                            _ => a.category_name.cmp(&b.category_name),
                        }
                    });
                    
                    app.all_series_categories = cats.clone();
                    app.series_categories = cats;
                    if !app.series_categories.is_empty() {
                        app.selected_series_category_index = 0;
                        app.series_category_list_state.select(Some(0));
                        // If we are coming from Login/Home initially, go to Content Type Selection
                        if app.current_screen != CurrentScreen::SeriesCategories && app.current_screen != CurrentScreen::SeriesStreams && app.current_screen != CurrentScreen::ContentTypeSelection {
                             app.current_screen = CurrentScreen::ContentTypeSelection;
                        }
                    }
                    app.state_loading = false;
                }
                AsyncAction::SeriesStreamsLoaded(mut streams) => {
                    // Sort streams: favorites first, then alphabetically
                    streams.sort_by(|a, b| {
                        let a_id = match &a.stream_id {
                            serde_json::Value::Number(n) => n.to_string(),
                            serde_json::Value::String(s) => s.clone(),
                            _ => a.stream_id.to_string(),
                        };
                        let b_id = match &b.stream_id {
                            serde_json::Value::Number(n) => n.to_string(),
                            serde_json::Value::String(s) => s.clone(),
                            _ => b.stream_id.to_string(),
                        };
                        
                        let a_fav = app.config.favorites.vod_streams.contains(&a_id);
                        let b_fav = app.config.favorites.vod_streams.contains(&b_id);
                        
                        match (a_fav, b_fav) {
                            (true, false) => std::cmp::Ordering::Less,
                            (false, true) => std::cmp::Ordering::Greater,
                            _ => a.name.cmp(&b.name),
                        }
                    });
                    
                    app.all_series_streams = streams.clone();
                    app.series_streams = streams;
                    app.current_screen = CurrentScreen::SeriesStreams;
                    app.state_loading = false;
                    app.selected_series_stream_index = 0;
                    app.series_stream_list_state.select(Some(0));
                }
                AsyncAction::PlayerStarted => {
                    app.state_loading = false;
                    app.loading_message = None;
                }
                AsyncAction::PlayerFailed(e) => {
                    app.state_loading = false;
                    app.loading_message = None;
                    app.player_error = Some(e);
                }
                AsyncAction::LoadingMessage(msg) => {
                    app.loading_message = Some(msg);
                }
                AsyncAction::TotalChannelsLoaded(count) => {
                    app.total_channels = count;
                    if let Some(account) = app.config.accounts.get_mut(app.selected_account_index) {
                        account.total_channels = Some(count);
                        let _ = app.config.save();
                    }
                }
                AsyncAction::TotalMoviesLoaded(count) => {
                    app.total_movies = count;
                    if let Some(account) = app.config.accounts.get_mut(app.selected_account_index) {
                        account.total_movies = Some(count);
                        let _ = app.config.save();
                    }
                }
                AsyncAction::TotalSeriesLoaded(count) => {
                    app.total_series = count;
                    if let Some(account) = app.config.accounts.get_mut(app.selected_account_index) {
                        account.total_series = Some(count);
                        let _ = app.config.save();
                    }
                }
                AsyncAction::PlaylistRefreshed(ui, si) => {
                    app.account_info = ui;
                    app.server_info = si;
                    app.state_loading = false;
                    app.loading_message = None;

                    // Update last_refreshed timestamp
                    if let Some(account) = app.config.accounts.get_mut(app.selected_account_index) {
                        account.last_refreshed = Some(chrono::Utc::now().timestamp());
                        let _ = app.config.save();
                    }
                }
                AsyncAction::SeriesInfoLoaded(info) => {
                    app.current_series_info = Some(info.clone());
                    app.state_loading = false;
                    // Extract episodes from the nested JSON structure
                    let mut episodes = Vec::new();
                    if let serde_json::Value::Object(episodes_map) = &info.episodes {
                        for (_season_key, season_episodes) in episodes_map {
                            if let serde_json::Value::Array(ep_array) = season_episodes {
                                for ep_val in ep_array {
                                    if let Ok(episode) = serde_json::from_value::<matrix_iptv_lib::api::SeriesEpisode>(ep_val.clone()) {
                                        episodes.push(episode);
                                    }
                                }
                            }
                        }
                    }
                    
                    // Sort episodes by season and episode number
                    episodes.sort_by(|a, b| {
                        match a.season.cmp(&b.season) {
                            std::cmp::Ordering::Equal => a.episode_num.cmp(&b.episode_num),
                            other => other,
                        }
                    });
                    
                    app.series_episodes = episodes;
                    app.selected_series_episode_index = 0;
                    if !app.series_episodes.is_empty() {
                        app.series_episode_list_state.select(Some(0));
                    }
                    app.state_loading = false;
                }
                AsyncAction::Error(e) => {
                    // Generic error handling - maybe show a popup? For now, print to stderr essentially (or login error)
                    // If we are at login, show it there
                    if app.current_screen == CurrentScreen::Login {
                        app.login_error = Some(e);
                    }
                    app.state_loading = false;
                }
            }
        }

        app.loading_tick = app.loading_tick.wrapping_add(1);
        
        // FTUE: Handle Matrix rain animation
        if app.show_matrix_rain {
            if let Ok(size) = terminal.size() {
                let rect = ratatui::layout::Rect::new(0, 0, size.width, size.height);
                // Initialize columns if not already done
                if app.matrix_rain_columns.is_empty() {
                    app.matrix_rain_columns = matrix_iptv_lib::matrix_rain::init_matrix_rain(rect);
                }
                
                // Update animation
                matrix_iptv_lib::matrix_rain::update_matrix_rain(&mut app.matrix_rain_columns, rect, app.loading_tick);
                
                // Only end startup animation after 3 seconds (screensaver runs indefinitely)
                if !app.matrix_rain_screensaver_mode {
                    if let Some(start_time) = app.matrix_rain_start_time {
                        if start_time.elapsed().as_secs() >= 3 {
                            app.show_matrix_rain = false;
                            // Only show welcome popup if user has no playlists configured
                            if app.config.accounts.is_empty() {
                                app.show_welcome_popup = true;
                            }
                            app.matrix_rain_start_time = None;
                        }
                    }
                }
            }
        }

        // 2. Poll inputs
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    // Only process key press events, not release (Windows sends both)
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }

                    // Priority 1: Help Popup
                    if app.show_help {
                        if matches!(
                            key.code,
                            KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q')
                        ) {
                            app.show_help = false;
                        }
                        continue;
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
                        continue;
                    }
                    
                    // Priority 3: Matrix Rain Screensaver
                    if app.show_matrix_rain && app.matrix_rain_screensaver_mode {
                        // Any key exits screensaver mode
                        app.show_matrix_rain = false;
                        app.matrix_rain_screensaver_mode = false;
                        app.matrix_rain_start_time = None;
                        continue;
                    }
                    
                    // Priority 4: Welcome Popup (FTUE)
                    if app.show_welcome_popup {
                        // Any key dismisses the welcome popup
                        app.show_welcome_popup = false;
                        continue;
                    }

                    // GLOBAL KEYS
                    if app.input_mode == InputMode::Normal {
                        if let KeyCode::Char('q') = key.code {
                            app.should_quit = true;
                        }
                        if key.code == KeyCode::Char('?') {
                            app.show_help = !app.show_help;
                            continue;
                        }

                        // Refresh Playlist
                        if key.code == KeyCode::Char('r') {
                            if let Some(client) = app.current_client.clone() {
                                let tx = tx.clone();
                                app.state_loading = true;
                                app.loading_message = Some("Refreshing playlist...".to_string());

                                // Clear lists to ensure UI reflects new state next time they are viewed
                                app.categories.clear();
                                app.all_categories.clear();
                                app.streams.clear();
                                app.all_streams.clear();
                                app.vod_categories.clear();
                                app.all_vod_categories.clear();
                                app.vod_streams.clear();
                                app.all_vod_streams.clear();

                                tokio::spawn(async move {
                                    // 1. Re-authenticate to get fresh info
                                    let (auth_success, ui, si) = match client.authenticate().await {
                                        Ok(r) => r,
                                        Err(e) => {
                                            let _ = tx
                                                .send(AsyncAction::Error(format!(
                                                    "Refresh failed: {}",
                                                    e
                                                )))
                                                .await;
                                            let _ = tx
                                                .send(AsyncAction::PlaylistRefreshed(None, None))
                                                .await;
                                            return;
                                        }
                                    };

                                    if !auth_success {
                                        let _ = tx
                                            .send(AsyncAction::Error(
                                                "Refresh authentication failed".to_string(),
                                            ))
                                            .await;
                                        let _ = tx
                                            .send(AsyncAction::PlaylistRefreshed(None, None))
                                            .await;
                                        return;
                                    }

                                    // 2. Refresh counts from UserInfo metadata
                                    if let Some(info) = &ui {
                                        let c_count = match &info.total_live_streams {
                                            Some(serde_json::Value::Number(n)) => {
                                                n.as_u64().unwrap_or(0) as usize
                                            }
                                            Some(serde_json::Value::String(s)) => {
                                                s.parse::<usize>().unwrap_or(0)
                                            }
                                            _ => 0,
                                        };
                                        let v_count = match &info.total_vod_streams {
                                            Some(serde_json::Value::Number(n)) => {
                                                n.as_u64().unwrap_or(0) as usize
                                            }
                                            Some(serde_json::Value::String(s)) => {
                                                s.parse::<usize>().unwrap_or(0)
                                            }
                                            _ => 0,
                                        };
                                        let s_count = match &info.total_series_streams {
                                            Some(serde_json::Value::Number(n)) => {
                                                n.as_u64().unwrap_or(0) as usize
                                            }
                                            Some(serde_json::Value::String(s)) => {
                                                s.parse::<usize>().unwrap_or(0)
                                            }
                                            _ => 0,
                                        };

                                        let _ = tx
                                            .send(AsyncAction::TotalChannelsLoaded(c_count))
                                            .await;
                                        let _ =
                                            tx.send(AsyncAction::TotalMoviesLoaded(v_count)).await;
                                        let _ =
                                            tx.send(AsyncAction::TotalSeriesLoaded(s_count)).await;
                                    }

                                    // 3. Finish
                                    let _ = tx.send(AsyncAction::PlaylistRefreshed(ui, si)).await;
                                });
                            }
                        }
                    }

                    if app.should_quit {
                        return Ok(());
                    }

                    // SCREEN SPECIFIC
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
                                }
                                KeyCode::Char('e') => {
                                    if !app.config.accounts.is_empty() {
                                        app.editing_account_index =
                                            Some(app.selected_account_index);
                                        let acc = &app.config.accounts[app.selected_account_index];
                                        app.input_name = tui_input::Input::new(acc.name.clone());
                                        app.input_url = tui_input::Input::new(acc.base_url.clone());
                                        app.input_username =
                                            tui_input::Input::new(acc.username.clone());
                                        app.input_password =
                                            tui_input::Input::new(acc.password.clone());
                                        app.input_epg_url = tui_input::Input::new(
                                            acc.epg_url.clone().unwrap_or_default(),
                                        );

                                        app.current_screen = CurrentScreen::Login;
                                        app.previous_screen = Some(CurrentScreen::Home);
                                    }
                                }
                                KeyCode::Char('d') => {
                                    if !app.config.accounts.is_empty() {
                                        app.config.remove_account(app.selected_account_index);
                                        if app.selected_account_index >= app.config.accounts.len()
                                            && !app.config.accounts.is_empty()
                                        {
                                            app.selected_account_index =
                                                app.config.accounts.len() - 1;
                                            app.account_list_state
                                                .select(Some(app.selected_account_index));
                                        } else if app.config.accounts.is_empty() {
                                            app.selected_account_index = 0;
                                            app.account_list_state.select(None);
                                        }
                                    }
                                }
                                KeyCode::Char('x') => app.current_screen = CurrentScreen::Settings,
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
                                KeyCode::Char('s') => {
                                    // Enter Series Mode
                                    if !app.config.accounts.is_empty() {
                                        let acc = &app.config.accounts[app.selected_account_index];
                                        let base_url = acc.base_url.clone();
                                        let username = acc.username.clone();
                                        let password = acc.password.clone();
                                        // 5-hour staleness check
                                        let now = chrono::Utc::now().timestamp();
                                        let needs_refresh = acc
                                            .last_refreshed
                                            .map(|last| now - last > (5 * 3600))
                                            .unwrap_or(true);

                                        app.state_loading = true;
                                        if needs_refresh {
                                            app.loading_message = Some(
                                                "Refreshing playlist (Data > 5h old)..."
                                                    .to_string(),
                                            );
                                        } else {
                                            app.loading_message =
                                                Some("Loading Series...".to_string());
                                        }
                                        app.login_error = None;

                                        let tx = tx.clone();
                                        tokio::spawn(async move {
                                            // 1. Authenticate first (crucial for valid token/session if needed, though usually just creds)
                                            match XtreamClient::new_with_doh(
                                                base_url, username, password,
                                            )
                                            .await
                                            {
                                                Ok(client) => {
                                                    match client.authenticate().await {
                                                        Ok((true, ui, si)) => {
                                                            let _ = tx
                                                                .send(AsyncAction::LoginSuccess(
                                                                    client.clone(),
                                                                    ui,
                                                                    si,
                                                                ))
                                                                .await;
                                                            // 2. Fetch Series Categories
                                                            match client
                                                                .get_series_categories()
                                                                .await
                                                            {
                                                                Ok(cats) => {
                                                                    let _ = tx.send(AsyncAction::SeriesCategoriesLoaded(cats)).await;
                                                                }
                                                                Err(e) => {
                                                                    let _ = tx.send(AsyncAction::Error(format!("Series Fetch Error: {}", e))).await;
                                                                }
                                                            }
                                                        }
                                                        Ok((false, _, _)) => {
                                                            let _ = tx
                                                                .send(AsyncAction::LoginFailed(
                                                                    "Authentication failed"
                                                                        .to_string(),
                                                                ))
                                                                .await;
                                                        }
                                                        Err(e) => {
                                                            let _ = tx
                                                                .send(AsyncAction::LoginFailed(
                                                                    e.to_string(),
                                                                ))
                                                                .await;
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    let _ = tx
                                                        .send(AsyncAction::LoginFailed(format!(
                                                            "Connection error: {}",
                                                            e
                                                        )))
                                                        .await;
                                                }
                                            }
                                        });
                                    }
                                }
                                KeyCode::Enter => {
                                    if !app.config.accounts.is_empty() {
                                        let acc = &app.config.accounts[app.selected_account_index];
                                        let base_url = acc.base_url.clone();
                                        let username = acc.username.clone();
                                        let password = acc.password.clone();

                                        // 5-hour staleness check
                                        let now = chrono::Utc::now().timestamp();
                                        let needs_refresh = acc
                                            .last_refreshed
                                            .map(|last| now - last > (5 * 3600))
                                            .unwrap_or(true);

                                        app.state_loading = true;
                                        if needs_refresh {
                                            app.loading_message = Some(
                                                "Refreshing playlist (Data > 5h old)..."
                                                    .to_string(),
                                            );
                                        } else {
                                            app.loading_message =
                                                Some("Loading playlist...".to_string());
                                        }

                                        app.login_error = None;
                                        let tx = tx.clone();
                                        tokio::spawn(async move {
                                            match XtreamClient::new_with_doh(
                                                base_url, username, password,
                                            )
                                            .await
                                            {
                                                Ok(client) => match client.authenticate().await {
                                                    Ok((true, ui, si)) => {
                                                        let _ = tx
                                                            .send(AsyncAction::LoginSuccess(
                                                                client, ui, si,
                                                            ))
                                                            .await;
                                                    }
                                                    Ok((false, _, _)) => {
                                                        let _ = tx
                                                            .send(AsyncAction::LoginFailed(
                                                                "Authentication failed".to_string(),
                                                            ))
                                                            .await;
                                                    }
                                                    Err(e) => {
                                                        let _ = tx
                                                            .send(AsyncAction::LoginFailed(
                                                                e.to_string(),
                                                            ))
                                                            .await;
                                                    }
                                                },
                                                Err(e) => {
                                                    let _ = tx
                                                        .send(AsyncAction::LoginFailed(format!(
                                                            "Connection error: {}",
                                                            e
                                                        )))
                                                        .await;
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
                                    // Reset search state
                                    app.search_mode = false;
                                    app.search_query.clear();
                                    app.update_search();
                                }
                                KeyCode::Char('2') => {
                                    app.current_screen = CurrentScreen::VodCategories;
                                    app.active_pane = Pane::Categories;
                                    // Reset search state
                                    app.search_mode = false;
                                    app.search_query.clear();
                                    app.update_search();
                                },
                                KeyCode::Char('3') => {
                                    app.current_screen = CurrentScreen::SeriesCategories;
                                    app.active_pane = Pane::Categories;
                                    // Reset search state
                                    app.search_mode = false;
                                    app.search_query.clear();
                                    app.update_search();
                                },
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
                                KeyCode::Enter => {
                                    match app.selected_content_type_index {
                                        0 => {
                                            app.current_screen = CurrentScreen::Categories;
                                            app.active_pane = Pane::Categories;
                                            // Reset search state
                                            app.search_mode = false;
                                            app.search_query.clear();
                                            app.update_search();
                                        }
                                        1 => {
                                            app.current_screen = CurrentScreen::VodCategories;
                                            app.active_pane = Pane::Categories;
                                            // Reset search state
                                            app.search_mode = false;
                                            app.search_query.clear();
                                            app.update_search();
                                        }
                                        2 => {
                                            app.current_screen = CurrentScreen::SeriesCategories;
                                            app.active_pane = Pane::Categories;
                                            // Reset search state
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
                                _ => {}
                            }
                        }
                        CurrentScreen::Login => {
                            if app.show_save_confirmation {
                                match key.code {
                                    KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                                        // Save and go back
                                        app.save_account();
                                        app.show_save_confirmation = false;
                                        app.current_screen = app
                                            .previous_screen
                                            .take()
                                            .unwrap_or(CurrentScreen::Home);
                                    }
                                    KeyCode::Char('n') | KeyCode::Char('N') => {
                                        // Discard and go back
                                        app.show_save_confirmation = false;
                                        app.current_screen = app
                                            .previous_screen
                                            .take()
                                            .unwrap_or(CurrentScreen::Home);

                                        // Reset inputs
                                        app.input_name = tui_input::Input::default();
                                        app.input_url = tui_input::Input::default();
                                        app.input_username = tui_input::Input::default();
                                        app.input_password = tui_input::Input::default();
                                        app.input_epg_url = tui_input::Input::default();
                                        app.editing_account_index = None;
                                    }
                                    KeyCode::Esc => {
                                        // Cancel exit
                                        app.show_save_confirmation = false;
                                    }
                                    _ => {}
                                }
                            } else {
                                match app.input_mode {
                                    InputMode::Normal => {
                                        match key.code {
                                            KeyCode::Esc => {
                                                // Check for changes
                                                let mut changed = false;

                                                // Determine original values
                                                let (
                                                    orig_name,
                                                    orig_url,
                                                    orig_user,
                                                    orig_pass,
                                                    orig_epg,
                                                ) = if let Some(idx) = app.editing_account_index {
                                                    if let Some(acc) = app.config.accounts.get(idx)
                                                    {
                                                        (
                                                            acc.name.clone(),
                                                            acc.base_url.clone(),
                                                            acc.username.clone(),
                                                            acc.password.clone(),
                                                            acc.epg_url.clone().unwrap_or_default(),
                                                        )
                                                    } else {
                                                        (
                                                            "".to_string(),
                                                            "".to_string(),
                                                            "".to_string(),
                                                            "".to_string(),
                                                            "".to_string(),
                                                        )
                                                    }
                                                } else {
                                                    (
                                                        "".to_string(),
                                                        "".to_string(),
                                                        "".to_string(),
                                                        "".to_string(),
                                                        "".to_string(),
                                                    )
                                                };

                                                if app.input_name.value() != orig_name
                                                    || app.input_url.value() != orig_url
                                                    || app.input_username.value() != orig_user
                                                    || app.input_password.value() != orig_pass
                                                    || app.input_epg_url.value() != orig_epg
                                                {
                                                    changed = true;
                                                }

                                                // Special case: If new account and everything is empty, it's not "changed", it's just nothing
                                                if app.editing_account_index.is_none()
                                                    && app.input_name.value().is_empty()
                                                    && app.input_url.value().is_empty()
                                                    && app.input_username.value().is_empty()
                                                    && app.input_password.value().is_empty()
                                                    && app.input_epg_url.value().is_empty()
                                                {
                                                    changed = false;
                                                }

                                                if changed {
                                                    app.show_save_confirmation = true;
                                                } else {
                                                    app.current_screen = app
                                                        .previous_screen
                                                        .take()
                                                        .unwrap_or(CurrentScreen::Home);
                                                    app.input_name = tui_input::Input::default();
                                                    app.input_url = tui_input::Input::default();
                                                    app.input_username =
                                                        tui_input::Input::default();
                                                    app.input_password =
                                                        tui_input::Input::default();
                                                    app.input_epg_url = tui_input::Input::default();
                                                    app.editing_account_index = None;
                                                    app.login_error = None;
                                                }
                                            }
                                            KeyCode::Char('j') | KeyCode::Down | KeyCode::Tab => {
                                                app.login_field_focus = match app.login_field_focus
                                                {
                                                    LoginField::Name => LoginField::Url,
                                                    LoginField::Url => LoginField::Username,
                                                    LoginField::Username => LoginField::Password,
                                                    LoginField::Password => LoginField::EpgUrl,
                                                    LoginField::EpgUrl => LoginField::Name,
                                                };
                                            }
                                            KeyCode::Char('k') | KeyCode::Up => {
                                                app.login_field_focus = match app.login_field_focus
                                                {
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
                                            KeyCode::Esc => app.toggle_input_mode(),
                                            KeyCode::Enter => {
                                                // Save and move to next
                                                app.toggle_input_mode();

                                                app.login_field_focus = match app.login_field_focus
                                                {
                                                    LoginField::Name => LoginField::Url,
                                                    LoginField::Url => LoginField::Username,
                                                    LoginField::Username => LoginField::Password,
                                                    LoginField::Password => LoginField::EpgUrl,
                                                    LoginField::EpgUrl => {
                                                        // Save account when pressing Enter on last field
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
                                                            // Reset inputs
                                                            app.input_name = tui_input::Input::default();
                                                            app.input_url = tui_input::Input::default();
                                                            app.input_username = tui_input::Input::default();
                                                            app.input_password = tui_input::Input::default();
                                                            app.input_epg_url = tui_input::Input::default();
                                                            app.login_error = None;
                                                            app.editing_account_index = None;

                                                            LoginField::Name
                                                        } else {
                                                            app.login_error = Some("Name and URL required".to_string());
                                                            LoginField::EpgUrl
                                                        }
                                                    }
                                                };
                                            }
                                            _ => {
                                                // Only support ascii keyboard in terminal basically
                                                match app.login_field_focus {
                                                    LoginField::Name => {
                                                        app.input_name
                                                            .handle_event(&Event::Key(key));
                                                    }
                                                    LoginField::Url => {
                                                        app.input_url
                                                            .handle_event(&Event::Key(key));
                                                    }
                                                    LoginField::Username => {
                                                        app.input_username
                                                            .handle_event(&Event::Key(key));
                                                    }
                                                    LoginField::Password => {
                                                        app.input_password
                                                            .handle_event(&Event::Key(key));
                                                    }
                                                    LoginField::EpgUrl => {
                                                        app.input_epg_url
                                                            .handle_event(&Event::Key(key));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        CurrentScreen::Categories | CurrentScreen::Streams => {
                            use matrix_iptv_lib::app::Pane;

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
                                    KeyCode::Char('/') => {
                                        app.search_mode = true;
                                        app.search_query.clear();
                                        app.update_search();
                                    }
                                    KeyCode::Esc | KeyCode::Backspace => {
                                        if app.active_pane == Pane::Streams
                                            && !app.streams.is_empty()
                                        {
                                            // Go back to categories pane and clear streams
                                            app.active_pane = Pane::Categories;
                                            app.streams.clear();
                                            app.all_streams.clear();
                                            app.selected_stream_index = 0;
                                            app.stream_list_state.select(None);
                                            // Reset search state
                                            app.search_mode = false;
                                            app.search_query.clear();
                                        } else {
                                            // Going back to Home, clear all state
                                            app.streams.clear();
                                            app.all_streams.clear();
                                            app.selected_stream_index = 0;
                                            app.selected_category_index = 0;
                                            app.stream_list_state.select(None);
                                            app.category_list_state.select(None);
                                            // Reset search state
                                            app.search_mode = false;
                                            app.search_query.clear();
                                            app.current_screen = CurrentScreen::ContentTypeSelection;
                                        }
                                    }
                                    KeyCode::Tab => {
                                        // Toggle between panes only
                                        match app.active_pane {
                                            Pane::Categories => {
                                                if !app.streams.is_empty() {
                                                    app.active_pane = Pane::Streams;
                                                }
                                            }
                                            Pane::Streams => {
                                                app.active_pane = Pane::Categories;
                                            }
                                            _ => {} // Episodes not applicable to Live TV
                                        }
                                    }
                                    KeyCode::Char('v') | KeyCode::Char('m') => {
                                        // Switch to VOD Mode
                                        if let Some(client) = &app.current_client {
                                            let client = client.clone();
                                            let tx = tx.clone();
                                            app.state_loading = true;
                                            app.search_mode = false;
                                            app.search_query.clear();
                                            tokio::spawn(async move {
                                                match client.get_vod_categories().await {
                                                    Ok(cats) => {
                                                        let _ = tx
                                                            .send(AsyncAction::VodCategoriesLoaded(
                                                                cats,
                                                            ))
                                                            .await;
                                                    }
                                                    Err(e) => {
                                                        let _ = tx
                                                            .send(AsyncAction::Error(e.to_string()))
                                                            .await;
                                                    }
                                                }
                                            });
                                        }
                                    }
                                    KeyCode::Left | KeyCode::Char('c') => {
                                        app.active_pane = Pane::Categories;
                                    }
                                    KeyCode::Right | KeyCode::Char('s') => {
                                        if !app.streams.is_empty() {
                                            app.active_pane = Pane::Streams;
                                        }
                                    }
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
                                    KeyCode::Char('f') => match app.active_pane {
                                        Pane::Categories => {
                                            if !app.categories.is_empty() {
                                                let id = app.categories
                                                    [app.selected_category_index]
                                                    .category_id
                                                    .clone();
                                                app.config.toggle_favorite_category(id);
                                                
                                                // Re-sort categories after favoriting
                                                app.categories.sort_by(|a, b| {
                                                    let a_fav = app.config.favorites.categories.contains(&a.category_id);
                                                    let b_fav = app.config.favorites.categories.contains(&b.category_id);
                                                    
                                                    if a.category_id == "ALL" {
                                                        return std::cmp::Ordering::Less;
                                                    }
                                                    if b.category_id == "ALL" {
                                                        return std::cmp::Ordering::Greater;
                                                    }
                                                    
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
                                                let stream =
                                                    &app.streams[app.selected_stream_index];
                                                let id = match &stream.stream_id {
                                                    serde_json::Value::Number(n) => n.to_string(),
                                                    serde_json::Value::String(s) => s.clone(),
                                                    _ => stream.stream_id.to_string(),
                                                };
                                                app.config.toggle_favorite_stream(id);
                                                
                                                // Re-sort streams after favoriting
                                                app.streams.sort_by(|a, b| {
                                                    let a_id = match &a.stream_id {
                                                        serde_json::Value::Number(n) => n.to_string(),
                                                        serde_json::Value::String(s) => s.clone(),
                                                        _ => a.stream_id.to_string(),
                                                    };
                                                    let b_id = match &b.stream_id {
                                                        serde_json::Value::Number(n) => n.to_string(),
                                                        serde_json::Value::String(s) => s.clone(),
                                                        _ => b.stream_id.to_string(),
                                                    };
                                                    
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
                                                // Load streams for selected category
                                                if !app.categories.is_empty() {
                                                    let cat_id = app.categories
                                                        [app.selected_category_index]
                                                        .category_id
                                                        .clone();
                                                    // Auto-focus streams pane for better UX
                                                    app.active_pane = Pane::Streams;

                                                    if let Some(client) = &app.current_client {
                                                        let client = client.clone();
                                                        let tx = tx.clone();
                                                        app.state_loading = true;
                                                        tokio::spawn(async move {
                                                            match client
                                                                .get_live_streams(&cat_id)
                                                                .await
                                                            {
                                                                Ok(streams) => {
                                                                    let _ = tx.send(AsyncAction::StreamsLoaded(streams)).await;
                                                                }
                                                                Err(e) => {
                                                                    let _ = tx
                                                                        .send(AsyncAction::Error(
                                                                            e.to_string(),
                                                                        ))
                                                                        .await;
                                                                }
                                                            }
                                                        });
                                                    }
                                                }
                                            }
                                            Pane::Streams => {
                                                // Play selected stream
                                                if !app.streams.is_empty() {
                                                    let stream =
                                                        &app.streams[app.selected_stream_index];
                                                    if let Some(client) = &app.current_client {
                                                        // Handle stream_id safely
                                                        let id = match &stream.stream_id {
                                                            serde_json::Value::Number(n) => {
                                                                n.to_string()
                                                            }
                                                            serde_json::Value::String(s) => {
                                                                s.clone()
                                                            }
                                                            _ => stream.stream_id.to_string(),
                                                        };

                                                        let url = client.get_stream_url(&id, "ts");

                                                        // Start player with loading state
                                                        app.state_loading = true;
                                                        app.player_error = None;
                                                        app.loading_message = Some(format!(
                                                            "Preparing Live Stream: {}...",
                                                            stream.name
                                                        ));
                                                        let tx = tx.clone();
                                                        let player = player.clone();
                                                        let stream_url = url.clone();

                                                        tokio::spawn(async move {
                                                            let _ = tx.send(AsyncAction::LoadingMessage("Connecting to stream server...".to_string())).await;
                                                            match player.play(&stream_url) {
                                                                Ok(_) => {
                                                                    let _ = tx.send(AsyncAction::LoadingMessage("Buffering video...".to_string())).await;

                                                                    // Wait for MPV to confirm playback (polls process status)
                                                                    match player
                                                                        .wait_for_playback(10000)
                                                                        .await
                                                                    {
                                                                        Ok(true) => {
                                                                            let _ = tx.send(AsyncAction::PlayerStarted).await;
                                                                        }
                                                                        Ok(false) => {
                                                                            let _ = tx.send(AsyncAction::PlayerFailed("Stream failed to start - MPV exited unexpectedly".to_string())).await;
                                                                        }
                                                                        Err(e) => {
                                                                            let _ = tx.send(AsyncAction::PlayerFailed(format!("Playback error: {}", e))).await;
                                                                        }
                                                                    }
                                                                }
                                                                Err(e) => {
                                                                    let _ = tx.send(AsyncAction::PlayerFailed(e.to_string())).await;
                                                                }
                                                            }
                                                        });
                                                    }
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                    KeyCode::Char('x') => {
                                        app.current_screen = CurrentScreen::Settings
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
                                    KeyCode::Char('/') => {
                                        app.search_mode = true;
                                        app.active_pane = Pane::Categories; // Ensure update_search knows we are in categories
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
                                        // Reset search state
                                        app.search_mode = false;
                                        app.search_query.clear();
                                        app.current_screen = CurrentScreen::ContentTypeSelection;
                                    }
                                    KeyCode::Tab => {
                                        // No split view for now
                                    }
                                    KeyCode::Char('l') => {
                                        // Switch to Live Categories
                                        if let Some(client) = &app.current_client {
                                            let client = client.clone();
                                            let tx = tx.clone();
                                            app.state_loading = true;
                                            app.search_mode = false;
                                            app.search_query.clear();
                                            tokio::spawn(async move {
                                                match client.get_live_categories().await {
                                                    Ok(cats) => {
                                                        let _ = tx
                                                            .send(AsyncAction::CategoriesLoaded(
                                                                cats,
                                                            ))
                                                            .await;
                                                    }
                                                    Err(e) => {
                                                        let _ = tx
                                                            .send(AsyncAction::Error(e.to_string()))
                                                            .await;
                                                    }
                                                }
                                            });
                                        }
                                    }
                                    KeyCode::Char('m') => {
                                        // Switch to Live Categories
                                        if let Some(client) = &app.current_client {
                                            let client = client.clone();
                                            let tx = tx.clone();
                                            app.state_loading = true;
                                            tokio::spawn(async move {
                                                match client.get_live_categories().await {
                                                    Ok(cats) => {
                                                        let _ = tx
                                                            .send(AsyncAction::CategoriesLoaded(
                                                                cats,
                                                            ))
                                                            .await;
                                                    }
                                                    Err(e) => {
                                                        let _ = tx
                                                            .send(AsyncAction::Error(e.to_string()))
                                                            .await;
                                                    }
                                                }
                                            });
                                        }
                                    }
                                    KeyCode::Char('j') | KeyCode::Down => app.next_vod_category(),
                                    KeyCode::Char('k') | KeyCode::Up => app.previous_vod_category(),
                                    KeyCode::Char('f') => {
                                        if !app.vod_categories.is_empty() {
                                            let id = app.vod_categories
                                                [app.selected_vod_category_index]
                                                .category_id
                                                .clone();
                                            app.config.toggle_favorite_vod_category(id);
                                            
                                            // Re-sort categories after favoriting
                                            app.vod_categories.sort_by(|a, b| {
                                                let a_fav = app.config.favorites.vod_categories.contains(&a.category_id);
                                                let b_fav = app.config.favorites.vod_categories.contains(&b.category_id);
                                                
                                                if a.category_id == "ALL" {
                                                    return std::cmp::Ordering::Less;
                                                }
                                                if b.category_id == "ALL" {
                                                    return std::cmp::Ordering::Greater;
                                                }
                                                
                                                match (a_fav, b_fav) {
                                                    (true, false) => std::cmp::Ordering::Less,
                                                    (false, true) => std::cmp::Ordering::Greater,
                                                    _ => a.category_name.cmp(&b.category_name),
                                                }
                                            });
                                            app.all_vod_categories = app.vod_categories.clone();
                                        }
                                    }
                                    KeyCode::Enter => {
                                        if !app.vod_categories.is_empty() {
                                            let cat_id = app.vod_categories
                                                [app.selected_vod_category_index]
                                                .category_id
                                                .clone();
                                            if let Some(client) = &app.current_client {
                                                let client = client.clone();
                                                let tx = tx.clone();
                                                app.state_loading = true;
                                                tokio::spawn(async move {
                                                    // Handle "All Movies" category
                                                    if cat_id == "ALL" {
                                                        match client.get_vod_streams_all().await {
                                                            Ok(streams) => {
                                                                let _ = tx
                                                                    .send(
                                                                        AsyncAction::VodStreamsLoaded(
                                                                            streams,
                                                                        ),
                                                                    )
                                                                    .await;
                                                            }
                                                            Err(e) => {
                                                                let _ = tx
                                                                    .send(AsyncAction::Error(
                                                                        e.to_string(),
                                                                    ))
                                                                    .await;
                                                            }
                                                        }
                                                    } else {
                                                        match client.get_vod_streams(&cat_id).await
                                                        {
                                                            Ok(streams) => {
                                                                let _ = tx
                                                                    .send(
                                                                        AsyncAction::VodStreamsLoaded(
                                                                            streams,
                                                                        ),
                                                                    )
                                                                    .await;
                                                            }
                                                            Err(e) => {
                                                                let _ = tx
                                                                    .send(AsyncAction::Error(
                                                                        e.to_string(),
                                                                    ))
                                                                    .await;
                                                            }
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
                                    KeyCode::Char('/') => {
                                        app.search_mode = true;
                                        app.active_pane = Pane::Streams; // Ensure update_search knows we are in streams
                                        app.search_query.clear();
                                        app.update_search();
                                    }
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
                                    KeyCode::Char('f') => {
                                        if !app.vod_streams.is_empty() {
                                            let stream =
                                                &app.vod_streams[app.selected_vod_stream_index];
                                            let id = match &stream.stream_id {
                                                serde_json::Value::Number(n) => n.to_string(),
                                                serde_json::Value::String(s) => s.clone(),
                                                _ => stream.stream_id.to_string(),
                                            };
                                            app.config.toggle_favorite_vod_stream(id);
                                            
                                            // Re-sort streams after favoriting
                                            app.vod_streams.sort_by(|a, b| {
                                                let a_id = match &a.stream_id {
                                                    serde_json::Value::Number(n) => n.to_string(),
                                                    serde_json::Value::String(s) => s.clone(),
                                                    _ => a.stream_id.to_string(),
                                                };
                                                let b_id = match &b.stream_id {
                                                    serde_json::Value::Number(n) => n.to_string(),
                                                    serde_json::Value::String(s) => s.clone(),
                                                    _ => b.stream_id.to_string(),
                                                };
                                                
                                                let a_fav = app.config.favorites.vod_streams.contains(&a_id);
                                                let b_fav = app.config.favorites.vod_streams.contains(&b_id);
                                                
                                                match (a_fav, b_fav) {
                                                    (true, false) => std::cmp::Ordering::Less,
                                                    (false, true) => std::cmp::Ordering::Greater,
                                                    _ => a.name.cmp(&b.name),
                                                }
                                            });
                                            app.all_vod_streams = app.vod_streams.clone();
                                        }
                                    }
                                    KeyCode::Char('l') => {
                                        // Switch to Live Categories
                                        if let Some(client) = &app.current_client {
                                            let client = client.clone();
                                            let tx = tx.clone();
                                            app.state_loading = true;
                                            tokio::spawn(async move {
                                                match client.get_live_categories().await {
                                                    Ok(cats) => {
                                                        let _ = tx
                                                            .send(AsyncAction::CategoriesLoaded(
                                                                cats,
                                                            ))
                                                            .await;
                                                    }
                                                    Err(e) => {
                                                        let _ = tx
                                                            .send(AsyncAction::Error(e.to_string()))
                                                            .await;
                                                    }
                                                }
                                            });
                                        }
                                    }
                                    KeyCode::Enter => {
                                        if !app.vod_streams.is_empty() {
                                            let stream =
                                                &app.vod_streams[app.selected_vod_stream_index];
                                            if let Some(client) = &app.current_client {
                                                // Handle stream_id safely (could be int or string)
                                                let id = match &stream.stream_id {
                                                    serde_json::Value::Number(n) => n.to_string(),
                                                    serde_json::Value::String(s) => s.clone(),
                                                    _ => stream.stream_id.to_string(),
                                                };

                                                // Use container_extension from API if available, else default to mp4
                                                let extension = stream
                                                    .container_extension
                                                    .as_deref()
                                                    .unwrap_or("mp4");
                                                let url = client.get_vod_url(&id, extension);

                                                // Start player with loading state
                                                app.state_loading = true;
                                                app.player_error = None;
                                                app.loading_message = Some(format!(
                                                    "Preparing Movie: {}...",
                                                    stream.name
                                                ));

                                                let tx = tx.clone();
                                                let player = player.clone();
                                                let stream_url = url.clone();

                                                tokio::spawn(async move {
                                                    let _ = tx
                                                        .send(AsyncAction::LoadingMessage(
                                                            "Resolving video source...".to_string(),
                                                        ))
                                                        .await;
                                                    match player.play(&stream_url) {
                                                        Ok(_) => {
                                                            let _ = tx
                                                                .send(AsyncAction::LoadingMessage(
                                                                    "Buffering movie..."
                                                                        .to_string(),
                                                                ))
                                                                .await;

                                                            // Wait for MPV to confirm playback (polls process status)
                                                            match player
                                                                .wait_for_playback(10000)
                                                                .await
                                                            {
                                                                Ok(true) => {
                                                                    let _ = tx.send(AsyncAction::PlayerStarted).await;
                                                                }
                                                                Ok(false) => {
                                                                    let _ = tx.send(AsyncAction::PlayerFailed("Movie failed to start - MPV exited unexpectedly".to_string())).await;
                                                                }
                                                                Err(e) => {
                                                                    let _ = tx.send(AsyncAction::PlayerFailed(format!("Playback error: {}", e))).await;
                                                                }
                                                            }
                                                        }
                                                        Err(e) => {
                                                            let _ = tx
                                                                .send(AsyncAction::PlayerFailed(
                                                                    e.to_string(),
                                                                ))
                                                                .await;
                                                        }
                                                    }
                                                });
                                            }
                                        }
                                    }
                                    KeyCode::Char('x') => {
                                        app.current_screen = CurrentScreen::Settings
                                    }
                                    _ => {}
                                }
                            }
                        }
                        CurrentScreen::SeriesCategories => {
                            if app.search_mode {
                                // Search mode - handle search input
                                match key.code {
                                    KeyCode::Esc => {
                                        app.search_mode = false;
                                        app.search_query.clear();
                                        // Restore full lists
                                        app.series_categories = app.all_series_categories.clone();
                                        app.series_streams = app.all_series_streams.clone();
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
                                // Normal mode - handle navigation
                                match key.code {
                                    KeyCode::Char('/') => {
                                        app.search_mode = true;
                                        app.search_query.clear();
                                        app.update_search();
                                    }
                                    KeyCode::Esc => {
                                        // Clear series state and go back to content selection
                                        app.series_streams.clear();
                                        app.all_series_streams.clear();
                                        app.selected_series_category_index = 0;
                                        app.selected_series_stream_index = 0;
                                        app.series_category_list_state.select(None);
                                        app.series_stream_list_state.select(None);
                                        // Reset search state
                                        app.search_mode = false;
                                        app.search_query.clear();
                                        app.current_screen = CurrentScreen::ContentTypeSelection;
                                    }
                                    KeyCode::Backspace => {
                                        // Navigate back
                                        app.series_streams.clear();
                                        app.all_series_streams.clear();
                                        app.selected_series_category_index = 0;
                                        app.selected_series_stream_index = 0;
                                        app.series_category_list_state.select(None);
                                        app.series_stream_list_state.select(None);
                                        // Reset search state
                                        app.search_mode = false;
                                        app.search_query.clear();
                                        app.current_screen = CurrentScreen::ContentTypeSelection;
                                    }
                                    KeyCode::Char('l') => {
                                        // Switch to Live
                                        if let Some(client) = &app.current_client {
                                            let client = client.clone();
                                            let tx = tx.clone();
                                            app.state_loading = true;
                                            app.search_mode = false;
                                            app.search_query.clear();
                                            tokio::spawn(async move {
                                                match client.get_live_categories().await {
                                                    Ok(cats) => {
                                                        let _ = tx
                                                            .send(AsyncAction::CategoriesLoaded(cats))
                                                            .await;
                                                    }
                                                    Err(e) => {
                                                        let _ = tx
                                                            .send(AsyncAction::Error(e.to_string()))
                                                            .await;
                                                    }
                                                }
                                            });
                                        }
                                    }
                                    KeyCode::Char('v') | KeyCode::Char('m') => {
                                        // Switch to VOD
                                        if let Some(client) = &app.current_client {
                                            let client = client.clone();
                                            let tx = tx.clone();
                                            app.state_loading = true;
                                            app.search_mode = false;
                                            app.search_query.clear();
                                            tokio::spawn(async move {
                                                match client.get_vod_categories().await {
                                                    Ok(cats) => {
                                                        let _ = tx
                                                            .send(AsyncAction::VodCategoriesLoaded(
                                                                cats,
                                                            ))
                                                            .await;
                                                    }
                                                    Err(e) => {
                                                        let _ = tx
                                                            .send(AsyncAction::Error(e.to_string()))
                                                            .await;
                                                    }
                                                }
                                            });
                                        }
                                    }
                                    KeyCode::Char('j') | KeyCode::Down => {
                                        app.next_series_category();
                                    }
                                    KeyCode::Char('k') | KeyCode::Up => {
                                        app.previous_series_category();
                                    }
                                    KeyCode::Enter => {
                                        if !app.series_categories.is_empty() {
                                            let cat_id = app.series_categories
                                                [app.selected_series_category_index]
                                                .category_id
                                                .clone();
                                            if let Some(client) = &app.current_client {
                                                let client = client.clone();
                                                let tx = tx.clone();
                                                app.state_loading = true;
                                                app.active_pane = Pane::Streams; // Move to streams pane
                                                tokio::spawn(async move {
                                                    match client.get_series_streams(&cat_id).await {
                                                        Ok(streams) => {
                                                            let _ = tx
                                                                .send(AsyncAction::SeriesStreamsLoaded(
                                                                    streams,
                                                                ))
                                                                .await;
                                                        }
                                                        Err(e) => {
                                                            let _ = tx
                                                                .send(AsyncAction::Error(
                                                                    e.to_string(),
                                                                ))
                                                                .await;
                                                        }
                                                    }
                                                });
                                            }
                                        }
                                    }
                                    KeyCode::Tab => {
                                        // Cycle through panes
                                        app.active_pane = match app.active_pane {
                                            Pane::Categories => {
                                                if !app.series_streams.is_empty() {
                                                    Pane::Streams
                                                } else {
                                                    Pane::Categories
                                                }
                                            }
                                            Pane::Streams => {
                                                if !app.series_episodes.is_empty() {
                                                    Pane::Episodes
                                                } else if !app.series_categories.is_empty() {
                                                    Pane::Categories
                                                } else {
                                                    Pane::Streams
                                                }
                                            }
                                            Pane::Episodes => Pane::Categories,
                                        };
                                    }
                                    KeyCode::Char('x') => {
                                        app.current_screen = CurrentScreen::Settings;
                                    }
                                    _ => {}
                                }
                            }
                        }

                        CurrentScreen::SeriesStreams => {
                            if app.search_mode {
                                // Search mode - handle search input
                                match key.code {
                                    KeyCode::Esc => {
                                        app.search_mode = false;
                                        app.search_query.clear();
                                        // Restore full lists
                                        app.series_categories = app.all_series_categories.clone();
                                        app.series_streams = app.all_series_streams.clone();
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
                                // Normal mode - handle navigation
                                match key.code {
                                    KeyCode::Char('/') => {
                                        app.search_mode = true;
                                        app.search_query.clear();
                                        app.update_search();
                                    }
                                    KeyCode::Esc | KeyCode::Backspace | KeyCode::Left => {
                                    // Navigate back based on active pane
                                    match app.active_pane {
                                        Pane::Episodes => {
                                            // Go back to series list
                                            app.series_episodes.clear();
                                            app.selected_series_episode_index = 0;
                                            app.series_episode_list_state.select(None);
                                            app.active_pane = Pane::Streams;
                                            // Reset search state
                                            app.search_mode = false;
                                            app.search_query.clear();
                                        }
                                        Pane::Streams => {
                                            // Go back to categories
                                            app.series_streams.clear();
                                            app.all_series_streams.clear();
                                            app.selected_series_stream_index = 0;
                                            app.series_stream_list_state.select(None);
                                            app.active_pane = Pane::Categories;
                                            // Reset search state
                                            app.search_mode = false;
                                            app.search_query.clear();
                                        }
                                        Pane::Categories => {
                                            // Go back to content selection
                                            app.series_streams.clear();
                                            app.all_series_streams.clear();
                                            app.selected_series_stream_index = 0;
                                            app.series_stream_list_state.select(None);
                                            // Reset search state
                                            app.search_mode = false;
                                            app.search_query.clear();
                                            app.current_screen = CurrentScreen::SeriesCategories;
                                        }
                                    }
                                }
                                KeyCode::Char('j') | KeyCode::Down => {
                                    match app.active_pane {
                                        Pane::Categories => app.next_series_category(),
                                        Pane::Streams => app.next_series_stream(),
                                        Pane::Episodes => app.next_series_episode(),
                                    }
                                }
                                KeyCode::Char('k') | KeyCode::Up => {
                                    match app.active_pane {
                                        Pane::Categories => app.previous_series_category(),
                                        Pane::Streams => app.previous_series_stream(),
                                        Pane::Episodes => app.previous_series_episode(),
                                    }
                                }
                                KeyCode::Tab => {
                                    // Cycle through panes
                                    app.active_pane = match app.active_pane {
                                        Pane::Categories => {
                                            if !app.series_streams.is_empty() {
                                                Pane::Streams
                                            } else {
                                                Pane::Categories
                                            }
                                        }
                                        Pane::Streams => {
                                            if !app.series_episodes.is_empty() {
                                                Pane::Episodes
                                            } else if !app.series_categories.is_empty() {
                                                Pane::Categories
                                            } else {
                                                Pane::Streams
                                            }
                                        }
                                        Pane::Episodes => Pane::Categories,
                                    };
                                }
                                KeyCode::Enter => {
                                    match app.active_pane {
                                        Pane::Categories => {
                                            // Load series for selected category
                                            if !app.series_categories.is_empty() {
                                                let cat_id = app.series_categories
                                                    [app.selected_series_category_index]
                                                    .category_id
                                                    .clone();
                                                if let Some(client) = &app.current_client {
                                                    let client = client.clone();
                                                    let tx = tx.clone();
                                                    app.state_loading = true;
                                                    app.active_pane = Pane::Streams; // Auto-switch to series pane
                                                    tokio::spawn(async move {
                                                        match client.get_series_streams(&cat_id).await {
                                                            Ok(streams) => {
                                                                let _ = tx
                                                                    .send(AsyncAction::SeriesStreamsLoaded(
                                                                        streams,
                                                                    ))
                                                                    .await;
                                                            }
                                                            Err(e) => {
                                                                let _ = tx
                                                                    .send(AsyncAction::Error(
                                                                        e.to_string(),
                                                                    ))
                                                                    .await;
                                                            }
                                                        }
                                                    });
                                                }
                                            }
                                        }
                                        Pane::Streams => {
                                            // Load episodes for selected series
                                            if !app.series_streams.is_empty() {
                                                let stream =
                                                    &app.series_streams[app.selected_series_stream_index];
                                                if let Some(client) = &app.current_client {
                                                    let series_id = match &stream.stream_id {
                                                        serde_json::Value::Number(n) => n.to_string(),
                                                        serde_json::Value::String(s) => s.clone(),
                                                        _ => stream.stream_id.to_string(),
                                                    };

                                                    let client = client.clone();
                                                    let tx = tx.clone();
                                                    app.state_loading = true;
                                                    app.active_pane = Pane::Episodes; // Auto-switch to episodes pane

                                                    tokio::spawn(async move {
                                                        match client.get_series_info(&series_id).await {
                                                            Ok(info) => {
                                                                let _ = tx
                                                                    .send(AsyncAction::SeriesInfoLoaded(info))
                                                                    .await;
                                                            }
                                                            Err(e) => {
                                                                let _ = tx
                                                                    .send(AsyncAction::Error(format!(
                                                                        "Failed to load episodes: {}",
                                                                        e
                                                                    )))
                                                                    .await;
                                                            }
                                                        }
                                                    });
                                                }
                                            }
                                        }
                                        Pane::Episodes => {
                                            // Play selected episode
                                            if !app.series_episodes.is_empty() {
                                                let episode = &app.series_episodes[app.selected_series_episode_index];
                                                
                                                // Build episode URL from direct_source or construct it
                                                if let Some(client) = &app.current_client {
                                                    let episode_url = if !episode.direct_source.is_empty() {
                                                        episode.direct_source.clone()
                                                    } else {
                                                        // Construct URL from episode ID
                                                        let ep_id = match &episode.id {
                                                            Some(serde_json::Value::Number(n)) => n.to_string(),
                                                            Some(serde_json::Value::String(s)) => s.clone(),
                                                            _ => String::new(), // Empty string will be caught below
                                                        };
                                                        
                                                        if ep_id.is_empty() {
                                                            String::new() // Return empty string
                                                        } else {
                                                            let ext = episode.container_extension.as_deref().unwrap_or("mp4");
                                                            format!("{}/series/{}/{}/{}.{}", 
                                                                client.base_url, client.username, client.password, ep_id, ext)
                                                        }
                                                    };

                                                    // Only play if we have a valid URL
                                                    if !episode_url.is_empty() {
                                                        app.state_loading = true;
                                                        app.player_error = None;
                                                        let episode_title = episode.title.as_deref().unwrap_or("Episode");
                                                        app.loading_message = Some(format!(
                                                            "Preparing S{:02}E{:02}: {}...",
                                                            episode.season, episode.episode_num, episode_title
                                                        ));

                                                        let tx = tx.clone();
                                                        let player = player.clone();

                                                        tokio::spawn(async move {
                                                            match player.play(&episode_url) {
                                                                Ok(_) => {
                                                                    match player.wait_for_playback(10000).await {
                                                                        Ok(true) => {
                                                                            let _ = tx.send(AsyncAction::PlayerStarted).await;
                                                                        }
                                                                        Ok(false) => {
                                                                            let _ = tx.send(AsyncAction::PlayerFailed(
                                                                                "Episode failed to start".to_string()
                                                                            )).await;
                                                                        }
                                                                        Err(e) => {
                                                                            let _ = tx.send(AsyncAction::PlayerFailed(
                                                                                format!("Playback error: {}", e)
                                                                            )).await;
                                                                        }
                                                                    }
                                                                }
                                                                Err(e) => {
                                                                    let _ = tx.send(AsyncAction::PlayerFailed(e.to_string())).await;
                                                                }
                                                            }
                                                        });
                                                    }
                                                }
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
                                matrix_iptv_lib::app::SettingsState::Main => {
                                    match key.code {
                                        KeyCode::Esc | KeyCode::Backspace => {
                                            app.current_screen = CurrentScreen::Home
                                        }
                                        KeyCode::Char('j') | KeyCode::Down => app.next_setting(),
                                        KeyCode::Char('k') | KeyCode::Up => app.previous_setting(),
                                        KeyCode::Enter => {
                                            // Handle settings action
                                            if !app.settings_options.is_empty() {
                                                match app.selected_settings_index {
                                                    0 => {
                                                        // Sub-menu: Manage Accounts
                                                        app.settings_state = matrix_iptv_lib::app::SettingsState::ManageAccounts;
                                                        // Reuse account list state for this view
                                                        if !app.config.accounts.is_empty() {
                                                            app.account_list_state.select(Some(0));
                                                        }
                                                    }
                                                    1 => {
                                                        // Set Timezone
                                                        let current_tz =
                                                            app.config.get_user_timezone();
                                                        app.input_timezone = tui_input::Input::new(
                                                            current_tz.clone(),
                                                        );

                                                        // Pre-select in list
                                                        if let Some(pos) = app
                                                            .timezone_list
                                                            .iter()
                                                            .position(|x| x == &current_tz)
                                                        {
                                                            app.timezone_list_state
                                                                .select(Some(pos));
                                                        } else {
                                                            app.timezone_list_state.select(Some(0));
                                                        }

                                                        app.current_screen =
                                                            CurrentScreen::TimezoneSettings;
                                                    }
                                                    2 => {
                                                        // Matrix Rain Screensaver
                                                        app.show_matrix_rain = true;
                                                        app.matrix_rain_screensaver_mode = true; // Screensaver mode (no logo)
                                                        app.matrix_rain_start_time = Some(std::time::Instant::now());
                                                        // Rain will be initialized automatically in the render loop
                                                    }
                                                    3 => {
                                                        // About
                                                        app.settings_state =
                                                            matrix_iptv_lib::app::SettingsState::About;
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                                matrix_iptv_lib::app::SettingsState::ManageAccounts => {
                                    match key.code {
                                        KeyCode::Esc | KeyCode::Backspace => {
                                            app.settings_state =
                                                matrix_iptv_lib::app::SettingsState::Main
                                        }
                                        KeyCode::Char('j') | KeyCode::Down => app.next_account(),
                                        KeyCode::Char('k') | KeyCode::Up => app.previous_account(),
                                        KeyCode::Char('n') => {
                                            // Add new account
                                            app.current_screen = CurrentScreen::Login;
                                            app.previous_screen = Some(CurrentScreen::Settings);
                                            app.editing_account_index = None;
                                            app.input_name = tui_input::Input::default();
                                            app.input_url = tui_input::Input::default();
                                            app.input_username = tui_input::Input::default();
                                            app.input_password = tui_input::Input::default();
                                            app.input_epg_url = tui_input::Input::default();
                                            app.login_error = None;
                                        }
                                        KeyCode::Char('d') => {
                                            // Delete account
                                            if !app.config.accounts.is_empty() {
                                                if let Some(idx) = app.account_list_state.selected()
                                                {
                                                    app.config.accounts.remove(idx);
                                                    // Save config
                                                    let _ = app.config.save();

                                                    if app.config.accounts.is_empty() {
                                                        app.account_list_state.select(None);
                                                    } else if idx >= app.config.accounts.len() {
                                                        app.account_list_state.select(Some(
                                                            app.config.accounts.len() - 1,
                                                        ));
                                                    }
                                                }
                                            }
                                        }
                                        KeyCode::Enter => {
                                            // Edit selected
                                            if !app.config.accounts.is_empty() {
                                                if let Some(idx) = app.account_list_state.selected()
                                                {
                                                    let acc = &app.config.accounts[idx];
                                                    app.input_name =
                                                        tui_input::Input::new(acc.name.clone());
                                                    app.input_url =
                                                        tui_input::Input::new(acc.base_url.clone());
                                                    app.input_username =
                                                        tui_input::Input::new(acc.username.clone());
                                                    app.input_password =
                                                        tui_input::Input::new(acc.password.clone());
                                                    app.input_epg_url = tui_input::Input::new(
                                                        acc.epg_url.clone().unwrap_or_default(),
                                                    );

                                                    app.editing_account_index = Some(idx);
                                                    app.current_screen = CurrentScreen::Login;
                                                    app.previous_screen =
                                                        Some(CurrentScreen::Settings);
                                                    app.login_field_focus = LoginField::Name;
                                                    app.login_error = None;
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                                matrix_iptv_lib::app::SettingsState::About => match key.code {
                                    KeyCode::Esc | KeyCode::Backspace => {
                                        app.settings_state = matrix_iptv_lib::app::SettingsState::Main;
                                        app.about_scroll = 0;
                                    }
                                    KeyCode::Down | KeyCode::Char('j') => {
                                        app.about_scroll = app.about_scroll.saturating_add(1)
                                    }
                                    KeyCode::Up | KeyCode::Char('k') => {
                                        app.about_scroll = app.about_scroll.saturating_sub(1)
                                    }
                                    _ => {}
                                },
                            }
                        }
                        CurrentScreen::TimezoneSettings => {
                            match key.code {
                                KeyCode::Esc => app.current_screen = CurrentScreen::Settings,
                                KeyCode::Enter => {
                                    let val = app.input_timezone.value().to_string();
                                    if !val.is_empty() {
                                        app.config.timezone = Some(val);
                                        let _ = app.config.save();
                                    }
                                    app.current_screen = CurrentScreen::Settings;
                                    app.refresh_settings_options();
                                }
                                KeyCode::Up => {
                                    app.previous_timezone();
                                    // Auto-fill input with selected
                                    if let Some(idx) = app.timezone_list_state.selected() {
                                        let content = app.timezone_list[idx].clone();
                                        app.input_timezone = tui_input::Input::new(content);
                                    }
                                }
                                KeyCode::Down => {
                                    app.next_timezone();
                                    // Auto-fill input with selected
                                    if let Some(idx) = app.timezone_list_state.selected() {
                                        let content = app.timezone_list[idx].clone();
                                        app.input_timezone = tui_input::Input::new(content);
                                    }
                                }
                                _ => {
                                    tui_input::backend::crossterm::EventHandler::handle_event(
                                        &mut app.input_timezone,
                                        &Event::Key(key),
                                    );
                                }
                            }
                        }
                        _ => {}
                    }
                    // Global help toggle removed from here as it's handled above
                } // End Event::Key block

                Event::Mouse(mouse) => {
                    use matrix_iptv_lib::app::Pane;
                    match mouse.kind {
                        MouseEventKind::Down(MouseButton::Left) => {
                            let x = mouse.column;
                            let y = mouse.row;

                            match app.current_screen {
                                CurrentScreen::Home => {
                                    if x >= app.area_accounts.x
                                        && x < app.area_accounts.x + app.area_accounts.width
                                        && y > app.area_accounts.y
                                        && y < app.area_accounts.y + app.area_accounts.height
                                    {
                                        // Simple row selection (won't be perfect if scrolled, but better than nothing)
                                        let row = (y - app.area_accounts.y - 1) as usize;
                                        if row < app.config.accounts.len() {
                                            app.selected_account_index = row;
                                            app.account_list_state.select(Some(row));
                                        }
                                    }
                                }
                                CurrentScreen::Categories
                                | CurrentScreen::Streams
                                | CurrentScreen::VodCategories
                                | CurrentScreen::VodStreams => {
                                    if x >= app.area_categories.x
                                        && x < app.area_categories.x + app.area_categories.width
                                        && y >= app.area_categories.y
                                        && y < app.area_categories.y + app.area_categories.height
                                    {
                                        app.active_pane = Pane::Categories;
                                    } else if x >= app.area_streams.x
                                        && x < app.area_streams.x + app.area_streams.width
                                        && y >= app.area_streams.y
                                        && y < app.area_streams.y + app.area_streams.height
                                    {
                                        app.active_pane = Pane::Streams;
                                    }
                                }
                                _ => {}
                            }
                        }
                        MouseEventKind::ScrollDown => {
                            if app.show_guide.is_some() {
                                app.guide_scroll = app.guide_scroll.saturating_add(1);
                            } else {
                                match app.current_screen {
                                    CurrentScreen::Home => app.next_account(),
                                    CurrentScreen::Categories | CurrentScreen::Streams => {
                                        match app.active_pane {
                                            Pane::Categories => app.next_category(),
                                            Pane::Streams => app.next_stream(),
                                            _ => {}
                                        }
                                    }
                                    CurrentScreen::VodCategories => app.next_vod_category(),
                                    CurrentScreen::VodStreams => app.next_vod_stream(),
                                    CurrentScreen::Settings => app.next_setting(),
                                    _ => {}
                                }
                            }
                        }
                        MouseEventKind::ScrollUp => {
                            if app.show_guide.is_some() {
                                app.guide_scroll = app.guide_scroll.saturating_sub(1);
                            } else {
                                match app.current_screen {
                                    CurrentScreen::Home => app.previous_account(),
                                    CurrentScreen::Categories | CurrentScreen::Streams => {
                                        match app.active_pane {
                                            Pane::Categories => app.previous_category(),
                                            Pane::Streams => app.previous_stream(),
                                            _ => {}
                                        }
                                    }
                                    CurrentScreen::VodCategories => app.previous_vod_category(),
                                    CurrentScreen::VodStreams => app.previous_vod_stream(),
                                    CurrentScreen::Settings => app.previous_setting(),
                                    _ => {}
                                }
                            }
                        }
                        _ => {}
                    }
                }

                _ => {} // Other events (resize, etc.)
            }
        }
    }
}
