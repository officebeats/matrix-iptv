use matrix_iptv_lib::sports::parse_sports_event;
use matrix_iptv_lib::scores::ScoreService;

#[tokio::main]
async fn main() {
    println!("=== ESPN Score Fetch Test ===\n");
    
    let service = ScoreService::new();
    match service.fetch_scores().await {
        Ok(scores) => {
            println!("Fetched {} games from ESPN\n", scores.len());
            
            // Find Lakers game
            for game in &scores {
                if game.home_team.to_lowercase().contains("lakers") || 
                   game.away_team.to_lowercase().contains("lakers") {
                    println!("=== LAKERS GAME FOUND ===");
                    println!("Home: {} ({})", game.home_team, game.home_score);
                    println!("Away: {} ({})", game.away_team, game.away_score);
                    println!("Status: {} - {}", game.status_state, game.status_detail);
                    println!("Clock: {}", game.display_clock);
                    println!();
                }
            }
            
            // Test matching with stream name
            let test_names = vec![
                "Los Angeles Lakers vs Portland Trail Blazers",
                "🏀 Los Angeles Lakers vs Portland Trail Blazers",
            ];
            
            println!("=== MATCHING TEST ===");
            for name in test_names {
                println!("Input: {}", name);
                if let Some(event) = parse_sports_event(name) {
                    let t1 = event.team1.chars()
                        .skip_while(|c| !c.is_ascii_alphanumeric() && !c.is_ascii_whitespace())
                        .collect::<String>().trim().to_lowercase();
                    let t2 = event.team2.to_lowercase();
                    println!("  Parsed: {} vs {}", t1, t2);
                    
                    let matched = scores.iter().find(|g| {
                        let h = g.home_team.to_lowercase();
                        let a = g.away_team.to_lowercase();
                        (h.contains(&t1) || a.contains(&t1) || t1.contains(&h) || t1.contains(&a)) &&
                        (h.contains(&t2) || a.contains(&t2) || t2.contains(&h) || t2.contains(&a))
                    });
                    
                    if let Some(m) = matched {
                        println!("  MATCHED: {} vs {} ({} - {})", m.home_team, m.away_team, m.home_score, m.away_score);
                    } else {
                        println!("  NO MATCH");
                    }
                } else {
                    println!("  Failed to parse sports event");
                }
                println!();
            }
        }
        Err(e) => println!("Failed to fetch scores: {}", e),
    }
}
