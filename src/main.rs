use std::{io, time::Duration};
use tokio::time::interval;

#[cfg(not(target_arch = "wasm32"))]
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, Clear, ClearType},
};

#[cfg(not(target_arch = "wasm32"))]
use ratatui::{backend::CrosstermBackend, Terminal};
#[cfg(not(target_arch = "wasm32"))]
use tokio::sync::mpsc;
use matrix_iptv_lib::app::{App, AsyncAction, CurrentScreen, Pane};
use matrix_iptv_lib::api::get_id_str;
use matrix_iptv_lib::{player, setup, ui, handlers, sports};

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Optional Direct Play URL (if provided, plays and exits)
    #[arg(short, long)]
    play: Option<String>,

    /// Check configuration and verify login
    #[arg(long)]
    check: bool,

    /// Skip checking for updates on startup
    #[arg(long)]
    skip_update: bool,
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
        player.play(&url, matrix_iptv_lib::config::PlayerEngine::Vlc, false, true).await?; // Use optimized VLC with smoothing for CLI play
        return Ok(());
    }

    // -- TUI MODE (Default) --

    // Check Dependencies First
    setup::check_and_install_dependencies()?;

    // Setup Terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture, Clear(ClearType::All))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // App State
    let mut app = App::new();
    let player = player::Player::new();

    // Check if FTUE is needed
    if app.config.accounts.is_empty() {
        if let Ok(Some(new_account)) = matrix_iptv_lib::onboarding::run_onboarding(&mut terminal) {
            app.config.accounts.push(new_account);
            if let Err(_) = app.config.save() {
                // Ignore save error here, it will be handled when main loop starts
            }
            // re-init app state now that we have an account
            // State will be updated by normal app loop
            // app.apply_category_filters(); // Optional: explicitly update filters
        } else {
            // User quit onboarding
            disable_raw_mode()?;
            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;
            terminal.show_cursor()?;
            return Ok(());
        }
    } else {
        // Run matrix rain screensaver natively if wanted (optional)
        app.show_welcome_popup = false; // We use FTUE now
    }

    // Async Channel
    let (tx, mut rx) = mpsc::channel::<AsyncAction>(1024);

    // Initial background tasks
    if !args.skip_update {
        let tx_update = tx.clone();
        tokio::spawn(async move {
            matrix_iptv_lib::setup::check_for_updates(tx_update, false).await;
        });
    }

    // Score Fetcher Task
    let tx_scores = tx.clone();
    tokio::spawn(async move {
        let service = matrix_iptv_lib::scores::ScoreService::new();
        // Initial fetch delayed by 5s to allow startup
        tokio::time::sleep(Duration::from_secs(5)).await;
        
        // Loop every 60s
        let mut ticker = interval(Duration::from_secs(60));
        loop {
            ticker.tick().await; 
             if let Ok(scores) = service.fetch_scores().await {
                 let _ = tx_scores.send(AsyncAction::ScoresLoaded(scores)).await;
             }
        }
    });

    let res = run_app(&mut terminal, &mut app, &player, tx, &mut rx).await;

    // Restore Terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    let exit_code = match res {
        Ok(Some(code)) => code,
        Ok(None) => 0,
        Err(err) => {
            println!("{:?}", err);
            1
        }
    };

    if exit_code == 42 {
        // On Windows, handle the update directly from the binary to avoid
        // the EBUSY bug in older versions of cli.js
        #[cfg(target_os = "windows")]
        {
            if let Err(e) = setup::perform_windows_self_update() {
                eprintln!("\n[!] Self-update failed: {}. Falling back to CLI updater.", e);
                std::process::exit(42);
            }
            std::process::exit(0);
        }

        #[cfg(not(target_os = "windows"))]
        std::process::exit(42);
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
) -> io::Result<Option<i32>> {
    #[allow(unused_assignments)]
    let mut needs_redraw = true;

    loop {
        if app.needs_stream_refresh {
            app.refresh_streams_from_cache();
            app.needs_stream_refresh = false;
            needs_redraw = true;
        }
        
        // Debounce expired: the full UI projection is now zero-cost and renders immediately.

        if needs_redraw {
            terminal.draw(|f| ui::ui(f, app)).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
            needs_redraw = false;
        }

        // 1. Check for Async Actions (Non-blocking)
        while let Ok(action) = rx.try_recv() {
            handlers::async_actions::handle_async_action(app, action, &tx).await;
            needs_redraw = true;
        }

        // 1.1 Process lazy category loads
        while let Some(action) = app.pending_lazy_loads.pop_front() {
            handlers::async_actions::handle_async_action(app, action, &tx).await;
            needs_redraw = true;
        }

        app.session.loading_tick = app.session.loading_tick.wrapping_add(1);
        if app.session.state_loading || app.show_matrix_rain {
            needs_redraw = true;
        }

        // Force continuous redraw to drive the animated lava-lamp typographic borders
        needs_redraw = true;

        // 1.5 Batch EPG Fetching — prefetch for all visible streams, not just the focused one
        if app.current_screen == CurrentScreen::Streams && app.active_pane == Pane::Streams && !app.streams.is_empty() {
            let focused_id = get_id_str(&app.streams[app.selected_stream_index].stream_id);
            if app.last_focused_stream_id.as_ref() != Some(&focused_id) {
                app.last_focused_stream_id = Some(focused_id.clone());
                app.focus_timestamp = Some(std::time::Instant::now());
            } else if let Some(ts) = app.focus_timestamp {
                if ts.elapsed().as_millis() >= 200 {
                    app.focus_timestamp = None;
                    
                    // Collect all visible stream IDs that aren't already cached
                    let mut uncached_ids: Vec<String> = Vec::new();
                    let visible_count = 40.min(app.streams.len()); // Fetch up to 40 visible
                    let start = app.selected_stream_index.saturating_sub(20);
                    let end = (start + visible_count).min(app.streams.len());
                    
                    for i in start..end {
                        let sid = get_id_str(&app.streams[i].stream_id);
                        if !app.epg_cache.contains_key(&sid) {
                            uncached_ids.push(sid);
                        }
                    }
                    
                    // Also ensure the focused stream is included
                    if !app.epg_cache.contains_key(&focused_id) && !uncached_ids.contains(&focused_id) {
                        uncached_ids.insert(0, focused_id.clone());
                    }
                    
                    if !uncached_ids.is_empty() {
                        if let Some(client) = &app.session.current_client {
                            let client = client.clone();
                            let tx = tx.clone();
                            tokio::spawn(async move {
                                let mut results = Vec::new();
                                // Fetch EPG sequentially (avoids server hammering)
                                for sid in uncached_ids {
                                    if let Ok(epg) = client.get_short_epg(&sid).await {
                                        if let Some(now_playing) = epg.epg_listings.get(0) {
                                            results.push((sid, now_playing.title.clone()));
                                        }
                                    }
                                }
                                
                                if !results.is_empty() {
                                    let _ = tx.send(AsyncAction::EpgBatchLoaded(results)).await;
                                }
                            });
                        }
                    }
                }
            }
        }

        // 1.6 Debounced Stream Health Check
        if (app.current_screen == CurrentScreen::Streams && app.active_pane == Pane::Streams && !app.streams.is_empty()) ||
           (app.current_screen == CurrentScreen::GlobalSearch && !app.global_search_results.is_empty()) {
            
            let focused_stream = if app.current_screen == CurrentScreen::GlobalSearch {
                app.global_search_results.get(app.selected_stream_index)
            } else {
                app.streams.get(app.selected_stream_index)
            };

            if let Some(stream) = focused_stream {
                let focused_id = get_id_str(&stream.stream_id);
                if stream.latency_ms.is_none() {
                    if let Some(client) = &app.session.current_client {
                        let client = client.clone();
                        let tx = tx.clone();
                        let fid = focused_id.clone();
                        let ext = stream.container_extension.as_deref().unwrap_or("ts");
                        let url = client.get_stream_url(&fid, ext);
                        
                        // We use a small delay to avoid spamming while scrolling
                        if app.focus_timestamp.is_none() {
                            app.focus_timestamp = Some(std::time::Instant::now());
                        } else if app.focus_timestamp.unwrap().elapsed().as_millis() >= 1000 {
                            app.focus_timestamp = None; // Reset
                            tokio::spawn(async move {
                                let start = std::time::Instant::now();
                                let req_client = reqwest::Client::builder()
                                    .timeout(std::time::Duration::from_secs(3))
                                    .build()
                                    .unwrap_or_default();
                                
                                if let Ok(resp) = req_client.head(&url).send().await {
                                    if resp.status().is_success() {
                                        let latency = start.elapsed().as_millis() as u64;
                                        let _ = tx.send(AsyncAction::StreamHealthLoaded(fid, latency)).await;
                                    } else {
                                        let _ = tx.send(AsyncAction::StreamHealthLoaded(fid, 2000)).await;
                                    }
                                } else {
                                    let _ = tx.send(AsyncAction::StreamHealthLoaded(fid, 5000)).await;
                                }
                            });
                        }
                    }
                }
            }
        }
        
        // 1.7 Debounced VOD Info Fetching
        if app.current_screen == CurrentScreen::VodStreams && app.active_pane == Pane::Streams && !app.vod_streams.is_empty() {
            let focused_id = get_id_str(&app.vod_streams[app.selected_vod_stream_index].stream_id);
            if app.last_focused_stream_id.as_ref() != Some(&focused_id) {
                app.last_focused_stream_id = Some(focused_id.clone());
                app.focus_timestamp = Some(std::time::Instant::now());
            } else if let Some(ts) = app.focus_timestamp {
                if ts.elapsed().as_millis() >= 500 {
                    app.focus_timestamp = None;
                    if let Some(client) = &app.session.current_client {
                        let client = client.clone();
                        let tx = tx.clone();
                        let fid = focused_id.clone();
                        tokio::spawn(async move {
                            if let Ok(info) = client.get_vod_info(&fid).await {
                                let _ = tx.send(AsyncAction::VodInfoLoaded(info)).await;
                            }
                        });
                    }
                }
            }
        }

        // 1.7.5 Debounced Series Info Fetching
        if app.current_screen == CurrentScreen::SeriesStreams && app.active_pane == Pane::Streams && !app.series_streams.is_empty() {
            let focused_id = get_id_str(&app.series_streams[app.selected_series_stream_index].stream_id);
            if app.last_focused_stream_id.as_ref() != Some(&focused_id) {
                app.last_focused_stream_id = Some(focused_id.clone());
                app.focus_timestamp = Some(std::time::Instant::now());
            } else if let Some(ts) = app.focus_timestamp {
                if ts.elapsed().as_millis() >= 500 {
                    app.focus_timestamp = None;
                    if let Some(client) = &app.session.current_client {
                        let client = client.clone();
                        let tx = tx.clone();
                        let fid = focused_id.clone();
                        tokio::spawn(async move {
                            if let Ok(info) = client.get_series_info(&fid).await {
                                let _ = tx.send(AsyncAction::SeriesInfoLoaded(info)).await;
                            }
                        });
                    }
                }
            }
        }

        // 1.8 Debounced Info Fetching for Global Search
        if app.current_screen == CurrentScreen::GlobalSearch && !app.global_search_results.is_empty() {
            let stream = &app.global_search_results[app.selected_stream_index];
            if stream.stream_type == "movie" || stream.stream_type == "series" {
                let focused_id = get_id_str(&stream.stream_id);
                if app.last_focused_stream_id.as_ref() != Some(&focused_id) {
                    app.last_focused_stream_id = Some(focused_id.clone());
                    app.focus_timestamp = Some(std::time::Instant::now());
                } else if let Some(ts) = app.focus_timestamp {
                    if ts.elapsed().as_millis() >= 500 {
                        app.focus_timestamp = None;
                        if let Some(client) = &app.session.current_client {
                            let client = client.clone();
                            let tx = tx.clone();
                            let fid = focused_id.clone();
                            let is_series = stream.stream_type == "series";
                            tokio::spawn(async move {
                                if is_series {
                                    if let Ok(info) = client.get_series_info(&fid).await {
                                        let _ = tx.send(AsyncAction::SeriesInfoLoaded(info)).await;
                                    }
                                } else {
                                    if let Ok(info) = client.get_vod_info(&fid).await {
                                        let _ = tx.send(AsyncAction::VodInfoLoaded(info)).await;
                                    }
                                }
                            });
                        }
                    }
                }
            }
        }

        // 1.10 Debounced Sports Info Fetching (Matches list)
        if app.current_screen == CurrentScreen::SportsDashboard && app.sports_matches.is_empty() && !app.session.state_loading {
            let tx = tx.clone();
            let category = app.sports_categories[app.selected_sports_category_index].clone();
            app.session.state_loading = true;
            tokio::spawn(async move {
                if let Ok(matches) = sports::fetch_streamed_matches(&category).await {
                    let _ = tx.send(AsyncAction::SportsMatchesLoaded(matches)).await;
                } else {
                    let _ = tx.send(AsyncAction::Error("System Protocol: Failed to link with sports uplink.".to_string())).await;
                }
            });
        }

        // 1.11 Debounced Sports Stream Link Fetching
        if app.current_screen == CurrentScreen::SportsDashboard && !app.sports_matches.is_empty() {
            if let Some(selected_match) = app.sports_matches.get(app.sports_list_state.selected().unwrap_or(0)) {
                let focused_id = selected_match.id.clone();
                if app.last_focused_stream_id.as_ref() != Some(&focused_id) {
                    app.last_focused_stream_id = Some(focused_id.clone());
                    app.focus_timestamp = Some(std::time::Instant::now());
                    app.sports_details_loading = true;
                } else if let Some(ts) = app.focus_timestamp {
                    if ts.elapsed().as_millis() >= 500 {
                        app.focus_timestamp = None;
                        let tx = tx.clone();
                        // Find first available source
                        if let Some(source) = selected_match.sources.first() {
                            let source_name = source.source.clone();
                            let source_id = source.id.clone();
                            tokio::spawn(async move {
                                if let Ok(links) = sports::fetch_streamed_links(&source_name, &source_id).await {
                                    let _ = tx.send(AsyncAction::SportsStreamsLoaded(links)).await;
                                }
                            });
                        }
                    }
                }
            }
        }

        // 1.12 Debounced Sports Matching for regular streams
        // Use cached_parsed to avoid expensive parse_stream() calls every frame
        if (app.current_screen == CurrentScreen::Streams || app.current_screen == CurrentScreen::GlobalSearch) && app.active_pane == Pane::Streams {
            let focused_stream = if app.current_screen == CurrentScreen::GlobalSearch {
                app.global_search_results.get(app.selected_stream_index)
            } else {
                app.streams.get(app.selected_stream_index)
            };

            if let Some(stream) = focused_stream {
                // Use cached parse result; only fall back to parse_stream if not cached
                let has_sports = if let Some(ref cached) = stream.cached_parsed {
                    cached.sports_event.is_some()
                } else {
                    false // Skip sports matching if not cached — not worth parsing every frame
                };
                if has_sports {
                    let stream_id = get_id_str(&stream.stream_id);
                    if app.last_focused_stream_id.as_ref() != Some(&stream_id) {
                        app.last_focused_stream_id = Some(stream_id.clone());
                        app.focus_timestamp = Some(std::time::Instant::now());
                        app.current_sports_streams.clear();
                    } else if let Some(ts) = app.focus_timestamp {
                        if ts.elapsed().as_millis() >= 1000 {
                            app.focus_timestamp = None;
                            let tx = tx.clone();
                            // Extract team names from cached parse
                            let (team1, team2) = if let Some(ref cached) = stream.cached_parsed {
                                if let Some(ref ev) = cached.sports_event {
                                    (ev.team1.clone(), ev.team2.clone())
                                } else { continue; }
                            } else { continue; };
                            
                            tokio::spawn(async move {
                                if let Ok(matches) = sports::fetch_streamed_matches("live").await {
                                    let found = matches.into_iter().find(|m| {
                                        let title = m.title.to_lowercase();
                                        title.contains(&team1.to_lowercase()) || title.contains(&team2.to_lowercase())
                                    });
                                    if let Some(m) = found {
                                        if let Some(source) = m.sources.first() {
                                            if let Ok(links) = sports::fetch_streamed_links(&source.source, &source.id).await {
                                                let _ = tx.send(AsyncAction::SportsStreamsLoaded(links)).await;
                                            }
                                        }
                                    }
                                }
                            });
                        }
                    }
                }
            }
        }
        
        // FTUE: Handle Matrix rain animation
        if app.show_matrix_rain {
            if let Ok(size) = terminal.size() {
                let rect = ratatui::layout::Rect::new(0, 0, size.width, size.height);
                // Initialize columns if not already done
                if app.matrix_rain_columns.is_empty() {
                    app.matrix_rain_columns = matrix_iptv_lib::matrix_rain::init_matrix_rain(rect);
                }
                
                // Update animation
                matrix_iptv_lib::matrix_rain::update_matrix_rain(
                    &mut app.matrix_rain_columns, 
                    rect, 
                    app.session.loading_tick, 
                    &mut app.matrix_rain_logo_hits, 
                    !app.matrix_rain_screensaver_mode
                );
                
                // Force continuous terminal redraws during animation sequences
                needs_redraw = true;
                
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
        let mut timeout_ms = 33;
        if app.session.state_loading || app.show_matrix_rain {
            timeout_ms = 16;
        }
        
        // Yield executor and wait asynchronously to avoid `crossterm::event::poll` hanging indefinitely on Windows
        tokio::time::sleep(Duration::from_millis(timeout_ms)).await;
        
        // Non-blocking poll
        if event::poll(Duration::from_millis(0))? {
            match event::read()? {
                Event::Key(key) => {
                    match handlers::input::handle_key_event(app, key, &tx, player).await? {
                        handlers::input::InputResult::Quit => return Ok(None),
                        handlers::input::InputResult::UpdateRequested => return Ok(Some(42)),
                        handlers::input::InputResult::Continue => { needs_redraw = true; continue; },
                        handlers::input::InputResult::Ok => { needs_redraw = true; }
                    }
                    // ── Input Coalescing: drain queued keys without redrawing ──
                    // When scrolling fast, multiple key events queue up. Process them
                    // all in one batch to avoid 33ms render + debounce between each.
                    while event::poll(Duration::from_millis(0))? {
                        if let Event::Key(next_key) = event::read()? {
                            match handlers::input::handle_key_event(app, next_key, &tx, player).await? {
                                handlers::input::InputResult::Quit => return Ok(None),
                                handlers::input::InputResult::UpdateRequested => return Ok(Some(42)),
                                handlers::input::InputResult::Continue => { needs_redraw = true; continue; },
                                handlers::input::InputResult::Ok => { needs_redraw = true; }
                            }
                        }
                    }
                } // End Event::Key block

                Event::Mouse(mouse) => {
                    handlers::mouse::handle_mouse_event(app, mouse, &tx);
                    needs_redraw = true;
                }

                Event::Resize(_, _) => {
                    needs_redraw = true;
                }

                _ => {} // Other events
            }
        }
    }
}
