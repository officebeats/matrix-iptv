fn main() {
    let re = regex::Regex::new(r"(?i)(?:^|[:])\s*([^:(|]+?)\s*(?:\(([^)]+?)\))?\s*(?:(?:\s+(?:x|vs|at)\s+)|@|\s-\s)\s*([^:(\[|/]+?)\s*(?:\(([^)]+?)\))?(?:\s+(?:start:|\[|\(|\d{1,2}:\d{2}|\s+-\s+|/|\|)|$)").unwrap();
    let display_name = "Hawks (ATL) x Bulls (CHI) start:2025-12-21 20:00:00";
    if let Some(caps) = re.captures(display_name) {
        println!("team1: {:?}", caps.get(1).map(|m| m.as_str().trim()));
        println!("team1_abbr: {:?}", caps.get(2).map(|m| m.as_str().trim()));
        println!("team2: {:?}", caps.get(3).map(|m| m.as_str().trim()));
        println!("team2_abbr: {:?}", caps.get(4).map(|m| m.as_str().trim()));
    } else {
        println!("No match!");
    }
}
