use matrix_iptv_lib::api::XtreamClient;
use matrix_iptv_lib::config::AppConfig;
use tokio;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let config = AppConfig::load()?;
    if let Some(acc) = config.accounts.get(0) {
        println!("📡 Inspecting account: {}", acc.name);
        let url = format!(
            "{}/player_api.php?username={}&password={}",
            acc.base_url, acc.username, acc.password
        );
        let client = reqwest::Client::new();
        let resp = client.get(&url).send().await?;
        let text = resp.text().await?;
        println!("📄 Raw Response:\n{}", text);
    } else {
        println!("❌ No accounts found.");
    }
    Ok(())
}
