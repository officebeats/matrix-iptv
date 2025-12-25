use matrix_iptv_lib::app::{App, CurrentScreen, Pane};
use matrix_iptv_lib::api::{Category, Stream};
use serde_json::json;

#[test]
fn test_american_mode_filtering() {
    let mut app = App::new();
    
    // Mock categories
    let categories = vec![
        Category {
            category_id: "ALL".to_string(),
            category_name: "All Channels".to_string(),
            parent_id: json!(0),
        },
        Category {
            category_id: "1".to_string(),
            category_name: "USA | News".to_string(),
            parent_id: json!(0),
        },
        Category {
            category_id: "2".to_string(),
            category_name: "UK | Sports".to_string(),
            parent_id: json!(0),
        },
        Category {
            category_id: "3".to_string(),
            category_name: "AMERICA | Movies".to_string(),
            parent_id: json!(0),
        },
        Category {
            category_id: "4".to_string(),
            category_name: "AM | Armenian Channel".to_string(),
            parent_id: json!(0),
        },
        Category {
            category_id: "5".to_string(),
            category_name: "AR | ALGERIE +6H USA".to_string(),
            parent_id: json!(0),
        },
    ];
    
    app.all_categories = categories;
    
    // Test Live TV Filtering
    app.current_screen = CurrentScreen::Categories;
    app.active_pane = Pane::Categories;
    
    // American Mode OFF (Default)
    app.config.american_mode = false;
    app.update_search();
    assert_eq!(app.categories.len(), 6, "Should show all 6 categories when American Mode is OFF");
    
    // American Mode ON
    app.config.american_mode = true;
    app.update_search();
    assert_eq!(app.categories.len(), 3, "Should show 3 categories (All, USA, AMERICA) and filter out UK, AM (Armenian), and AR (Arabic)");
    assert!(app.categories.iter().any(|c| c.category_name.contains("USA") && !c.category_name.contains("AR |")));
    assert!(app.categories.iter().any(|c| c.category_name.contains("AMERICA")));
    assert!(!app.categories.iter().any(|c| c.category_name.contains("UK")));
    assert!(!app.categories.iter().any(|c| c.category_name.contains("AM |")));
    assert!(!app.categories.iter().any(|c| c.category_name.contains("AR |")));
}

#[test]
fn test_vod_english_filtering() {
    let mut app = App::new();
    
    // Mock VOD categories
    let vod_categories = vec![
        Category {
            category_id: "1".to_string(),
            category_name: "Action | EN".to_string(),
            parent_id: json!(0),
        },
        Category {
            category_id: "2".to_string(),
            category_name: "Comedy | FR".to_string(),
            parent_id: json!(0),
        },
        Category {
            category_id: "3".to_string(),
            category_name: "Drama | ENGLISH".to_string(),
            parent_id: json!(0),
        },
    ];
    
    app.all_vod_categories = vod_categories;
    
    // Test VOD Filtering
    app.current_screen = CurrentScreen::VodCategories;
    app.active_pane = Pane::Categories;
    
    // American Mode ON
    app.config.american_mode = true;
    app.update_search();
    assert_eq!(app.vod_categories.len(), 2, "Should show 2 English VOD categories");
}

#[test]
fn test_live_stream_filtering() {
    let mut app = App::new();
    
    // Mock Live Streams
    let streams = vec![
        Stream {
            name: "US | CNN HD".to_string(),
            stream_type: "live".to_string(),
            stream_id: json!(1),
            category_id: Some("1".to_string()),
            ..Default::default()
        },
        Stream {
            name: "UK | BBC ONE".to_string(),
            stream_type: "live".to_string(),
            stream_id: json!(2),
            category_id: Some("1".to_string()),
            ..Default::default()
        },
        Stream {
            name: "USA | FOX NEWS".to_string(),
            stream_type: "live".to_string(),
            stream_id: json!(3),
            category_id: Some("1".to_string()),
            ..Default::default()
        },
    ];
    
    app.all_streams = streams;
    
    // Test Live Stream Filtering
    app.current_screen = CurrentScreen::Streams;
    app.active_pane = Pane::Streams;
    
    // American Mode ON
    app.config.american_mode = true;
    app.update_search();
    assert_eq!(app.streams.len(), 2, "Should show 2 American Live streams");
}
