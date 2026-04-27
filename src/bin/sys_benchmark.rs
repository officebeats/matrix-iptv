use matrix_iptv_lib::api::{Category, Stream};
use matrix_iptv_lib::config::ProcessingMode;
use matrix_iptv_lib::flex_id::FlexId;
use matrix_iptv_lib::preprocessing::{preprocess_categories, preprocess_streams};
use std::collections::HashSet;
use std::time::Instant;

#[tokio::main]
async fn main() {
    println!("=== Systems Architecture Performance Benchmark ===");
    println!("Target Scale: 50,000 Streams, 1,000 Categories\n");

    // 1. Setup Mock Data
    println!("Generating mock data...");
    let mut streams = Vec::new();
    for i in 0..50000 {
        // Use "USA | NFL" to pass both is_american_live and is_sports_content.
        let s = Stream {
            num: Some(FlexId::from_number((i % 1000) as i64)),
            name: format!("USA | NFL: Game Title {} (2024) [4K]", i),
            stream_id: FlexId::from_number(i as i64),
            epg_channel_id: Some("NFL.HD".to_string()),
            category_id: Some("1".to_string()),
            container_extension: Some("mkv".to_string()),
            rating: Some(4.5),
            stream_type: "live".to_string(),
            ..Default::default()
        };
        streams.push(s);
    }

    let mut categories = Vec::new();
    for i in 0..1000 {
        let c = Category {
            category_id: i.to_string(),
            category_name: format!("VIP | US | Action Movies {}", i),
            parent_id: FlexId::from_number(0),
            ..Default::default()
        };
        categories.push(c);
    }

    let favorites = HashSet::new();
    // Use Sports and Merica to trigger all logic paths (Cleaning + Sports Icons)
    let modes = vec![ProcessingMode::Merica, ProcessingMode::Sports];

    // 2. Measure Categories Preprocessing (Zero-Latency Projection)
    println!("Running Category Pre-parsing...");
    let start_cat = Instant::now();
    preprocess_categories(
        &mut categories,
        &favorites,
        &modes,
        true,
        false,
        "BenchmarkAccount",
    );
    let duration_cat = start_cat.elapsed();
    println!("  >> Processed 1,000 categories in: {:?}\n", duration_cat);

    // 3. Measure Stream Preprocessing (Multi-Core & Zero-Copy Sort)
    println!("Running Stream Preprocessing (Filter + Cleaning + Zero-Copy Sort)...");
    let start_proc = Instant::now();

    // We pass None for tx to skip UI messages in benchmark
    // is_live = true to match mock data
    preprocess_streams(
        &mut streams,
        &favorites,
        &modes,
        true,
        "BenchmarkAccount",
        None,
    );

    let duration_proc = start_proc.elapsed();
    println!(
        "  >> Processed {} streams (remaining after filter) in: {:?}\n",
        streams.len(),
        duration_proc
    );

    if streams.is_empty() {
        println!("❌ ERROR: All streams filtered out! Check filter logic in preprocessing.rs");
        return;
    }

    println!("Verification Metrics:");
    println!("  - Cleaned Names (Sample): {}", streams[0].clean_name);
    println!(
        "  - Multi-Core Utilization: Detected {} logical cores",
        num_cpus_count()
    );

    // Benchmark target: < 800ms for 50k streams (including filtering, parallel cleaning, and sorting)
    if duration_proc.as_millis() < 800 {
        println!("✅ PERFORMANCE TARGET MET: Preprocessing sub-800ms at scale.");
    } else {
        println!("⚠️ PERFORMANCE WARNING: Preprocessing exceeded 800ms threshold.");
    }
}

fn num_cpus_count() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}
