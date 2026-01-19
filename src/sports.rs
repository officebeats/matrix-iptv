use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use anyhow::Result;


/// Returns the primary color for a team based on its name or abbreviation.
pub fn get_team_color(name: &str) -> Color {
    let name = name.to_uppercase();

    // NBA
    if name.contains("HAWKS") || name == "ATL" {
        return Color::Rgb(224, 58, 62);
    }
    if name.contains("CELTICS") || name == "BOS" {
        return Color::Rgb(0, 200, 80); // Brighter Green
    }
    if name.contains("NETS") || name == "BKN" {
        return Color::Rgb(200, 200, 200); // Silver instead of Black
    }
    if name.contains("HORNETS") || name == "CHA" {
        return Color::Cyan; // Max brightness
    }
    if name.contains("BULLS") || name == "CHI" {
        return Color::Rgb(206, 17, 65);
    }
    if name.contains("CAVALIERS") || name == "CLE" {
        return Color::Rgb(134, 0, 56);
    }
    if name.contains("MAVERICKS") || name == "DAL" {
        return Color::Cyan;
    }
    if name.contains("NUGGETS") || name == "DEN" {
        return Color::Rgb(254, 197, 36);
    }
    if name.contains("PISTONS") || name == "DET" {
        return Color::Rgb(135, 206, 250); // Light Sky Blue
    }
    if name.contains("WARRIORS") || name == "GSW" {
        return Color::Cyan;
    }
    if name.contains("ROCKETS") || name == "HOU" {
        return Color::Rgb(206, 17, 65);
    }
    if name.contains("PACERS") || name == "IND" {
        return Color::Rgb(253, 187, 48);
    }
    if name.contains("CLIPPERS") || name == "LAC" {
        return Color::Rgb(200, 16, 46);
    }
    if name.contains("LAKERS") || name == "LAL" {
        return Color::Rgb(85, 37, 131);
    }
    if name.contains("GRIZZLIES") || name == "MEM" {
        return Color::Rgb(93, 118, 169);
    }
    if name.contains("HEAT") || name == "MIA" {
        return Color::Rgb(152, 0, 46);
    }
    if name.contains("BUCKS") || name == "MIL" {
        return Color::Rgb(0, 71, 27);
    }
    if name.contains("TIMBERWOLVES") || name == "MIN" {
        return Color::Cyan;
    }
    if name.contains("PELICANS") || name == "NOP" {
        return Color::Rgb(135, 206, 250);
    }
    if name.contains("KNICKS") || name == "NYK" {
        return Color::Rgb(245, 132, 38);
    }
    if name.contains("THUNDER") || name == "OKC" {
        return Color::Cyan; // Brighter Blue
    }
    if name.contains("MAGIC") || name == "ORL" {
        return Color::Cyan; // Brighter Blue
    }
    if name.contains("76ERS") || name == "PHI" {
        return Color::Rgb(100, 200, 255);
    }
    if name.contains("SUNS") || name == "PHX" {
        return Color::Rgb(29, 17, 96);
    }
    if name.contains("BLAZERS") || name == "POR" {
        return Color::Rgb(224, 58, 62);
    }
    if name.contains("KINGS") || name == "SAC" {
        return Color::Rgb(90, 45, 129);
    }
    if name.contains("SPURS") || name == "SAS" {
        return Color::Rgb(196, 206, 212);
    }
    if name.contains("RAPTORS") || name == "TOR" {
        return Color::Rgb(206, 17, 65);
    }
    if name.contains("JAZZ") || name == "UTA" {
        return Color::Rgb(100, 200, 255); // Major brightness bump
    }
    if name.contains("WIZARDS") || name == "WAS" {
        return Color::Rgb(227, 24, 55);
    }

    // NFL
    if name.contains("CARDINALS") || name == "ARI" {
        return Color::Rgb(151, 35, 63);
    }
    if name.contains("FALCONS") {
        return Color::Rgb(167, 25, 48);
    }
    if name.contains("RAVENS") || name == "BAL" {
        return Color::Rgb(186, 85, 211); // Medium Orchid (Bright Purple)
    }
    if name.contains("BILLS") || name == "BUF" {
        return Color::Rgb(100, 200, 255);
    }
    if name.contains("PANTHERS") || name == "CAR" {
        return Color::Cyan;
    }
    if name.contains("BEARS") {
        return Color::Rgb(135, 206, 235); // Sky Blue
    }
    if name.contains("BENGALS") || name == "CIN" {
        return Color::Rgb(255, 140, 0); // Orange
    }
    if name.contains("BROWNS") {
        return Color::Rgb(49, 29, 0);
    }
    if name.contains("COWBOYS") {
        return Color::Rgb(135, 206, 250);
    }
    if name.contains("BRONCOS") {
        return Color::Rgb(251, 79, 20);
    }
    if name.contains("LIONS") {
        return Color::Rgb(100, 200, 255);
    }
    if name.contains("PACKERS") || name == "GB" {
        return Color::Rgb(50, 205, 50); // Lime Green
    }
    if name.contains("TEXANS") {
        return Color::Rgb(135, 206, 250); // Replacing Deep Steel Blue
    }
    if name.contains("COLTS") {
        return Color::Rgb(135, 206, 250);
    }
    if name.contains("JAGUARS") || name == "JAX" {
        return Color::Rgb(0, 255, 220); // Teal -> Bright Cyan-Teal
    }
    if name.contains("CHIEFS") || name == "KC" {
        return Color::Rgb(227, 24, 55);
    }
    if name.contains("RAIDERS") || name == "LV" {
        return Color::Rgb(200, 200, 200);
    }
    if name.contains("CHARGERS") {
        return Color::Rgb(100, 220, 255);
    }
    if name.contains("RAMS") {
        return Color::Rgb(100, 200, 255);
    }
    if name.contains("DOLPHINS") {
        return Color::Rgb(0, 255, 230);
    }
    if name.contains("VIKINGS") {
        return Color::Rgb(79, 38, 131);
    }
    if name.contains("PATRIOTS") || name == "NE" {
        return Color::Rgb(100, 180, 255); // Brighter than steel blue
    }
    if name.contains("SAINTS") || name == "NO" {
        return Color::Rgb(211, 188, 141);
    }
    if name.contains("GIANTS") || name == "NYG" {
        return Color::Rgb(100, 200, 255);
    }
    if name.contains("JETS") || name == "NYJ" {
        return Color::Rgb(18, 87, 64);
    }
    if name.contains("EAGLES") {
        return Color::Rgb(0, 250, 200); // Midnight green is too dark, go Mint
    }
    if name.contains("STEELERS") || name == "PIT" {
        return Color::Rgb(255, 182, 18);
    }
    if name.contains("49ERS") || name == "SF" {
        return Color::Rgb(170, 0, 0);
    }
    if name.contains("SEAHAWKS") || name == "SEA" {
        return Color::Cyan;
    }
    if name.contains("BUCCANEERS") || name == "TB" {
        return Color::Rgb(213, 10, 10);
    }
    if name.contains("TITANS") || name == "TEN" {
        return Color::Cyan;
    }
    if name.contains("COMMANDERS") {
        return Color::Rgb(119, 49, 65);
    }

    // NHL
    if name.contains("BRUINS") || name == "BOS" {
        return Color::Rgb(255, 184, 28);
    }
    if name.contains("BLACKHAWKS") || name == "CHI" {
        return Color::Rgb(207, 10, 44);
    }
    if name.contains("RED WINGS") || name == "DET" {
        return Color::Rgb(206, 17, 38);
    }
    if name.contains("MAPLE LEAFS") || name == "TOR" {
        return Color::Rgb(100, 200, 255);
    }
    if name.contains("CANADIENS") || name == "MTL" {
        return Color::Rgb(175, 30, 45);
    }
    if name.contains("RANGERS") || name == "NYR" {
        return Color::Rgb(100, 200, 255);
    }
    if name.contains("PENGUINS") || name == "PIT" {
        return Color::Rgb(252, 181, 20);
    }

    // MLB
    if name.contains("YANKEES") || name == "NYY" {
        return Color::Rgb(135, 206, 250); // Royal Blue
    }
    if name.contains("RED SOX") || name == "BOS" {
        return Color::Rgb(189, 48, 57);
    }
    if name.contains("DODGERS") || name == "LAD" {
        return Color::Rgb(135, 206, 250);
    }
    if name.contains("CUBS") || name == "CHC" {
        return Color::Rgb(135, 206, 250);
    }
    if name.contains("GIANTS") || name == "SFG" {
        return Color::Rgb(253, 90, 30);
    }
    if name.contains("METS") || name == "NYM" {
        return Color::Rgb(135, 206, 250);
    }

    // Premier League (Soccer)
    if name.contains("ARSENAL") || name == "ARS" {
        return Color::Rgb(239, 1, 7);
    }
    if name.contains("CHELSEA") || name == "CHE" {
        return Color::Rgb(100, 200, 255);
    }
    if name.contains("LIVERPOOL") || name == "LIV" {
        return Color::Rgb(200, 16, 46);
    }
    if name.contains("MAN CITY") || name == "MCI" {
        return Color::Cyan;
    }
    if name.contains("MAN UTD") || name == "MUN" {
        return Color::Rgb(218, 41, 28);
    }
    if name.contains("SPURS") || name == "TOT" {
        return Color::Rgb(19, 34, 87);
    }

    // MLS
    if name.contains("INTER MIAMI") {
        return Color::Rgb(247, 181, 205);
    }
    if name.contains("LAFC") {
        return Color::Rgb(0, 0, 0);
    }
    if name.contains("GALAXY") || name == "LAG" {
        return Color::Rgb(100, 200, 255);
    }
    if name.contains("SOUNDERS") {
        return Color::Rgb(93, 151, 65);
    }
    if name.contains("ATLANTA UNITED") {
        return Color::Rgb(128, 0, 10);
    }

    // UFC / Combat Sports
    if name.contains("UFC") {
        return Color::Rgb(210, 10, 10);
    } // Red
    if name.contains("PFL") {
        return Color::Rgb(100, 200, 255);
    }
    if name.contains("BELLATOR") {
        return Color::Yellow;
    } // Goldish

    // Fallback
    Color::Reset
}

/// Lightens a color for better visibility on dark terminals.
/// Boosts RGB values closer to 255 while preserving hue.
fn lighten_color(color: Color) -> Color {
    match color {
        Color::Rgb(r, g, b) => {
            // Calculate luminance (using rough approximation)
            let luminance = (r as f32 * 0.299 + g as f32 * 0.587 + b as f32 * 0.114) / 255.0;
            
            // If color is too dark (luminance < 0.5), lighten it (increased from 0.4)
            if luminance < 0.5 {
                let boost = 1.7 + (0.5 - luminance); // Increased boost by ~15%
                let new_r = ((r as f32 * boost).min(255.0)) as u8;
                let new_g = ((g as f32 * boost).min(255.0)) as u8;
                let new_b = ((b as f32 * boost).min(255.0)) as u8;
                Color::Rgb(new_r.max(100), new_g.max(100), new_b.max(100)) // Raised floor from 80 to 100
            } else {
                Color::Rgb(r, g, b)
            }
        }
        other => other,
    }
}

/// Returns the primary color for a team with Home/Away fallback logic.
/// Home = Matrix Green, Away = White.
/// Colors are automatically lightened for dark terminal visibility.
pub fn get_team_color_with_fallback(name: &str, is_home: bool) -> Color {
    let specific = get_team_color(name);
    if specific != Color::Reset {
        lighten_color(specific)
    } else if is_home {
        crate::ui::colors::MATRIX_GREEN
    } else {
        Color::White
    }
}

pub fn is_generic_label(name: &str) -> bool {
    let name = name.to_uppercase();
    let generics = [
        "MNF", "SNF", "TNF", "NFL LIVE", "NFL MEDIA", "NFL NETWORK", "NFL PACKAGE",
        "NBA TV", "NBA PACKAGE", "NBA GAMETIME", "MLB TV", "MLB PACKAGE",
        "EVENT ONLY", "LIVE NOW", "REPLAY", "FULL REPLAY", "DIRECT TV",
        "SPORTS", "FOOTBALL", "BASKETBALL", "BASEBALL", "HOCKEY", "SOCCER",
        "LIVE SPORTS", "GAME PASS", "REDZONE", "NFL REDZONE"
    ];

    for g in generics {
        if name == g || name.contains(g) && name.len() < g.len() + 5 {
            return true;
        }
    }

    // Check for "PACKAGE 01", "LIVE 05" etc.
    let re_generic = regex::Regex::new(r"(?i)^(LIVE|PACKAGE|NETWORK|TV|STREAM)\s+\d+$").unwrap();
    if re_generic.is_match(&name) {
        return true;
    }

    false
}

#[derive(Debug, Clone)]
pub struct SportsEvent {
    pub team1: String, // Home
    pub team1_abbr: Option<String>,
    pub team2: String, // Away
    pub team2_abbr: Option<String>,
    pub start_time_raw: String,
}

pub fn parse_sports_event(display_name: &str) -> Option<SportsEvent> {
    // Regex for: [Prefix:] Team One (T1) [separator] Team Two (T2) [time/other info]
    // Supported separators: x, vs, @, - (if surrounded by spaces)
    // We use a non-greedy match for names and look for boundaries like " - ", " start:", "[", or end of string.
    // Enhanced stop markers: look for common IPTV suffixes like " (HD)", " - ET", " / UK", " | ", etc.
    let re = regex::Regex::new(r"(?i)(?:^|[:])\s*([^:(|]+?)\s*(?:\(([^)]+?)\))?\s*(?:(?:\s+(?:x|vs|at)\s+)|@|\s-\s)\s*([^:(\[|/]+?)\s*(?:\(([^)]+?)\))?(?:\s+(?:start:|\[|\(|\d{1,2}:\d{2}|\s+-\s+|/|\|)|$)").ok()?;

    if let Some(caps) = re.captures(display_name) {
        let team1 = caps.get(1)?.as_str().trim().to_string();
        let team1_abbr = caps.get(2).map(|m| m.as_str().trim().to_string());
        let team2 = caps.get(3)?.as_str().trim().to_string();
        let team2_abbr = caps.get(4).map(|m| m.as_str().trim().to_string());
        
        // Debug prints
        if display_name.contains("start:2025-12-21") {
            println!("DEBUG: display_name: {:?}", display_name);
        }

        // Scrub prefixes from team1
        let mut team1 = team1;

        // 1. Strip [brackets] (e.g. [EVENT ONLY])
        if team1.contains('[') {
            let re_bracket = regex::Regex::new(r"\[.*?\]").unwrap();
            team1 = re_bracket.replace_all(&team1, "").to_string();
        }

        // 2. Strip common patterns like "LIVE FOOTBALL 01", "NBA PACKAGE", etc.
        // Usually these are at the start and followed by spaces.
        let prefixes = [
            r"(?i)^LIVE\s+FOOTBALL\s+\d+\b",
            r"(?i)^LIVE\s+NBA\s+\d+\b",
            r"(?i)^NBA\s+PACKAGE\b",
            r"(?i)^NFL\s+PACKAGE\b",
            r"(?i)^WORLD\s+SPORT\b",
            r"(?i)^UK\s+SPORTS\b",
            r"(?i)^US\s+SPORTS\b",
            r"(?i)^✦●✦",
            r"(?i)^######",
            r"(?i)^======",
        ];

        for p in prefixes {
            let re_p = regex::Regex::new(p).unwrap();
            team1 = re_p.replace(&team1, "").to_string();
        }

        let team1 = team1.trim().to_string();

        // VALIDATION: If either team is a generic label, this isn't a specific matchup event
        if is_generic_label(&team1) || is_generic_label(&team2) {
            return None;
        }

        // Try to extract start time if present in the rest of the string
        let start_time_re =
            regex::Regex::new(r"(?i)start:\s*(\d{4}-\d{2}-\d{2}\s\d{2}:\d{2}:\d{2})").ok()?;
        let am_pm_re = regex::Regex::new(r"(?i)(\d{1,2}:\d{2})\s*(am|pm)").ok()?;

        let start_time_raw = if let Some(time_caps) = start_time_re.captures(display_name) {
            time_caps
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default()
        } else if let Some(time_caps) = am_pm_re.captures(display_name) {
            let t = time_caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let p = time_caps.get(2).map(|m| m.as_str()).unwrap_or("");
            format!("{} {}", t, p)
        } else {
            String::new()
        };

        return Some(SportsEvent {
            team1,
            team1_abbr,
            team2,
            team2_abbr,
            start_time_raw,
        });
    }

    None
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_generic_label() {
        assert!(is_generic_label("MNF"));
        assert!(is_generic_label("NFL Live 01"));
        assert!(is_generic_label("NBA Package"));
        assert!(is_generic_label("Live Now"));
        assert!(!is_generic_label("Cowboys"));
        assert!(!is_generic_label("Washington"));
    }

    #[test]
    fn test_parse_sports_event_blacklist() {
        // NFL LIVE 01 x MNF should return None
        assert!(parse_sports_event("NFL LIVE 01 x MNF").is_none());
        assert!(parse_sports_event("NBA TV x LIVE NOW").is_none());
    }

    #[test]
    fn test_parse_sports_event_valid() {
        let res = parse_sports_event("Cowboys x Commanders").unwrap();
        assert_eq!(res.team1, "Cowboys");
        assert_eq!(res.team2, "Commanders");
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamedTeam {
    pub name: String,
    pub badge: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamedTeams {
    pub home: Option<StreamedTeam>,
    pub away: Option<StreamedTeam>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamedSource {
    pub source: String,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamedMatch {
    pub id: String,
    pub title: String,
    pub category: String,
    pub date: i64, // Unix timestamp in ms
    pub popular: bool,
    pub teams: Option<StreamedTeams>,
    pub sources: Vec<StreamedSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamedStream {
    pub id: String,
    pub stream_no: i32,
    pub language: String,
    pub hd: bool,
    pub embed_url: String,
    pub source: String,
}

pub async fn fetch_streamed_matches(endpoint: &str) -> Result<Vec<StreamedMatch>> {
    let url = format!("https://streamed.pk/api/matches/{}", endpoint);
    let client = reqwest::Client::new();
    let res = client.get(url).send().await?;
    let matches: Vec<StreamedMatch> = res.json().await?;
    Ok(matches)
}

pub async fn fetch_streamed_links(source: &str, id: &str) -> Result<Vec<StreamedStream>> {
    let url = format!("https://streamed.pk/api/stream/{}/{}", source, id);
    let client = reqwest::Client::new();
    let res = client.get(url).send().await?;
    let streams: Vec<StreamedStream> = res.json().await?;
    Ok(streams)
}
