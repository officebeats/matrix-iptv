use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use matrix_iptv_lib::api::{Category, SeriesEpisode, Stream};
use matrix_iptv_lib::app::{App, CurrentScreen, Pane};
use matrix_iptv_lib::flex_id::FlexId;
use matrix_iptv_lib::handlers::input::handle_key_event;
use matrix_iptv_lib::player::Player;
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use std::sync::Arc;
use std::time::Instant;

// ─── Helpers ───────────────────────────────────────────────────────────────────

fn make_stream(id: u64, name: &str) -> Arc<Stream> {
    Arc::new(Stream {
        name: name.to_string(),
        stream_id: FlexId::Number(id as i64),
        search_name: name.to_lowercase(),
        ..Default::default()
    })
}

fn make_vod_stream(id: u64, name: &str) -> Arc<Stream> {
    Arc::new(Stream {
        name: name.to_string(),
        stream_id: FlexId::Number(id as i64),
        stream_type: "movie".to_string(),
        search_name: name.to_lowercase(),
        is_english: true,
        ..Default::default()
    })
}

fn make_series_stream(id: u64, name: &str) -> Arc<Stream> {
    Arc::new(Stream {
        name: name.to_string(),
        stream_id: FlexId::Number(id as i64),
        stream_type: "series".to_string(),
        search_name: name.to_lowercase(),
        is_english: true,
        ..Default::default()
    })
}

fn make_episode(id: i64, title: &str) -> SeriesEpisode {
    SeriesEpisode {
        id: Some(FlexId::Number(id)),
        episode_num: id as i32,
        title: Some(title.to_string()),
        container_extension: Some("mp4".to_string()),
        info: None,
        season: 1,
        direct_source: String::new(),
    }
}

fn make_category(id: &str, name: &str) -> Arc<Category> {
    Arc::new(Category {
        category_id: id.to_string(),
        category_name: name.to_string(),
        search_name: name.to_lowercase(),
        parent_id: FlexId::Number(0),
        ..Default::default()
    })
}

async fn press_runtime_key(app: &mut App, code: KeyCode) {
    let (tx, _rx) = tokio::sync::mpsc::channel(1);
    let player = Player::new();
    let key = KeyEvent {
        code,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    };

    handle_key_event(app, key, &tx, &player).await.unwrap();
}

/// Render one frame of the UI — panics on crash
fn render_frame(app: &mut App) {
    app.show_matrix_rain = false;
    app.transition_last_screen = Some(app.current_screen.clone());
    app.transition_effect = None;
    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| {
            matrix_iptv_lib::ui::ui(f, app);
        })
        .unwrap();
}

fn render_frame_text(app: &mut App) -> String {
    app.show_matrix_rain = false;
    app.transition_last_screen = Some(app.current_screen.clone());
    app.transition_effect = None;
    let backend = TestBackend::new(140, 40);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| {
            matrix_iptv_lib::ui::ui(f, app);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    let mut text = String::new();
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            text.push_str(buffer[(x, y)].symbol());
        }
        text.push('\n');
    }
    text
}

/// Generate N streams with unique names for strong8k-scale testing
fn generate_streams(count: usize) -> Vec<Arc<Stream>> {
    (0..count)
        .map(|i| {
            let name = format!("US | Channel {} HD", i);
            Arc::new(Stream {
                name: name.clone(),
                stream_id: FlexId::Number(i as i64),
                search_name: name.to_lowercase(),
                is_american: true,
                ..Default::default()
            })
        })
        .collect()
}

fn generate_categories(count: usize) -> Vec<Arc<Category>> {
    (0..count)
        .map(|i| {
            let name = format!("Category {}", i);
            make_category(&i.to_string(), &name)
        })
        .collect()
}

// ─── Test 1: All Screens Render Without Panic (Empty State) ────────────────────

#[test]
fn test_all_screens_render_empty_state() {
    let screens = vec![
        CurrentScreen::Home,
        CurrentScreen::Login,
        CurrentScreen::Categories,
        CurrentScreen::Streams,
        CurrentScreen::VodCategories,
        CurrentScreen::VodStreams,
        CurrentScreen::SeriesCategories,
        CurrentScreen::SeriesStreams,
        CurrentScreen::Settings,
        CurrentScreen::TimezoneSettings,
        CurrentScreen::Play,
        CurrentScreen::ContentTypeSelection,
        CurrentScreen::GlobalSearch,
        CurrentScreen::GroupManagement,
        CurrentScreen::GroupPicker,
        CurrentScreen::UpdatePrompt,
        CurrentScreen::SportsDashboard,
    ];

    for screen in screens {
        let mut app = App::new();
        app.current_screen = screen.clone();
        render_frame(&mut app);
        // If we get here without panic, the screen rendered OK
    }
}

#[test]
fn test_footer_hints_match_vod_and_series_stream_hotkeys() {
    let mut app = App::new();
    app.current_screen = CurrentScreen::Categories;
    app.active_pane = Pane::Categories;
    app.categories = generate_categories(2);
    let live_text = render_frame_text(&mut app);
    assert!(!live_text.contains("PgDn"));

    let mut app = App::new();
    app.current_screen = CurrentScreen::VodStreams;
    app.active_pane = Pane::Streams;
    app.streams = generate_streams(2);
    let vod_text = render_frame_text(&mut app);
    assert!(vod_text.contains("g top"));
    assert!(vod_text.contains("G bottom"));
    assert!(!vod_text.contains("PgDn"));
    assert!(!vod_text.contains("g add group"));
    assert!(!vod_text.contains("v fav"));
    assert!(!vod_text.contains("i info"));

    let mut app = App::new();
    app.current_screen = CurrentScreen::SeriesStreams;
    app.active_pane = Pane::Streams;
    app.series_streams = vec![make_series_stream(1, "US | Example Show")];
    let series_text = render_frame_text(&mut app);
    assert!(series_text.contains("enter episodes"));
    assert!(series_text.contains("Home top"));
    assert!(series_text.contains("End bottom"));
    assert!(!series_text.contains("PgDn"));
    assert!(!series_text.contains("g add group"));
    assert!(!series_text.contains("v fav"));
    assert!(!series_text.contains("i info"));
}

#[tokio::test]
async fn test_runtime_navigation_matches_stream_footer_hints() {
    let mut app = App::new();
    app.current_screen = CurrentScreen::VodStreams;
    app.active_pane = Pane::Streams;
    app.vod_streams = vec![
        make_vod_stream(1, "Movie 1"),
        make_vod_stream(2, "Movie 2"),
        make_vod_stream(3, "Movie 3"),
    ];
    app.selected_vod_stream_index = 1;
    app.vod_stream_list_state.select(Some(1));

    press_runtime_key(&mut app, KeyCode::Char('g')).await;
    assert_eq!(app.selected_vod_stream_index, 0, "VOD g should jump to top");

    press_runtime_key(&mut app, KeyCode::Char('G')).await;
    assert_eq!(
        app.selected_vod_stream_index, 2,
        "VOD G should jump to bottom"
    );

    let mut app = App::new();
    app.current_screen = CurrentScreen::SeriesCategories;
    app.series_categories = generate_categories(3);
    app.selected_series_category_index = 0;
    app.series_category_list_state.select(Some(0));

    press_runtime_key(&mut app, KeyCode::Char('G')).await;
    assert_eq!(
        app.selected_series_category_index, 2,
        "Series category G should jump to bottom"
    );

    let mut app = App::new();
    app.current_screen = CurrentScreen::SeriesStreams;
    app.active_pane = Pane::Streams;
    app.series_streams = vec![
        make_series_stream(1, "Show 1"),
        make_series_stream(2, "Show 2"),
        make_series_stream(3, "Show 3"),
    ];
    app.selected_series_stream_index = 1;
    app.series_stream_list_state.select(Some(1));

    press_runtime_key(&mut app, KeyCode::Home).await;
    assert_eq!(app.selected_series_stream_index, 0);

    press_runtime_key(&mut app, KeyCode::End).await;
    assert_eq!(app.selected_series_stream_index, 2);

    app.active_pane = Pane::Episodes;
    app.series_episodes = vec![
        make_episode(1, "Episode 1"),
        make_episode(2, "Episode 2"),
        make_episode(3, "Episode 3"),
    ];
    app.selected_series_episode_index = 1;
    app.series_episode_list_state.select(Some(1));

    press_runtime_key(&mut app, KeyCode::Home).await;
    assert_eq!(app.selected_series_episode_index, 0);

    press_runtime_key(&mut app, KeyCode::End).await;
    assert_eq!(app.selected_series_episode_index, 2);
}

#[test]
fn test_help_popup_does_not_advertise_stale_hotkeys() {
    let mut app = App::new();
    app.show_help = true;
    let text = render_frame_text(&mut app);
    assert!(!text.contains("g/G         jump to top / bottom"));
    assert!(!text.contains("toggle filter active/all"));
    assert!(!text.contains("PgDn/PgUp"));
    assert!(!text.contains("ctrl+d/u"));
    assert!(text.contains("/ or f"));
    assert!(text.contains("category grid or live add-to-group"));
}

// ─── Test 2: Large Stream List (2000 items — strong8k scale) ───────────────────

#[test]
fn test_large_stream_list_rendering() {
    let mut app = App::new();
    let streams = generate_streams(2000);
    app.all_streams = streams;
    app.current_screen = CurrentScreen::Categories;
    app.active_pane = Pane::Streams;
    app.update_search();

    // Verify: streams populated (capped at 1000 by update_search)
    assert_eq!(
        app.streams.len(),
        1000,
        "Streams should be capped at 1000 by update_search"
    );

    // Verify: cached_parsed is populated on all 1000
    for (i, s) in app.streams.iter().enumerate() {
        assert!(
            s.cached_parsed.is_some(),
            "Stream {} should have cached_parsed populated",
            i
        );
    }

    // Render — must not panic
    render_frame(&mut app);
}

// ─── Test 3: Large Category List (200 items) ───────────────────────────────────

#[test]
fn test_large_category_list_rendering() {
    let mut app = App::new();
    app.all_categories = generate_categories(200);
    app.current_screen = CurrentScreen::Categories;
    app.active_pane = Pane::Categories;
    app.update_search();

    assert_eq!(app.categories.len(), 200);

    // Render
    render_frame(&mut app);
}

// ─── Test 4: Search Filtering Correctness (Live TV) ────────────────────────────

#[test]
fn test_search_filtering_live_tv() {
    let mut app = App::new();
    app.all_streams = vec![
        make_stream(1, "ESPN HD"),
        make_stream(2, "CNN International"),
        make_stream(3, "BBC World News"),
        make_stream(4, "ESPN2 HD"),
        make_stream(5, "Fox Sports 1"),
    ];
    app.current_screen = CurrentScreen::Categories;
    app.active_pane = Pane::Streams;

    // Empty search → all streams
    app.search_state.query.clear();
    app.update_search();
    assert_eq!(
        app.streams.len(),
        5,
        "Empty search should return all streams"
    );

    // Exact substring search
    app.search_state.query = "espn".to_string();
    app.update_search();
    assert!(
        app.streams.len() >= 2,
        "Searching 'espn' should match ESPN HD and ESPN2 HD"
    );

    // No match search
    app.search_state.query = "xyznonexistent".to_string();
    app.update_search();
    assert_eq!(
        app.streams.len(),
        0,
        "Nonsense query should return 0 results"
    );

    // Clear search restores all
    app.search_state.query.clear();
    app.update_search();
    assert_eq!(
        app.streams.len(),
        5,
        "Clearing search should restore all streams"
    );
}

// ─── Test 5: Search Filtering Correctness (VOD) ───────────────────────────────

#[test]
fn test_search_filtering_vod() {
    let mut app = App::new();
    app.all_vod_streams = vec![
        make_vod_stream(1, "The Matrix (1999)"),
        make_vod_stream(2, "Inception (2010)"),
        make_vod_stream(3, "Interstellar (2014)"),
    ];
    app.current_screen = CurrentScreen::VodCategories;
    app.active_pane = Pane::Streams;

    app.search_state.query = "matrix".to_string();
    app.update_search();
    assert_eq!(
        app.vod_streams.len(),
        1,
        "VOD search for 'matrix' should return 1"
    );
    assert!(
        app.vod_streams[0].cached_parsed.is_some(),
        "VOD stream should have cached parse"
    );

    app.search_state.query.clear();
    app.update_search();
    assert_eq!(
        app.vod_streams.len(),
        3,
        "Clearing search should restore all VOD streams"
    );
}

// ─── Test 6: Search Filtering Correctness (Series) ────────────────────────────

#[test]
fn test_search_filtering_series() {
    let mut app = App::new();
    app.all_series_streams = vec![
        make_series_stream(1, "Breaking Bad"),
        make_series_stream(2, "Game of Thrones"),
        make_series_stream(3, "The Wire"),
    ];
    app.current_screen = CurrentScreen::SeriesCategories;
    app.active_pane = Pane::Streams;

    app.search_state.query = "wire".to_string();
    app.update_search();
    assert_eq!(
        app.series_streams.len(),
        1,
        "Series search for 'wire' should return 1"
    );

    app.search_state.query.clear();
    app.update_search();
    assert_eq!(
        app.series_streams.len(),
        3,
        "Clearing search should restore all series"
    );
}

// ─── Test 7: Navigation Boundary Checks ────────────────────────────────────────

#[test]
fn test_navigation_boundaries() {
    let mut app = App::new();
    app.all_streams = vec![
        make_stream(1, "Stream 1"),
        make_stream(2, "Stream 2"),
        make_stream(3, "Stream 3"),
    ];
    app.current_screen = CurrentScreen::Categories;
    app.active_pane = Pane::Streams;
    app.update_search();

    assert_eq!(app.streams.len(), 3);
    assert_eq!(app.selected_stream_index, 0);

    // Navigate down
    app.next_stream();
    assert_eq!(app.selected_stream_index, 1);

    app.next_stream();
    assert_eq!(app.selected_stream_index, 2);

    // At boundary — wraps to 0
    app.next_stream();
    assert_eq!(app.selected_stream_index, 0, "Should wrap around to 0");

    // Navigate up from 0 — wraps to last
    app.previous_stream();
    assert_eq!(app.selected_stream_index, 2, "Should wrap to last item");
}

// ─── Test 8: VOD/Series Screen Rendering ──────────────────────────────────────

#[test]
fn test_vod_series_screen_rendering() {
    let mut app = App::new();

    // VOD
    app.all_vod_categories = vec![make_category("1", "Action Movies")];
    app.all_vod_streams = vec![
        make_vod_stream(1, "The Matrix (1999)"),
        make_vod_stream(2, "Inception (2010)"),
    ];
    app.current_screen = CurrentScreen::VodCategories;
    app.active_pane = Pane::Categories;
    app.update_search();
    render_frame(&mut app);

    app.active_pane = Pane::Streams;
    app.update_search();
    render_frame(&mut app);

    // Series
    app.all_series_categories = vec![make_category("10", "Drama Series")];
    app.all_series_streams = vec![
        make_series_stream(1, "Breaking Bad"),
        make_series_stream(2, "Better Call Saul"),
    ];
    app.current_screen = CurrentScreen::SeriesCategories;
    app.active_pane = Pane::Categories;
    app.update_search();
    render_frame(&mut app);

    app.active_pane = Pane::Streams;
    app.update_search();
    render_frame(&mut app);
}

// ─── Test 9: Settings Screen Rendering ─────────────────────────────────────────

#[test]
fn test_settings_screen_rendering() {
    let mut app = App::new();
    app.current_screen = CurrentScreen::Settings;
    render_frame(&mut app);

    app.current_screen = CurrentScreen::TimezoneSettings;
    render_frame(&mut app);
}

// ─── Test 10: Global Search Rendering ──────────────────────────────────────────

#[test]
fn test_global_search_rendering() {
    let mut app = App::new();

    // Populate all content types into global pools
    app.global_all_streams = vec![make_stream(1, "ESPN HD"), make_stream(2, "CNN")];
    app.global_all_vod_streams = vec![make_vod_stream(10, "The Matrix (1999)")];
    app.global_all_series_streams = vec![make_series_stream(20, "Breaking Bad")];

    app.current_screen = CurrentScreen::GlobalSearch;

    // Empty search → no results
    app.search_state.query.clear();
    app.update_search();
    assert_eq!(
        app.global_search_results.len(),
        0,
        "Empty global search should show no results"
    );
    render_frame(&mut app);

    // Search across all types
    app.search_state.query = "the".to_string();
    app.update_search();
    assert!(
        !app.global_search_results.is_empty(),
        "Global search for 'the' should match at least The Matrix"
    );
    // Verify caching
    for s in &app.global_search_results {
        assert!(
            s.cached_parsed.is_some(),
            "Global search results should have cached_parsed"
        );
    }
    render_frame(&mut app);
}

// ─── Test 11: Cache Performance — Parse Once Not Per Frame ─────────────────────

#[test]
fn test_cache_prevents_redundant_parsing() {
    let mut app = App::new();
    app.all_streams = generate_streams(100);
    app.current_screen = CurrentScreen::Categories;
    app.active_pane = Pane::Streams;

    let start = Instant::now();
    app.update_search();
    let first_update = start.elapsed();

    // All cached
    for s in &app.streams {
        assert!(s.cached_parsed.is_some());
    }

    // Second call should be near-instant (cache hit) since all are already cached
    let start2 = Instant::now();
    app.update_search();
    let second_update = start2.elapsed();

    // The second call re-creates the Vec but Arc::make_mut won't clone since
    // the Arcs are uniquely held by self.streams after the first call
    // Release mode: strict 3-second limit (user requirement)
    // Debug mode: loose limit since unoptimized builds are ~10x slower
    #[cfg(not(debug_assertions))]
    {
        assert!(
            first_update.as_millis() < 3000,
            "First update_search with 100 streams must complete under 3s, took {}ms",
            first_update.as_millis()
        );
        assert!(
            second_update.as_millis() < 3000,
            "Second update_search must complete under 3s, took {}ms",
            second_update.as_millis()
        );
    }
    #[cfg(debug_assertions)]
    {
        assert!(
            first_update.as_millis() < 30000,
            "First update_search with 100 streams should complete under 30s in debug, took {}ms",
            first_update.as_millis()
        );
        assert!(
            second_update.as_millis() < 30000,
            "Second update_search should complete under 30s in debug, took {}ms",
            second_update.as_millis()
        );
    }
}

// ─── Test 12: Render Performance With Cached Data ──────────────────────────────

#[test]
fn test_render_performance_with_cache() {
    let mut app = App::new();
    app.all_streams = generate_streams(1000);
    app.all_categories = generate_categories(100);
    app.current_screen = CurrentScreen::Categories;
    app.active_pane = Pane::Streams;
    app.update_search();

    // Render multiple frames — should be fast since parse is cached
    let start = Instant::now();
    for _ in 0..10 {
        render_frame(&mut app);
    }
    let render_time = start.elapsed();

    // Release mode: 10 frames must complete well under 3s (typically <500ms)
    // Debug mode: loose threshold
    #[cfg(not(debug_assertions))]
    assert!(
        render_time.as_millis() < 3000,
        "10 render frames must complete under 3s, took {}ms",
        render_time.as_millis()
    );
    #[cfg(debug_assertions)]
    assert!(
        render_time.as_millis() < 60000,
        "10 render frames should complete under 60s in debug, took {}ms",
        render_time.as_millis()
    );
}

// ─── Test 13: Sports Dashboard Empty State ─────────────────────────────────────

#[test]
fn test_sports_dashboard_empty() {
    let mut app = App::new();
    app.current_screen = CurrentScreen::SportsDashboard;
    render_frame(&mut app);
}

// ─── Test 14: Cross-Pane Search (Categories view searching streams) ────────────

#[test]
fn test_cross_pane_search() {
    let mut app = App::new();
    app.all_categories = vec![
        make_category("1", "US Sports"),
        make_category("2", "UK News"),
    ];
    app.global_all_streams = vec![make_stream(1, "ESPN HD"), make_stream(2, "BBC World")];
    app.current_screen = CurrentScreen::Categories;
    app.active_pane = Pane::Categories;

    // Search for a stream name while in categories pane
    app.search_state.query = "espn".to_string();
    app.update_search();

    // Categories should be filtered
    // Streams should also be searched cross-pane
    assert!(
        !app.streams.is_empty(),
        "Cross-pane search should find ESPN in streams"
    );
}
