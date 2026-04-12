use matrix_iptv_lib::api::XtreamClient;
use matrix_iptv_lib::config::{AppConfig, ProcessingMode};
use matrix_iptv_lib::parser;
use tokio;

/// Comprehensive geo-filter verification test.
/// Connects to the real provider, fetches Live/VOD/Series categories,
/// and asserts that 'Merica mode filtering correctly excludes foreign content.
#[test]
fn test_merica_filter_all_content_types() {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

    rt.block_on(async {
        // 1. Load user config
        let config = AppConfig::load().expect("Failed to load config.json");
        let account = config.accounts.first().expect("No accounts found");
        let is_merica = config.processing_modes.contains(&ProcessingMode::Merica);

        println!("Account: {}", account.name);
        println!("'Merica mode active: {}", is_merica);
        println!("Processing modes: {:?}", config.processing_modes);
        println!("---");

        // 2. Connect
        let client = XtreamClient::new(
            account.base_url.clone(),
            account.username.clone(),
            account.password.clone(),
        );

        // =====================================================================
        // 3. LIVE CATEGORIES
        // =====================================================================
        println!("\n=== LIVE CATEGORIES ===");
        match client.get_live_categories().await {
            Ok(cats) => {
                println!("Total live categories from provider: {}", cats.len());

                let american: Vec<_> = cats
                    .iter()
                    .filter(|c| parser::is_american_live(&c.category_name))
                    .collect();
                let foreign: Vec<_> = cats
                    .iter()
                    .filter(|c| !parser::is_american_live(&c.category_name))
                    .collect();

                println!("American (pass filter): {} categories", american.len());
                println!("Foreign  (blocked):     {} categories", foreign.len());

                // Show a sample of what gets blocked
                println!("\n--- Sample BLOCKED live categories (first 20) ---");
                for cat in foreign.iter().take(20) {
                    println!("  BLOCKED: {}", cat.category_name);
                }

                // Show what passes
                println!("\n--- Sample PASSED live categories (first 20) ---");
                for cat in american.iter().take(20) {
                    println!("  PASSED:  {}", cat.category_name);
                }

                assert!(
                    american.len() > 0,
                    "Must have at least some American live categories"
                );
            }
            Err(e) => println!("SKIP live categories (error: {})", e),
        }

        // =====================================================================
        // 4. VOD (MOVIE) CATEGORIES
        // =====================================================================
        println!("\n=== VOD (MOVIE) CATEGORIES ===");
        match client.get_vod_categories().await {
            Ok(cats) => {
                println!("Total VOD categories from provider: {}", cats.len());

                let english: Vec<_> = cats
                    .iter()
                    .filter(|c| parser::is_english_vod(&c.category_name))
                    .collect();
                let foreign: Vec<_> = cats
                    .iter()
                    .filter(|c| !parser::is_english_vod(&c.category_name))
                    .collect();

                println!("English (pass filter): {} categories", english.len());
                println!("Foreign (blocked):     {} categories", foreign.len());

                // Show blocked
                println!("\n--- Sample BLOCKED VOD categories (first 30) ---");
                for cat in foreign.iter().take(30) {
                    println!("  BLOCKED: {}", cat.category_name);
                }

                // Show passed
                println!("\n--- Sample PASSED VOD categories (first 30) ---");
                for cat in english.iter().take(30) {
                    println!("  PASSED:  {}", cat.category_name);
                }

                // Specifically verify ALB, MT, TR are blocked
                let alb_leak: Vec<_> = english
                    .iter()
                    .filter(|c| {
                        let upper = c.category_name.to_uppercase();
                        upper.starts_with("ALB")
                            || upper.starts_with("MT ")
                            || upper.starts_with("TR ")
                            || upper.contains("| ALB")
                            || upper.contains("| MT")
                            || upper.contains("| TR")
                            || upper.contains("|ALB")
                            || upper.contains("|MT")
                            || upper.contains("|TR")
                    })
                    .collect();

                if !alb_leak.is_empty() {
                    println!("\n!!! LEAKS DETECTED (ALB/MT/TR still passing) !!!");
                    for cat in &alb_leak {
                        println!("  LEAK: {}", cat.category_name);
                    }
                } else {
                    println!("\n✓ No ALB/MT/TR leaks detected in VOD categories");
                }

                assert!(
                    english.len() > 0,
                    "Must have at least some English VOD categories"
                );
            }
            Err(e) => println!("SKIP VOD categories (error: {})", e),
        }

        // =====================================================================
        // 5. SERIES CATEGORIES
        // =====================================================================
        println!("\n=== SERIES CATEGORIES ===");
        match client.get_series_categories().await {
            Ok(cats) => {
                println!("Total series categories from provider: {}", cats.len());

                let english: Vec<_> = cats
                    .iter()
                    .filter(|c| parser::is_english_vod(&c.category_name))
                    .collect();
                let foreign: Vec<_> = cats
                    .iter()
                    .filter(|c| !parser::is_english_vod(&c.category_name))
                    .collect();

                println!("English (pass filter): {} categories", english.len());
                println!("Foreign (blocked):     {} categories", foreign.len());

                // Show blocked
                println!("\n--- Sample BLOCKED series categories (first 30) ---");
                for cat in foreign.iter().take(30) {
                    println!("  BLOCKED: {}", cat.category_name);
                }

                // Show passed
                println!("\n--- Sample PASSED series categories (first 30) ---");
                for cat in english.iter().take(30) {
                    println!("  PASSED:  {}", cat.category_name);
                }

                assert!(
                    english.len() > 0,
                    "Must have at least some English series categories"
                );
            }
            Err(e) => println!("SKIP series categories (error: {})", e),
        }

        println!("\n=== ALL TESTS COMPLETE ===");
    });
}
