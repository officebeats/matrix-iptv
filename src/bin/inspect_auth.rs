use matrix_iptv_lib::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args: Vec<String> = std::env::args().collect();

    let (base_url, username, password, name) = if args.len() >= 4 {
        (
            args[1].clone(),
            args[2].clone(),
            args[3].clone(),
            "Manual".to_string(),
        )
    } else {
        let config = AppConfig::load()?;
        if let Some(acc) = config.accounts.first() {
            (
                acc.base_url.clone(),
                acc.username.clone(),
                acc.password.clone(),
                acc.name.clone(),
            )
        } else {
            return Err(anyhow::anyhow!("No accounts found and no CLI args provided. Usage: inspect_auth <url> <user> <pass>"));
        }
    };

    println!("📡 Inspecting account: {}", name);
    println!("🔗 URL: {}", base_url);

    use matrix_iptv_lib::api::XtreamClient;
    let client = XtreamClient::new(base_url, username, password);

    match client.authenticate().await {
        Ok((success, user_info, _server_info)) => {
            println!("✅ Authentication Success: {}", success);
            if let Some(info) = user_info {
                println!("👤 User Status: {:?}", info.status);
                println!("📅 Exp Date: {:?}", info.exp_date);
            }
        }
        Err(e) => {
            println!("❌ Authentication Failed!");
            println!("{}", e);
        }
    }
    Ok(())
}
