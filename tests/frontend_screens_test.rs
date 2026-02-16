use matrix_iptv_lib::api::{Category, Stream};
use matrix_iptv_lib::flex_id::FlexId;
use matrix_iptv_lib::app::{App, CurrentScreen, Pane};
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

fn make_category(id: &str, name: &str) -> Arc<Category> {
    Arc::new(Category {
        category_id: id.to_string(),
        category_name: name.to_string(),
        search_name: name.to_lowercase(),
        parent_id: FlexId::Number(0),
        ..Default::default()
    })
}

/// Render one frame of the UI — panics on crash
fn render_frame(app: &mut App) {
    let backend = TestBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| {
            matrix_iptv_lib::ui::ui(f, app);
        })
        .unwrap();
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
    assert_eq!(app.streams.len(), 5, "Empty search should return all streams");

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
    assert_eq!(app.vod_streams.len(), 1, "VOD search for 'matrix' should return 1");
    assert!(app.vod_streams[0].cached_parsed.is_some(), "VOD stream should have cached parse");

    app.search_state.query.clear();
    app.update_search();
    assert_eq!(app.vod_streams.len(), 3, "Clearing search should restore all VOD streams");
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
    assert_eq!(app.series_streams.len(), 1, "Series search for 'wire' should return 1");

    app.search_state.query.clear();
    app.update_search();
    assert_eq!(app.series_streams.len(), 3, "Clearing search should restore all series");
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
    app.global_all_streams = vec![
        make_stream(1, "ESPN HD"),
        make_stream(2, "CNN"),
    ];
    app.global_all_vod_streams = vec![
        make_vod_stream(10, "The Matrix (1999)"),
    ];
    app.global_all_series_streams = vec![
        make_series_stream(20, "Breaking Bad"),
    ];

    app.current_screen = CurrentScreen::GlobalSearch;

    // Empty search → no results
    app.search_state.query.clear();
    app.update_search();
    assert_eq!(app.global_search_results.len(), 0, "Empty global search should show no results");
    render_frame(&mut app);

    // Search across all types
    app.search_state.query = "the".to_string();
    app.update_search();
    assert!(
        app.global_search_results.len() >= 1,
        "Global search for 'the' should match at least The Matrix"
    );
    // Verify caching
    for s in &app.global_search_results {
        assert!(s.cached_parsed.is_some(), "Global search results should have cached_parsed");
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
    app.global_all_streams = vec![
        make_stream(1, "ESPN HD"),
        make_stream(2, "BBC World"),
    ];
    app.current_screen = CurrentScreen::Categories;
    app.active_pane = Pane::Categories;

    // Search for a stream name while in categories pane
    app.search_state.query = "espn".to_string();
    app.update_search();

    // Categories should be filtered
    // Streams should also be searched cross-pane
    assert!(
        app.streams.len() >= 1,
        "Cross-pane search should find ESPN in streams"
    );
}
