use matrix_iptv_lib::api::XtreamClient;
use matrix_iptv_lib::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::load()?;
    let account = &config.accounts[0]; // Trex
    
    let client = XtreamClient::new_with_doh(
        account.base_url.clone(),
        account.username.clone(),
        account.password.clone(),
    ).await?;
    
    client.authenticate().await?;
    
    // Get series categories
    let cats = client.get_series_categories().await?;
    
    // Find a category with series
    if let Some(cat) = cats.iter().find(|c| c.category_name.contains("APPLE")) {
        println!("Category: {}", cat.category_name);
        
        // Get series in this category
        let series = client.get_series_streams(&cat.category_id).await?;
        println!("Found {} series", series.len());
        
        // Use first series
        if let Some(show) = series.first() {
            println!("\nTesting with Series: {}", show.name);
            println!("Stream ID: {:?}", show.stream_id);
            
            // Get series ID from stream_id
            let series_id = match &show.stream_id {
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::String(s) => s.clone(),
                _ => show.stream_id.to_string(),
            };
            
            println!("\nFetching series info for ID: {}", series_id);
            
            // Make direct API call to get_series_info
            let url = format!(
                "{}/player_api.php?username={}&password={}&action=get_series_info&series_id={}",
                account.base_url, account.username, account.password, series_id
            );
            
            println!("URL: {}", url);
            
            let response: serde_json::Value = reqwest::get(&url).await?.json().await?;
            println!("\nSeries Info Response:");
            println!("{:#?}", response);
        }
    }
    
    Ok(())
}
