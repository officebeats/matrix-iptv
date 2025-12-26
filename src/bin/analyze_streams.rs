use matrix_iptv_lib::api::XtreamClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use the first working account to fetch some stream names
    let client = XtreamClient::new_with_doh(
        "http://your-provider.com".to_string(),
        "username".to_string(),
        "password".to_string(),
        matrix_iptv_lib::config::DnsProvider::System,
    )
    .await?;

    client.authenticate().await?;

    println!("=== Stream Name Analysis ===\n");

    // Get categories and sample streams from each
    let cats = client.get_live_categories().await?;

    for cat in cats.iter().take(5) {
        println!("Category: {}", cat.category_name);

        if let Ok(streams) = client.get_live_streams(&cat.category_id).await {
            println!("  Sample streams ({} total):", streams.len());
            for stream in streams.iter().take(10) {
                println!("    - {}", stream.name);
            }
        }
        println!();
    }

    Ok(())
}
