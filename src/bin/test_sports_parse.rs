use matrix_iptv_lib::sports::parse_sports_event;
fn main() {
    let names = vec![
        "Los Angeles Lakers vs Portland Trail Blazers",
        "🏀 Los Angeles Lakers vs Portland Trail Blazers",
        "NBA: Lakers vs Trail Blazers",
        "Hawks (ATL) x Bulls (CHI)",
    ];
    for name in names {
        println!("Input: {}", name);
        match parse_sports_event(name) {
            Some(event) => println!("  Team1: {}, Team2: {}", event.team1, event.team2),
            None => println!("  NO MATCH"),
        }
        println!();
    }
}
