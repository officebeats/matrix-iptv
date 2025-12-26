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
        config.dns_provider,
    ).await?;
    
    client.authenticate().await?;
    
    // Get series categories
    let cats = client.get_series_categories().await?;
    println!("Found {} series categories", cats.len());
    
    if let Some(first_cat) = cats.first() {
        println!("\nCategory: {}", first_cat.category_name);
        
        // Get series in this category
        let series = client.get_series_streams(&first_cat.category_id).await?;
        println!("Found {} series in category", series.len());
        
        if let Some(first_series) = series.first() {
            println!("\nFirst series structure:");
            println!("{:#?}", first_series);
        }
    }
    
    Ok(())
}
