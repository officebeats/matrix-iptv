use std::{io, time::Duration};

#[cfg(not(target_arch = "wasm32"))]
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

#[cfg(not(target_arch = "wasm32"))]
use ratatui::{backend::CrosstermBackend, Terminal};
#[cfg(not(target_arch = "wasm32"))]
use tokio::sync::mpsc;
use matrix_iptv_lib::app::{App, AsyncAction, CurrentScreen, Pane};
use matrix_iptv_lib::api::get_id_str;
use matrix_iptv_lib::{player, setup, ui, handlers};

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
        player.play(&url, false)?; // Use optimized settings for CLI play
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
            handlers::async_actions::handle_async_action(app, action, &tx).await;
        }

        app.loading_tick = app.loading_tick.wrapping_add(1);

        // 1.5 Debounced EPG Fetching
        if app.current_screen == CurrentScreen::Streams && app.active_pane == Pane::Streams && !app.streams.is_empty() {
            let focused_id = get_id_str(&app.streams[app.selected_stream_index].stream_id);
            if app.last_focused_stream_id.as_ref() != Some(&focused_id) {
                app.last_focused_stream_id = Some(focused_id.clone());
                app.focus_timestamp = Some(std::time::Instant::now());
            } else if let Some(ts) = app.focus_timestamp {
                if ts.elapsed().as_millis() >= 300 {
                    app.focus_timestamp = None; // Reset so we don't spam
                    if !app.epg_cache.contains_key(&focused_id) {
                        if let Some(client) = &app.current_client {
                            let client = client.clone();
                            let tx = tx.clone();
                            let fid = focused_id.clone();
                            tokio::spawn(async move {
                                if let Ok(epg) = client.get_short_epg(&fid).await {
                                    if let Some(now_playing) = epg.epg_listings.get(0) {
                                        let _ = tx.send(AsyncAction::EpgLoaded(fid, now_playing.title.clone())).await;
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
                    app.loading_tick, 
                    &mut app.matrix_rain_logo_hits, 
                    !app.matrix_rain_screensaver_mode
                );
                
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
        if event::poll(Duration::from_millis(33))? {
            match event::read()? {
                Event::Key(key) => {
                    match handlers::input::handle_key_event(app, key, &tx, player).await? {
                        handlers::input::InputResult::Quit => return Ok(()),
                        handlers::input::InputResult::Continue => continue,
                        handlers::input::InputResult::Ok => {}
                    }
                } // End Event::Key block

                Event::Mouse(mouse) => {
                    handlers::mouse::handle_mouse_event(app, mouse);
                }

                _ => {} // Other events (resize, etc.)
            }
        }
    }
}
