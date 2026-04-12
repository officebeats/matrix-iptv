use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use matrix_iptv_lib::api::{Category, Stream};
use matrix_iptv_lib::app::{App, CurrentScreen, Pane};
use matrix_iptv_lib::flex_id::FlexId;
use std::sync::Arc;

fn make_key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
        state: KeyEventState::empty(),
    }
}

fn make_key_with_mod(code: KeyCode, mods: KeyModifiers) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: mods,
        kind: KeyEventKind::Press,
        state: KeyEventState::empty(),
    }
}

fn make_stream(id: u64, name: &str) -> Arc<Stream> {
    Arc::new(Stream {
        name: name.to_string(),
        stream_id: FlexId::Number(id as i64),
        search_name: name.to_lowercase(),
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

fn ensure_areas(app: &mut App) {
    if app.area_streams.height == 0 {
        app.area_streams = ratatui::layout::Rect::new(30, 2, 90, 34);
    }
    if app.area_categories.height == 0 {
        app.area_categories = ratatui::layout::Rect::new(0, 2, 30, 34);
    }
    if app.area_episodes.height == 0 {
        app.area_episodes = ratatui::layout::Rect::new(90, 2, 30, 34);
    }
}

async fn handle_key(app: &mut App, key: KeyEvent) {
    let (tx, _rx) = tokio::sync::mpsc::channel(32);
    let player = matrix_iptv_lib::player::Player::new();
    let _ = matrix_iptv_lib::handlers::input::handle_key_event(app, key, &tx, &player).await;
}

fn setup_live_screen(app: &mut App, stream_count: usize) {
    app.current_screen = CurrentScreen::Streams;
    app.active_pane = Pane::Streams;
    for i in 0..stream_count {
        app.streams
            .push(make_stream(i as u64, &format!("CH {}", i)));
    }
    app.all_streams = app.streams.clone();
    app.selected_stream_index = 0;
    app.stream_list_state.select(Some(0));

    app.categories.push(make_category("ALL", "All"));
    app.selected_category_index = 0;
    app.category_list_state.select(Some(0));
    ensure_areas(app);
}

fn setup_vod_screen(app: &mut App, stream_count: usize) {
    app.current_screen = CurrentScreen::VodStreams;
    app.active_pane = Pane::Streams;
    for i in 0..stream_count {
        app.vod_streams.push(Arc::new(Stream {
            name: format!("Movie {}", i),
            stream_id: FlexId::Number(i as i64),
            stream_type: "movie".to_string(),
            search_name: format!("movie {}", i),
            ..Default::default()
        }));
    }
    app.all_vod_streams = app.vod_streams.clone();
    app.selected_vod_stream_index = 0;
    app.vod_stream_list_state.select(Some(0));
    ensure_areas(app);
}

fn setup_series_screen(app: &mut App, stream_count: usize) {
    app.current_screen = CurrentScreen::SeriesStreams;
    app.active_pane = Pane::Streams;
    for i in 0..stream_count {
        app.series_streams.push(Arc::new(Stream {
            name: format!("Series {}", i),
            stream_id: FlexId::Number(i as i64),
            stream_type: "series".to_string(),
            search_name: format!("series {}", i),
            ..Default::default()
        }));
    }
    app.all_series_streams = app.series_streams.clone();
    app.selected_series_stream_index = 0;
    app.series_stream_list_state.select(Some(0));
    ensure_areas(app);
}

fn setup_global_search(app: &mut App, result_count: usize) {
    app.current_screen = CurrentScreen::GlobalSearch;
    for i in 0..result_count {
        app.global_search_results
            .push(make_stream(i as u64, &format!("Result {}", i)));
    }
    app.selected_stream_index = 0;
    app.global_search_list_state.select(Some(0));
    app.search_mode = false;
    ensure_areas(app);
}

#[tokio::test]
async fn test_pageup_pagedown_live_streams() {
    let mut app = App::new();
    setup_live_screen(&mut app, 100);

    handle_key(&mut app, make_key(KeyCode::PageDown)).await;
    assert!(
        app.selected_stream_index > 1,
        "PageDown should advance selection significantly"
    );

    let after_page_down = app.selected_stream_index;
    handle_key(&mut app, make_key(KeyCode::PageUp)).await;
    assert!(
        app.selected_stream_index < after_page_down,
        "PageUp should move selection backward"
    );
}

#[tokio::test]
async fn test_home_end_live_streams() {
    let mut app = App::new();
    setup_live_screen(&mut app, 100);

    handle_key(&mut app, make_key(KeyCode::End)).await;
    assert_eq!(
        app.selected_stream_index, 99,
        "End should jump to last item"
    );

    handle_key(&mut app, make_key(KeyCode::Home)).await;
    assert_eq!(
        app.selected_stream_index, 0,
        "Home should jump to first item"
    );
}

#[tokio::test]
async fn test_ctrl_d_ctrl_u_live_streams() {
    let mut app = App::new();
    setup_live_screen(&mut app, 100);

    handle_key(
        &mut app,
        make_key_with_mod(KeyCode::Char('d'), KeyModifiers::CONTROL),
    )
    .await;
    let after_ctrl_d = app.selected_stream_index;
    assert!(after_ctrl_d > 0, "Ctrl+D should move down by half page");

    handle_key(
        &mut app,
        make_key_with_mod(KeyCode::Char('u'), KeyModifiers::CONTROL),
    )
    .await;
    assert_eq!(
        app.selected_stream_index, 0,
        "Ctrl+U from position 0 should stay at 0"
    );

    handle_key(
        &mut app,
        make_key_with_mod(KeyCode::Char('d'), KeyModifiers::CONTROL),
    )
    .await;
    let pos1 = app.selected_stream_index;
    handle_key(
        &mut app,
        make_key_with_mod(KeyCode::Char('d'), KeyModifiers::CONTROL),
    )
    .await;
    let pos2 = app.selected_stream_index;
    assert!(pos2 > pos1, "Second Ctrl+D should advance further");

    handle_key(
        &mut app,
        make_key_with_mod(KeyCode::Char('u'), KeyModifiers::CONTROL),
    )
    .await;
    assert!(
        app.selected_stream_index < pos2,
        "Ctrl+U should move back up"
    );
}

#[tokio::test]
async fn test_pageup_pagedown_vod_streams() {
    let mut app = App::new();
    setup_vod_screen(&mut app, 80);

    handle_key(&mut app, make_key(KeyCode::PageDown)).await;
    assert!(
        app.selected_vod_stream_index > 1,
        "PageDown on VOD should advance"
    );

    handle_key(&mut app, make_key(KeyCode::End)).await;
    assert_eq!(
        app.selected_vod_stream_index, 79,
        "End on VOD should jump to last"
    );

    handle_key(&mut app, make_key(KeyCode::Home)).await;
    assert_eq!(
        app.selected_vod_stream_index, 0,
        "Home on VOD should jump to first"
    );
}

#[tokio::test]
async fn test_pageup_pagedown_series_streams() {
    let mut app = App::new();
    setup_series_screen(&mut app, 60);

    handle_key(&mut app, make_key(KeyCode::PageDown)).await;
    assert!(
        app.selected_series_stream_index > 1,
        "PageDown on Series should advance"
    );

    handle_key(&mut app, make_key(KeyCode::End)).await;
    assert_eq!(
        app.selected_series_stream_index, 59,
        "End on Series should jump to last"
    );

    handle_key(&mut app, make_key(KeyCode::Home)).await;
    assert_eq!(
        app.selected_series_stream_index, 0,
        "Home on Series should jump to first"
    );
}

#[tokio::test]
async fn test_pageup_pagedown_global_search() {
    let mut app = App::new();
    setup_global_search(&mut app, 50);

    handle_key(&mut app, make_key(KeyCode::PageDown)).await;
    assert!(
        app.selected_stream_index > 1,
        "PageDown in global search should advance"
    );

    handle_key(&mut app, make_key(KeyCode::Home)).await;
    assert_eq!(
        app.selected_stream_index, 0,
        "Home in global search should jump to first"
    );

    handle_key(&mut app, make_key(KeyCode::End)).await;
    assert_eq!(
        app.selected_stream_index, 49,
        "End in global search should jump to last"
    );
}

#[tokio::test]
async fn test_g_g_vim_style_vod_streams() {
    let mut app = App::new();
    setup_vod_screen(&mut app, 50);

    handle_key(&mut app, make_key(KeyCode::Char('g'))).await;
    assert_eq!(
        app.selected_vod_stream_index, 0,
        "g on VOD streams should jump to top"
    );

    handle_key(&mut app, make_key(KeyCode::Char('j'))).await;
    assert_eq!(app.selected_vod_stream_index, 1, "j should move down one");

    handle_key(&mut app, make_key(KeyCode::Char('G'))).await;
    assert_eq!(
        app.selected_vod_stream_index, 49,
        "G on VOD streams should jump to bottom"
    );
}

#[tokio::test]
async fn test_category_page_navigation_live() {
    let mut app = App::new();
    app.current_screen = CurrentScreen::Categories;
    app.active_pane = Pane::Categories;
    app.grid_cols = 1;
    for i in 0..40 {
        app.categories
            .push(make_category(&i.to_string(), &format!("Cat {}", i)));
    }
    app.selected_category_index = 0;
    app.category_list_state.select(Some(0));
    ensure_areas(&mut app);

    handle_key(&mut app, make_key(KeyCode::PageDown)).await;
    assert!(
        app.selected_category_index > 1,
        "PageDown in categories should advance"
    );

    handle_key(&mut app, make_key(KeyCode::End)).await;
    assert_eq!(
        app.selected_category_index, 39,
        "End in categories should jump to last"
    );

    handle_key(&mut app, make_key(KeyCode::Home)).await;
    assert_eq!(
        app.selected_category_index, 0,
        "Home in categories should jump to first"
    );
}

#[tokio::test]
async fn test_series_episodes_page_navigation() {
    let mut app = App::new();
    app.current_screen = CurrentScreen::SeriesStreams;
    app.active_pane = Pane::Episodes;
    for i in 0..30 {
        app.series_episodes
            .push(matrix_iptv_lib::api::SeriesEpisode {
                id: Some(FlexId::from_number(i as i64)),
                episode_num: i as i32,
                title: Some(format!("Episode {}", i)),
                container_extension: Some("mp4".to_string()),
                info: None,
                season: 1,
                direct_source: String::new(),
            });
    }
    app.selected_series_episode_index = 0;
    app.series_episode_list_state.select(Some(0));
    ensure_areas(&mut app);

    handle_key(&mut app, make_key(KeyCode::PageDown)).await;
    assert!(
        app.selected_series_episode_index > 1,
        "PageDown in episodes should advance"
    );

    handle_key(&mut app, make_key(KeyCode::End)).await;
    assert_eq!(
        app.selected_series_episode_index, 29,
        "End in episodes should jump to last"
    );

    handle_key(&mut app, make_key(KeyCode::Home)).await;
    assert_eq!(
        app.selected_series_episode_index, 0,
        "Home in episodes should jump to first"
    );
}

#[tokio::test]
async fn test_pagedown_does_not_wrap() {
    let mut app = App::new();
    setup_live_screen(&mut app, 100);

    handle_key(&mut app, make_key(KeyCode::End)).await;
    assert_eq!(app.selected_stream_index, 99);

    handle_key(&mut app, make_key(KeyCode::PageDown)).await;
    assert_eq!(
        app.selected_stream_index, 99,
        "PageDown at bottom should not wrap around"
    );

    handle_key(&mut app, make_key(KeyCode::Home)).await;
    handle_key(&mut app, make_key(KeyCode::PageUp)).await;
    assert_eq!(
        app.selected_stream_index, 0,
        "PageUp at top should not wrap around"
    );
}
