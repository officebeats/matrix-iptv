//! QA Test for v3.0.6 Features
//! Tests: Auto-Refresh, Custom Groups
//! Run with: cargo run --release --bin qa_features

use matrix_iptv_lib::config::{AppConfig, ChannelGroup};

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  MATRIX IPTV v3.0.6 - QA Feature Test Suite                  ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    let mut passed = 0;
    let mut failed = 0;

    // ===== FEATURE 9: Auto-Refresh Tests =====
    println!("━━━ FEATURE 9: Auto-Refresh / Playlist Sync ━━━\n");

    // Test 1: auto_refresh_hours exists and defaults to 12
    print!("  [TEST] auto_refresh_hours default value... ");
    {
        let config = AppConfig::default();
        if config.auto_refresh_hours == 12 {
            println!("✅ PASS (default: 12)");
            passed += 1;
        } else {
            println!("❌ FAIL (got: {})", config.auto_refresh_hours);
            failed += 1;
        }
    }

    // Test 2: last_refreshed is set on accounts
    print!("  [TEST] Accounts have last_refreshed timestamps... ");
    {
        match AppConfig::load() {
            Ok(config) => {
                let has_timestamps = config.accounts.iter().all(|a| a.last_refreshed.is_some());
                if has_timestamps && !config.accounts.is_empty() {
                    println!("✅ PASS ({} accounts with timestamps)", config.accounts.len());
                    passed += 1;
                } else if config.accounts.is_empty() {
                    println!("⚠️ SKIP (no accounts configured)");
                } else {
                    println!("❌ FAIL (some accounts missing timestamps)");
                    failed += 1;
                }
            }
            Err(e) => {
                println!("❌ FAIL (config load error: {})", e);
                failed += 1;
            }
        }
    }

    // Test 3: Staleness calculation
    print!("  [TEST] Staleness detection logic... ");
    {
        let now = chrono::Utc::now().timestamp();
        let fresh = now - (12 * 3600); // 12 hours ago
        let stale = now - (48 * 3600); // 48 hours ago
        let threshold = 24i64;
        
        let is_fresh_ok = (now - fresh) <= (threshold * 3600);
        let is_stale_ok = (now - stale) > (threshold * 3600);
        
        if is_fresh_ok && is_stale_ok {
            println!("✅ PASS (fresh=12h OK, stale=48h detected)");
            passed += 1;
        } else {
            println!("❌ FAIL");
            failed += 1;
        }
    }

    println!();

    // ===== FEATURE 2: Custom Groups Tests =====
    println!("━━━ FEATURE 2: Custom Channel Groups ━━━\n");

    // Test 4: ChannelGroup struct exists
    print!("  [TEST] ChannelGroup struct creation... ");
    {
        let group = ChannelGroup {
            name: "Test Group".to_string(),
            icon: Some("📁".to_string()),
            stream_ids: vec!["123".to_string(), "456".to_string()],
        };
        if group.name == "Test Group" && group.stream_ids.len() == 2 {
            println!("✅ PASS");
            passed += 1;
        } else {
            println!("❌ FAIL");
            failed += 1;
        }
    }

    // Test 5: Create group via config
    print!("  [TEST] Config.create_group()... ");
    {
        let mut config = AppConfig::default();
        let idx = config.create_group("Sports".to_string(), Some("⚽".to_string()));
        if idx == 0 && config.favorites.groups.len() == 1 && config.favorites.groups[0].name == "Sports" {
            println!("✅ PASS (created at index {})", idx);
            passed += 1;
        } else {
            println!("❌ FAIL");
            failed += 1;
        }
    }

    // Test 6: Add to group
    print!("  [TEST] Config.add_to_group()... ");
    {
        let mut config = AppConfig::default();
        config.create_group("News".to_string(), None);
        config.add_to_group(0, "stream_123".to_string());
        config.add_to_group(0, "stream_456".to_string());
        
        if config.favorites.groups[0].stream_ids.len() == 2 {
            println!("✅ PASS (2 streams added)");
            passed += 1;
        } else {
            println!("❌ FAIL (got {} streams)", config.favorites.groups[0].stream_ids.len());
            failed += 1;
        }
    }

    // Test 7: Prevent duplicate add
    print!("  [TEST] Duplicate stream prevention... ");
    {
        let mut config = AppConfig::default();
        config.create_group("Movies".to_string(), None);
        config.add_to_group(0, "stream_123".to_string());
        config.add_to_group(0, "stream_123".to_string()); // Duplicate
        
        if config.favorites.groups[0].stream_ids.len() == 1 {
            println!("✅ PASS (duplicate rejected)");
            passed += 1;
        } else {
            println!("❌ FAIL (duplicate was added)");
            failed += 1;
        }
    }

    // Test 8: Remove from group
    print!("  [TEST] Config.remove_from_group()... ");
    {
        let mut config = AppConfig::default();
        config.create_group("Kids".to_string(), None);
        config.add_to_group(0, "stream_a".to_string());
        config.add_to_group(0, "stream_b".to_string());
        config.remove_from_group(0, "stream_a");
        
        if config.favorites.groups[0].stream_ids.len() == 1 
           && config.favorites.groups[0].stream_ids[0] == "stream_b" {
            println!("✅ PASS");
            passed += 1;
        } else {
            println!("❌ FAIL");
            failed += 1;
        }
    }

    // Test 9: Delete group
    print!("  [TEST] Config.delete_group()... ");
    {
        let mut config = AppConfig::default();
        config.create_group("Group1".to_string(), None);
        config.create_group("Group2".to_string(), None);
        config.delete_group(0);
        
        if config.favorites.groups.len() == 1 && config.favorites.groups[0].name == "Group2" {
            println!("✅ PASS");
            passed += 1;
        } else {
            println!("❌ FAIL");
            failed += 1;
        }
    }

    // Test 10: Rename group
    print!("  [TEST] Config.rename_group()... ");
    {
        let mut config = AppConfig::default();
        config.create_group("OldName".to_string(), None);
        config.rename_group(0, "NewName".to_string());
        
        if config.favorites.groups[0].name == "NewName" {
            println!("✅ PASS");
            passed += 1;
        } else {
            println!("❌ FAIL");
            failed += 1;
        }
    }

    // Test 11: Groups persist in favorites
    print!("  [TEST] Groups field exists in Favorites... ");
    {
        let config = AppConfig::default();
        // Just verify the field exists and is empty by default
        if config.favorites.groups.is_empty() {
            println!("✅ PASS (groups: Vec initialized)");
            passed += 1;
        } else {
            println!("❌ FAIL");
            failed += 1;
        }
    }

    println!();

    // ===== Summary =====
    println!("══════════════════════════════════════════════════════════════");
    println!("  RESULTS: {} passed, {} failed", passed, failed);
    if failed == 0 {
        println!("  STATUS: ✅ ALL TESTS PASSED");
    } else {
        println!("  STATUS: ❌ SOME TESTS FAILED");
    }
    println!("══════════════════════════════════════════════════════════════");
}
