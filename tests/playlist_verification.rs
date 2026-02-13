use matrix_iptv_lib::app::{App, CurrentScreen, Pane};
use matrix_iptv_lib::api::Category;
use serde_json::json;
use std::sync::Arc;

#[test]
fn test_navigation_sync_live_tv() {
    let mut app = App::new();
    app.all_categories = vec![
        Arc::new(Category { category_id: "1".to_string(), category_name: "Action".to_string(), parent_id: json!(0), ..Default::default() }),
        Arc::new(Category { category_id: "2".to_string(), category_name: "Drama".to_string(), parent_id: json!(0), ..Default::default() }),
    ];
    
    // Enter screen
    app.current_screen = CurrentScreen::Categories;
    app.active_pane = Pane::Categories;
    app.update_search();
    assert_eq!(app.categories.len(), 2);
    
    // Search
    app.search_state.query = "Action".to_string();
    app.update_search();
    assert_eq!(app.categories.len(), 1);
    
    // Back to Selection
    app.current_screen = CurrentScreen::ContentTypeSelection;
    
    // Re-enter (Simulate the fix in main.rs)
    app.current_screen = CurrentScreen::Categories;
    app.active_pane = Pane::Categories;
    app.search_state.query.clear();
    app.search_mode = false;
    app.update_search(); // Added by fix
    
    assert_eq!(app.categories.len(), 2, "Live TV categories should be restored");
}

#[test]
fn test_navigation_sync_vod() {
    let mut app = App::new();
    app.all_vod_categories = vec![
        Arc::new(Category { category_id: "10".to_string(), category_name: "Movie1".to_string(), parent_id: json!(0), ..Default::default() }),
    ];
    
    app.current_screen = CurrentScreen::VodCategories;
    app.active_pane = Pane::Categories;
    app.update_search();
    assert_eq!(app.vod_categories.len(), 1);
    
    app.search_state.query = "NoMatch".to_string();
    app.update_search();
    assert_eq!(app.vod_categories.len(), 0);
    
    app.current_screen = CurrentScreen::ContentTypeSelection;
    
    // Re-enter
    app.current_screen = CurrentScreen::VodCategories;
    app.active_pane = Pane::Categories;
    app.search_state.query.clear();
    app.search_mode = false;
    app.update_search();
    
    assert_eq!(app.vod_categories.len(), 1, "VOD categories should be restored");
}

#[test]
fn test_navigation_sync_series() {
    let mut app = App::new();
    app.all_series_categories = vec![
        Arc::new(Category { category_id: "20".to_string(), category_name: "Series1".to_string(), parent_id: json!(0), ..Default::default() }),
    ];
    
    app.current_screen = CurrentScreen::SeriesCategories;
    app.active_pane = Pane::Categories;
    app.update_search();
    assert_eq!(app.series_categories.len(), 1);
    
    app.search_state.query = "Search".to_string();
    app.update_search();
    assert_eq!(app.series_categories.len(), 0);
    
    app.current_screen = CurrentScreen::ContentTypeSelection;
    
    // Re-enter
    app.current_screen = CurrentScreen::SeriesCategories;
    app.active_pane = Pane::Categories;
    app.search_state.query.clear();
    app.search_mode = false;
    app.update_search();
    
    assert_eq!(app.series_categories.len(), 1, "Series categories should be restored");
}
