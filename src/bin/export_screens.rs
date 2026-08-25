use std::sync::Arc;
use std::fs::File;
use std::io::Write;
use std::collections::HashSet;
use chrono::Utc;
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use ratatui::style::{Color, Modifier};
use matrix_iptv_lib::app::{App, CurrentScreen, Pane, LoginField, InputMode, SettingsState};
use matrix_iptv_lib::api::{Category, Stream};
use matrix_iptv_lib::config::{Account, AccountType, CategorySortOrder};
use matrix_iptv_lib::flex_id::FlexId;
use matrix_iptv_lib::sports::{StreamedMatch, StreamedStream, StreamedTeams, StreamedTeam, StreamedSource};
use serde::Serialize;

#[derive(Serialize)]
struct ScreenExport {
    name: String,
    title: String,
    description: String,
    width: u16,
    height: u16,
    cells: Vec<CellExport>,
}

#[derive(Serialize)]
struct CellExport {
    x: u16,
    y: u16,
    char: String,
    fg: [u8; 3],
    bg: [u8; 3],
    bold: bool,
    dim: bool,
    italic: bool,
    underline: bool,
}

fn color_to_rgb(color: Color, default_fg: bool) -> [u8; 3] {
    match color {
        Color::Rgb(r, g, b) => [r, g, b],
        Color::Black => [0, 0, 0],
        Color::Red => [205, 49, 49],
        Color::Green => [0, 255, 65], // Matrix Green
        Color::Yellow => [229, 229, 16],
        Color::Blue => [36, 114, 200],
        Color::Magenta => [188, 63, 188],
        Color::Cyan => [0, 255, 255],
        Color::Gray => [156, 163, 176],
        Color::DarkGray => [60, 75, 65],
        Color::LightRed => [255, 71, 87],
        Color::LightGreen => [0, 255, 136],
        Color::LightYellow => [255, 220, 100],
        Color::LightBlue => [80, 160, 255],
        Color::LightMagenta => [255, 120, 255],
        Color::LightCyan => [150, 255, 255],
        Color::White => [240, 245, 240],
        Color::Indexed(i) => [i, i, i],
        Color::Reset => {
            if default_fg {
                [0, 255, 65] // Matrix Green
            } else {
                [0, 0, 0] // Black
            }
        }
    }
}

fn make_account(name: &str, url: &str, user: &str, pass: &str, acc_type: AccountType, total_c: usize, total_m: usize, total_s: usize) -> Account {
    Account {
        name: name.to_string(),
        base_url: url.to_string(),
        username: user.to_string(),
        password: pass.to_string(),
        account_type: acc_type,
        epg_url: None,
        last_refreshed: Some(1787500000),
        total_channels: Some(total_c),
        total_movies: Some(total_m),
        total_series: Some(total_s),
        server_timezone: Some("America/New_York".to_string()),
        hidden_categories: HashSet::new(),
        category_sort_order: CategorySortOrder::Default,
    }
}

fn make_stream(id: u64, name: &str) -> Arc<Stream> {
    let now = Utc::now();
    let mut parsed = matrix_iptv_lib::parser::parse_stream(name, Some("America/New_York"));
    parsed.start_time = Some(now - chrono::Duration::minutes(45));
    parsed.stop_time = Some(now + chrono::Duration::minutes(75));

    Arc::new(Stream {
        name: name.to_string(),
        stream_id: FlexId::Number(id as i64),
        search_name: name.to_lowercase(),
        is_english: true,
        latency_ms: Some(42),
        added: Some("1787000000".to_string()),
        rating: Some(8.8),
        cached_parsed: Some(Box::new(parsed)),
        ..Default::default()
    })
}

fn make_vod(id: u64, name: &str, rating: Option<f32>) -> Arc<Stream> {
    Arc::new(Stream {
        name: name.to_string(),
        stream_id: FlexId::Number(id as i64),
        stream_type: "movie".to_string(),
        search_name: name.to_lowercase(),
        rating,
        is_english: true,
        latency_ms: Some(38),
        ..Default::default()
    })
}

fn make_series(id: u64, name: &str) -> Arc<Stream> {
    Arc::new(Stream {
        name: name.to_string(),
        stream_id: FlexId::Number(id as i64),
        stream_type: "series".to_string(),
        search_name: name.to_lowercase(),
        is_english: true,
        latency_ms: Some(45),
        ..Default::default()
    })
}

fn make_cat(id: &str, name: &str, is_sports: bool) -> Arc<Category> {
    Arc::new(Category {
        category_id: id.to_string(),
        category_name: name.to_string(),
        search_name: name.to_lowercase(),
        parent_id: FlexId::Number(0),
        is_sports,
        ..Default::default()
    })
}

fn render_screen(app: &mut App, name: &str, title: &str, description: &str, width: u16, height: u16) -> ScreenExport {
    app.show_welcome_popup = false;
    app.transition_last_screen = Some(app.current_screen.clone());
    app.transition_effect = None;

    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| {
        matrix_iptv_lib::ui::ui(f, app);
    }).unwrap();

    let buffer = terminal.backend().buffer();
    let mut cells = Vec::new();

    for y in 0..height {
        for x in 0..width {
            let cell = buffer.cell((x, y)).unwrap();
            let fg = color_to_rgb(cell.fg, true);
            let bg = color_to_rgb(cell.bg, false);
            let bold = cell.modifier.contains(Modifier::BOLD);
            let dim = cell.modifier.contains(Modifier::DIM);
            let italic = cell.modifier.contains(Modifier::ITALIC);
            let underline = cell.modifier.contains(Modifier::UNDERLINED);

            cells.push(CellExport {
                x,
                y,
                char: cell.symbol().to_string(),
                fg,
                bg,
                bold,
                dim,
                italic,
                underline,
            });
        }
    }

    ScreenExport {
        name: name.to_string(),
        title: title.to_string(),
        description: description.to_string(),
        width,
        height,
        cells,
    }
}

fn setup_base_app() -> App {
    let mut app = App::new();
    app.show_welcome_popup = false;
    app.show_matrix_rain = false;
    
    // Load test accounts matching test_config.json
    app.config.accounts = vec![
        make_account("Strong 8K Live Server", "http://zfruvync.duperab.xyz", "PE1S9S8U", "••••••••", AccountType::Xtream, 18420, 34210, 8940),
        make_account("Trex Premium FHD", "http://line.offcial-trex.pro", "3a6aae52fb", "••••••••", AccountType::Xtream, 12500, 21000, 5400),
        make_account("Premium Mega 4K Cinema", "http://smarters.live:80", "ZPY7BP5A", "••••••••", AccountType::Xtream, 9800, 42000, 11200),
        make_account("Promax 4K OTT", "http://line.queen-4k.cc/", "11D246", "••••••••", AccountType::Xtream, 7200, 15000, 3100),
        make_account("Mega OTT Free M3U", "http://line.4smart.in", "45Z88W6", "••••••••", AccountType::Xtream, 5200, 8900, 1400),
    ];

    // Populate categories
    app.all_categories = vec![
        make_cat("1", "⭐ US: Premium Cinema & HBO", false),
        make_cat("2", "🏈 US: Live Sports & PPV", true),
        make_cat("3", "📰 US: National News & Info", false),
        make_cat("4", "🎬 US: Entertainment & Comedy", false),
        make_cat("5", "⚽ UK: Sky & TNT Sports", true),
        make_cat("6", "🍁 CA: Sports & National", true),
        make_cat("7", "🏎️ F1 & Motorsport 4K", true),
        make_cat("8", "🥊 UFC & Boxing PPV", true),
        make_cat("9", "🇫🇷 FR: France National", false),
        make_cat("10", "🇩🇪 DE: Sky Deutschland", true),
    ];
    app.categories = app.all_categories.clone();

    // Populate streams
    app.all_streams = vec![
        make_stream(101, "US: ESPN HD (60fps Live)"),
        make_stream(102, "US: ESPN2 HD (Sports Center)"),
        make_stream(103, "US: FOX Sports 1 4K UHD"),
        make_stream(104, "US: TNT Sports HD (NBA Live)"),
        make_stream(105, "US: NBC Sports 4K HDR"),
        make_stream(106, "US: NFL RedZone HD (Commercial Free)"),
        make_stream(107, "US: NBA TV HD (Live Studio)"),
        make_stream(108, "US: MLB Network HD (Ballpark Live)"),
        make_stream(109, "US: MSNBC HD (Live News Desk)"),
        make_stream(110, "US: CNN International HD"),
        make_stream(111, "US: HBO East 4K HDR"),
        make_stream(112, "US: Showtime East FHD"),
    ];
    app.streams = app.all_streams.clone();

    // Populate VOD
    app.all_vod_categories = vec![
        make_cat("v1", "🔥 4K UHD Ultra HDR Cinema", false),
        make_cat("v2", "💥 Action, Thriller & Heist", false),
        make_cat("v3", "🚀 Sci-Fi & Cyberpunk Classics", false),
        make_cat("v4", "🎭 Comedy & Standup Specials", false),
        make_cat("v5", "🏆 Award Winners & Oscar Nominees", false),
        make_cat("v6", "🕵️ Crime, Mystery & Noir", false),
    ];
    app.vod_categories = app.all_vod_categories.clone();

    app.all_vod_streams = vec![
        make_vod(201, "The Matrix Resurrections (2021) [4K HDR]", Some(8.4)),
        make_vod(202, "Dune: Part Two (2024) [4K IMAX HDR]", Some(8.9)),
        make_vod(203, "Oppenheimer (2023) [4K UHD Master]", Some(8.9)),
        make_vod(204, "Blade Runner 2049 (2017) [4K Atmos]", Some(8.7)),
        make_vod(205, "Interstellar (2014) [1080p BluRay]", Some(8.8)),
        make_vod(206, "The Dark Knight (2008) [4K Remaster]", Some(9.0)),
        make_vod(207, "Inception (2010) [4K UHD]", Some(8.8)),
        make_vod(208, "Cyberpunk: Edgerunners Movie (2024)", Some(8.6)),
    ];
    app.vod_streams = app.all_vod_streams.clone();

    // Populate Series
    app.all_series_categories = vec![
        make_cat("s1", "📺 HBO Originals & Prestige Drama", false),
        make_cat("s2", "🤖 Sci-Fi, Cyber & Dystopian", false),
        make_cat("s3", "🍎 Apple TV+ Originals 4K", false),
        make_cat("s4", "🍿 Netflix Binge Hits", false),
        make_cat("s5", "⭐ All-Time Classic Box Sets", false),
    ];
    app.series_categories = app.all_series_categories.clone();

    app.all_series_streams = vec![
        make_series(301, "Severance [Season 1-2 Complete 4K]"),
        make_series(302, "The Last of Us [Season 1-2 Complete]"),
        make_series(303, "House of the Dragon [Season 1-2 4K HDR]"),
        make_series(304, "Succession [Complete Series FHD]"),
        make_series(305, "Stranger Things [Season 1-5 4K]"),
        make_series(306, "Breaking Bad [Complete Remastered 4K]"),
        make_series(307, "Mr. Robot [Complete Series FHD]"),
        make_series(308, "Dark [Complete Sci-Fi Trilogy 4K]"),
    ];
    app.series_streams = app.all_series_streams.clone();

    // Populate Sports Dashboard
    app.sports_categories = vec![
        "live-now".to_string(),
        "nba-basketball".to_string(),
        "premier-league".to_string(),
        "champions-league".to_string(),
        "nfl-football".to_string(),
        "motorsport-f1".to_string(),
    ];

    app.sports_matches = vec![
        StreamedMatch {
            id: "match-1".to_string(),
            title: "NBA: Boston Celtics vs Los Angeles Lakers".to_string(),
            category: "basketball".to_string(),
            date: 1787522400000,
            teams: Some(StreamedTeams {
                home: Some(StreamedTeam { name: "Boston Celtics".to_string(), badge: None }),
                away: Some(StreamedTeam { name: "Los Angeles Lakers".to_string(), badge: None }),
            }),
            sources: vec![
                StreamedSource {
                    source: "StreamHD".to_string(),
                    id: "s1".to_string(),
                }
            ],
            popular: true,
        },
        StreamedMatch {
            id: "match-2".to_string(),
            title: "Premier League: Arsenal vs Manchester City".to_string(),
            category: "football".to_string(),
            date: 1787526000000,
            teams: Some(StreamedTeams {
                home: Some(StreamedTeam { name: "Arsenal".to_string(), badge: None }),
                away: Some(StreamedTeam { name: "Manchester City".to_string(), badge: None }),
            }),
            sources: vec![],
            popular: true,
        },
        StreamedMatch {
            id: "match-3".to_string(),
            title: "UEFA Champions League: Real Madrid vs Bayern Munich".to_string(),
            category: "football".to_string(),
            date: 1787530000000,
            teams: Some(StreamedTeams {
                home: Some(StreamedTeam { name: "Real Madrid".to_string(), badge: None }),
                away: Some(StreamedTeam { name: "Bayern Munich".to_string(), badge: None }),
            }),
            sources: vec![],
            popular: true,
        },
    ];

    app.current_sports_streams = vec![
        StreamedStream {
            id: "stream-1".to_string(),
            stream_no: 1,
            hd: true,
            embed_url: "http://stream.example.com/live/1".to_string(),
            source: "TNT Sports 4K".to_string(),
            language: "English".to_string(),
        },
        StreamedStream {
            id: "stream-2".to_string(),
            stream_no: 2,
            hd: true,
            embed_url: "http://stream.example.com/live/2".to_string(),
            source: "ESPN Extra HD".to_string(),
            language: "English".to_string(),
        },
    ];

    // Populate live scores
    app.live_scores = vec![
        matrix_iptv_lib::scores::ScoreGame {
            id: "game-1".to_string(),
            league: "NBA".to_string(),
            start_time: "2026-08-23T20:00:00Z".to_string(),
            status_state: "in".to_string(),
            status_detail: "4th Qtr".to_string(),
            home_team: "Boston Celtics".to_string(),
            home_score: "108".to_string(),
            home_abbr: "BOS".to_string(),
            home_color: None,
            home_record: None,
            home_logo: None,
            away_team: "Los Angeles Lakers".to_string(),
            away_score: "102".to_string(),
            away_abbr: "LAL".to_string(),
            away_color: None,
            away_record: None,
            away_logo: None,
            display_clock: "3:42".to_string(),
            period: 4,
            venue_name: None,
            venue_city: None,
            venue_state: None,
            broadcasts: Vec::new(),
            last_play: None,
            home_win_pct: None,
            away_win_pct: None,
            headline: None,
            series_summary: None,
            top_scorer: None,
        }
    ];

    app.global_all_streams = app.all_streams.clone();
    app.global_all_vod_streams = app.all_vod_streams.clone();
    app.global_all_series_streams = app.all_series_streams.clone();

    app
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let width = 140;
    let height = 40;
    let mut screens = Vec::new();

    // ─── 1. Home Screen (Playlist Selector) ───
    {
        let mut app = setup_base_app();
        app.current_screen = CurrentScreen::Home;
        app.account_list_state.select(Some(0));
        screens.push(render_screen(&mut app, "01_home_screen", "Home Screen (Playlist Selector)", "Entry hub showing configured IPTV provider playlists with OMP cursor selection and Matrix border.", width, height));
    }

    // ─── 2. Content Type Selection Hub ───
    {
        let mut app = setup_base_app();
        app.current_screen = CurrentScreen::ContentTypeSelection;
        app.selected_content_type_index = 0; // Live TV
        screens.push(render_screen(&mut app, "02_content_type_selection", "Content Hub (Type Selector)", "Navigation hub for selecting Live Channels, Movies & VOD, or TV Series.", width, height));
    }

    // ─── 3. Live TV Channels (Dual Pane Guide with Metadata & Health) ───
    {
        let mut app = setup_base_app();
        app.current_screen = CurrentScreen::Streams;
        app.active_pane = Pane::Streams;
        app.category_list_state.select(Some(1)); // US: Live Sports & PPV
        app.stream_list_state.select(Some(2)); // FOX Sports 1 4K
        screens.push(render_screen(&mut app, "03_live_tv_channels", "Live TV Guide (Dual-Pane)", "High density split view showing channels with quality tags, latency health bars, video profile, and EPG live progress.", width, height));
    }

    // ─── 4. Movies & VOD Cinema ───
    {
        let mut app = setup_base_app();
        app.current_screen = CurrentScreen::VodStreams;
        app.active_pane = Pane::Streams;
        app.vod_category_list_state.select(Some(0)); // 4K UHD Cinema
        app.vod_stream_list_state.select(Some(1)); // Dune: Part Two
        screens.push(render_screen(&mut app, "04_movies_vod_cinema", "Movies & VOD Cinema", "Video on demand browser with resolution badges, ratings, audio format, and release year metadata.", width, height));
    }

    // ─── 5. TV Series & Box Sets ───
    {
        let mut app = setup_base_app();
        app.current_screen = CurrentScreen::SeriesStreams;
        app.active_pane = Pane::Streams;
        app.series_category_list_state.select(Some(0)); // HBO Originals
        app.series_stream_list_state.select(Some(0)); // Severance
        screens.push(render_screen(&mut app, "05_tv_series_shows", "TV Series & Box Sets", "Multi-season episodic browser with series categories, season packaging, and episode indices.", width, height));
    }

    // ─── 6. Live Sports Dashboard ───
    {
        let mut app = setup_base_app();
        app.current_screen = CurrentScreen::SportsDashboard;
        app.active_pane = Pane::Streams;
        app.sports_category_list_state.select(Some(0)); // Live Now
        app.sports_list_state.select(Some(0)); // NBA Celtics vs Lakers
        screens.push(render_screen(&mut app, "06_sports_dashboard", "Live Sports Match Center", "Live fixture tracker with team colors, real-time scoreboards, match clocks, and active streams.", width, height));
    }

    // ─── 7. Global Search (Ctrl+Space) ───
    {
        let mut app = setup_base_app();
        app.current_screen = CurrentScreen::GlobalSearch;
        app.search_state.query = "matrix".to_string();
        app.update_search();
        app.global_search_list_state.select(Some(0));
        screens.push(render_screen(&mut app, "07_global_search", "Global Omni-Search", "Instant cross-content search across all live channels, on-demand movies, and TV series.", width, height));
    }

    // ─── 8. Settings Screen ───
    {
        let mut app = setup_base_app();
        app.current_screen = CurrentScreen::Settings;
        app.settings_list_state.select(Some(2)); // Playlist Mode
        app.refresh_settings_options();
        screens.push(render_screen(&mut app, "08_settings_screen", "Application Settings (OMP)", "Full configuration suite for Timezone, Video Player (MPV/VLC), DNS, Filters, and Diagnostics.", width, height));
    }

    // ─── 8b. Category Management & Country Filter ───
    {
        let mut app = setup_base_app();
        app.current_screen = CurrentScreen::Settings;
        app.settings_state = SettingsState::CategoryManagement;
        app.category_mgmt.search_query = "US".to_string();
        app.category_mgmt.search_mode = false;
        app.category_mgmt.list_state.select(Some(0));
        screens.push(render_screen(&mut app, "08b_category_management_country_filter", "Category Manager & Country Filter", "Fast country filter (e.g. 'US') with [o] country isolate, [space] toggle, [a] toggle all, and [c] show all.", width, height));
    }

    // ─── 9. Add / Edit Playlist Form ───
    {
        let mut app = setup_base_app();
        app.current_screen = CurrentScreen::Login;
        app.login_field_focus = LoginField::Name;
        app.input_mode = InputMode::Editing;
        app.input_name = tui_input::Input::new("Strong 8K Live Server".to_string());
        app.input_url = tui_input::Input::new("http://zfruvync.duperab.xyz".to_string());
        app.input_username = tui_input::Input::new("PE1S9S8U".to_string());
        app.input_password = tui_input::Input::new("11EZZUMW".to_string());
        app.input_epg_url = tui_input::Input::new("http://zfruvync.duperab.xyz/xmltv.php".to_string());
        screens.push(render_screen(&mut app, "09_playlist_login_form", "Playlist Connection Form", "Terminal form editor with active green cursor for entering Xtream Codes or M3U credentials.", width, height));
    }

    // ─── 10. Matrix Digital Rain Screensaver ───
    {
        let mut app = setup_base_app();
        app.current_screen = CurrentScreen::Home;
        app.show_matrix_rain = true;
        screens.push(render_screen(&mut app, "10_matrix_digital_rain", "Matrix Digital Rain Screensaver", "Iconic phosphor green streaming Matrix glyph rain with Getting Started quickstart card.", width, height));
    }

    // ─── 11. Help & Keybindings Modal ───
    {
        let mut app = setup_base_app();
        app.current_screen = CurrentScreen::Streams;
        app.active_pane = Pane::Streams;
        app.show_help = true;
        screens.push(render_screen(&mut app, "11_help_keybindings_modal", "Help & Keybindings Modal", "Pop-up cheat sheet showing vim navigation keys, player shortcuts, search hotkeys, and tips.", width, height));
    }

    let json = serde_json::to_string_pretty(&screens)?;
    let mut file = File::create("screens_dump.json")?;
    file.write_all(json.as_bytes())?;

    println!("Successfully rendered and exported {} screens to screens_dump.json", screens.len());
    Ok(())
}
