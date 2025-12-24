use matrix_iptv_lib::config::AppConfig;

fn main() {
    println!("Starting Account Cleanup...");

    // Load Config
    let mut config = AppConfig::load().expect("Failed to load config");
    let original_count = config.accounts.len();
    println!("Loaded config with {} accounts.", original_count);

    if original_count < 8 {
        println!("Warning: Expected at least 8 accounts, found {}. Aborting to prevent accidental deletion.", original_count);
        return;
    }

    // Indices to remove: 7, 6, 0 (Descending order is critical)
    let indices_to_remove = vec![7, 6, 0];

    for &idx in &indices_to_remove {
        if idx < config.accounts.len() {
            let removed = config.accounts.remove(idx);
            println!(
                "Removed Account [{}]: {} ({})",
                idx, removed.name, removed.base_url
            );
        } else {
            println!("Index {} out of bounds, skipping.", idx);
        }
    }

    // Save
    config.save().expect("Failed to save config");
    println!(
        "Cleanup Complete. Remaining accounts: {}",
        config.accounts.len()
    );
}
