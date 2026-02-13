use matrix_iptv_lib::api::Stream;
use matrix_iptv_lib::app::{App, CurrentScreen, Pane};
use std::sync::Arc;

#[test]
fn test_stream_caching_logic() {
    // Setup App
    let mut app = App::new();
    // Ensure we are in a state where update_search processes streams
    app.current_screen = CurrentScreen::Categories;
    app.active_pane = Pane::Streams;
    
    // Add a test stream
    let stream = Stream {
        name: "Test Stream US".to_string(),
        stream_id: serde_json::json!(1),
        ..Default::default()
    };
    app.all_streams = vec![Arc::new(stream)];
    
    // Trigger the caching logic
    app.update_search();
    
    // Verify results
    assert!(!app.streams.is_empty(), "Streams list should be populated");
    let cached_stream = &app.streams[0];
    
    // The Critical Verify: Is the cache populated?
    assert!(cached_stream.cached_parsed.is_some(), "cached_parsed should be Some after update_search");
    
    // Verify the parsed content matches expectation
    let parsed = cached_stream.cached_parsed.as_ref().unwrap();
    assert_eq!(parsed.original_name, "Test Stream US");
}
