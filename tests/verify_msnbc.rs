use std::collections::HashSet;
use matrix_iptv_lib::api::Stream;
use matrix_iptv_lib::flex_id::FlexId;
use matrix_iptv_lib::config::ProcessingMode;
use matrix_iptv_lib::preprocessing::preprocess_streams;

#[test]
fn test_merica_mode_msnbc_verification() {
    // 1. Setup: Create a mix of American, Foreign, and duplicate channels
    let mut streams = vec![
        Stream {
            stream_id: FlexId::from_string("1".to_string()),
            name: "US | MSNBC HEVC".to_string(), // Should pass
            ..Default::default()
        },
        Stream {
            stream_id: FlexId::from_string("2".to_string()),
            name: "ARABE-SPORTS | BEIN 1".to_string(), // Should fail (Foreign)
            ..Default::default()
        },
        Stream {
            stream_id: FlexId::from_string("3".to_string()),
            name: "UK | BBC ONE".to_string(), // Should fail (Foreign prefix)
            ..Default::default()
        },
        Stream {
            stream_id: FlexId::from_string("4".to_string()),
            name: "FIREPLACE TV".to_string(), // Should pass (Neutral)
            ..Default::default()
        },
        Stream {
            stream_id: FlexId::from_string("5".to_string()),
            name: "MSNBC".to_string(), // Should pass (Exact match)
            ..Default::default()
        },
    ];

    let favorites = HashSet::new();
    let modes = vec![ProcessingMode::Merica];
    
    // 2. Execution: Run the preprocessing logic
    // Note: The signature is (streams, favorites, modes, is_live, account_name)
    preprocess_streams(&mut streams, &favorites, &modes, true, "TestAccount");

    // 3. Verification: Check what survived
    let names: Vec<String> = streams.iter().map(|s| s.name.clone()).collect();
    println!("Survived Streams: {:?}", names);

    // Assertions
    assert!(names.iter().any(|n| n.contains("MSNBC")), "MSNBC should survive");
    // Check clean name (preprocessing removes "US |")
    assert!(names.iter().any(|n| n == "MSNBC HEVC"), "US | prefix should be cleaned");
    
    assert!(!names.iter().any(|n| n.contains("ARABE")), "ARABE-SPORTS should be removed");
    assert!(!names.iter().any(|n| n.contains("BBC ONE")), "UK Content should be removed");
    
    // 4. Search Verification (Simulating app.rs logic)
    // preprocessing populates search_name!
    let query = "msnbc";
    let search_results: Vec<&Stream> = streams.iter()
        .filter(|s| s.search_name.contains(query))
        .collect();
    
    assert!(!search_results.is_empty(), "Search for 'msnbc' should return results");
    // Should pass finding "MSNBC HEVC" (cleaned) and "MSNBC" (original id 5)
    // Note: clean_american_name replaces name.
    // Stream 1 becomes "MSNBC HEVC". search_name "msnbc hevc".
    // Stream 5 becomes "MSNBC". search_name "msnbc".
    
    println!("Search Results: {:?}", search_results.iter().map(|s| &s.name).collect::<Vec<_>>());
    assert_eq!(search_results.len(), 2, "Should find both MSNBC channels");
}
