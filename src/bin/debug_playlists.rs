use matrix_iptv_lib::api::XtreamClient;
use matrix_iptv_lib::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::load()?;
    println!("Inspecting {} accounts...", config.accounts.len());

    for (i, acc) in config.accounts.iter().enumerate() {
        if !acc.name.to_lowercase().contains("strong") && !acc.name.to_lowercase().contains("mega")
        {
            continue;
        }

        println!("\nAccount: {}", acc.name);
        let client = XtreamClient::new(
            acc.base_url.clone(),
            acc.username.clone(),
            acc.password.clone(),
        );

        let categories = client.get_live_categories().await?;
        let sports_cats: Vec<_> = categories
            .into_iter()
            .filter(|c| {
                let name = c.category_name.to_lowercase();
                name.contains("sports")
                    || name.contains("nba")
                    || name.contains("nfl")
                    || name.contains("football")
                    || name.contains("soccer")
                    || name.contains("sport")
            })
            .collect();

        println!("Found {} sports-related categories.", sports_cats.len());

        for cat in sports_cats.take(3) {
            println!("  Category: {}", cat.category_name);
            let streams = client.get_live_streams(&cat.category_id).await?;
            for s in streams.iter().take(10) {
                println!("    - {}", s.name);
            }
        }
    }

    Ok(())
}

trait Take {
    fn take(self, n: usize) -> Vec<matrix_iptv_lib::api::Category>;
}

impl Take for Vec<matrix_iptv_lib::api::Category> {
    fn take(mut self, n: usize) -> Vec<matrix_iptv_lib::api::Category> {
        self.truncate(n);
        self
    }
}
