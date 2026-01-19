use serde::{Deserialize, Serialize};
use anyhow::Result;
use reqwest::Client;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EspnResponse {
    pub events: Option<Vec<EspnEvent>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EspnEvent {
    pub id: String,
    pub date: String, // ISO 8601 UTC
    pub short_name: Option<String>, // e.g. "CHI @ GB"
    pub status: EspnStatus,
    pub competitions: Vec<EspnCompetition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EspnStatus {
    pub clock: Option<f64>,
    pub display_clock: Option<String>, // "12:00"
    pub period: Option<i32>,
    #[serde(rename = "type")]
    pub status_type: EspnStatusType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EspnStatusType {
    pub id: String,
    pub name: String, // STATUS_SCHEDULED, STATUS_IN_PROGRESS, STATUS_FINAL
    pub state: String, // "pre", "in", "post"
    pub detail: Option<String>, // "Final", "OT", "10:00 - 1st Quarter"
    pub short_detail: Option<String>, // "Final", "1st", "Halftime"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EspnCompetition {
    pub id: String,
    pub competitors: Vec<EspnCompetitor>,
    pub venue: Option<EspnVenue>,
    pub broadcasts: Option<Vec<EspnBroadcast>>,
    pub situation: Option<EspnSituation>,
    pub headlines: Option<Vec<EspnHeadline>>,
    pub series: Option<EspnSeries>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EspnBroadcast {
    pub market: Option<String>,
    pub names: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EspnSituation {
    pub last_play: Option<EspnLastPlay>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EspnLastPlay {
    pub text: Option<String>,
    pub probability: Option<EspnProbability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EspnProbability {
    pub home_win_percentage: Option<f64>,
    pub away_win_percentage: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EspnHeadline {
    #[serde(rename = "type")]
    pub headline_type: Option<String>, // "Recap"
    pub description: Option<String>,   // Full headline text
    pub short_link_text: Option<String>, // Short summary
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EspnSeries {
    pub summary: Option<String>, // "Series tied 2-2"
    #[serde(rename = "type")]
    pub series_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EspnLeaderCategory {
    pub name: Option<String>,     // "points", "rebounds", "assists"
    pub leaders: Option<Vec<EspnLeaderEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EspnLeaderEntry {
    pub display_value: Option<String>, // "24"
    pub athlete: Option<EspnAthlete>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EspnAthlete {
    pub display_name: Option<String>, // "Ja Morant"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EspnVenue {
    #[serde(rename = "fullName")]
    pub full_name: Option<String>,
    pub address: Option<EspnAddress>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EspnAddress {
    pub city: Option<String>,
    pub state: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EspnCompetitor {
    pub id: String,
    pub uid: Option<String>,
    pub order: Option<i32>,
    pub home_away: String, // "home" or "away"
    pub score: Option<String>,
    pub team: EspnTeam,
    pub leaders: Option<Vec<EspnLeaderCategory>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EspnTeam {
    pub id: String,
    pub uid: Option<String>,
    pub location: Option<String>, // "Chicago"
    pub name: Option<String>,     // "Bears"
    pub abbreviation: Option<String>, // "CHI"
    pub display_name: Option<String>, // "Chicago Bears"
    pub color: Option<String>,
    pub alternate_color: Option<String>,
    pub logo: Option<String>,     // Team logo URL
}

#[derive(Debug, Clone)]
pub struct ScoreGame {
    pub id: String,
    pub league: String,
    pub start_time: String,
    pub status_state: String, // pre, in, post
    pub status_detail: String, // "Final", "12:43 1st"
    pub home_team: String,
    pub home_score: String,
    pub home_abbr: String,
    pub home_color: Option<String>,    // Team primary color hex
    pub home_record: Option<String>,   // e.g., "24-18"
    pub home_logo: Option<String>,     // Team logo URL
    pub away_team: String,
    pub away_score: String,
    pub away_abbr: String,
    pub away_color: Option<String>,
    pub away_record: Option<String>,
    pub away_logo: Option<String>,
    pub display_clock: String,
    pub period: i32,
    pub venue_name: Option<String>,
    pub venue_city: Option<String>,
    pub venue_state: Option<String>,
    // Enhanced intelligence data
    pub broadcasts: Vec<String>,          // TV networks
    pub last_play: Option<String>,        // "Lakers Full timeout"
    pub home_win_pct: Option<f64>,        // Win probability
    pub away_win_pct: Option<f64>,
    pub headline: Option<String>,         // Game summary/recap headline
    pub series_summary: Option<String>,   // Playoff series status "Series tied 2-2"
    pub top_scorer: Option<String>,       // "Ja Morant - 24 PTS"
}

pub struct ScoreService {
    client: Client,
}

impl ScoreService {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
        }
    }

    pub async fn fetch_scores(&self) -> Result<Vec<ScoreGame>> {
        let leagues = vec![
            ("football/nfl", "NFL"),
            ("basketball/nba", "NBA"),
            ("mixed-martial-arts/ufc", "UFC"), // Might be different structure
            ("hockey/nhl", "NHL"),
            ("baseball/mlb", "MLB"),
            ("soccer/usa.1", "MLS"),
            ("soccer/eng.1", "EPL"), 
        ];

        let mut all_games = Vec::new();

        for (endpoint, league_name) in leagues {
            let url = format!(
                "http://site.api.espn.com/apis/site/v2/sports/{}/scoreboard",
                endpoint
            );

            if let Ok(resp) = self.client.get(&url).send().await {
                if let Ok(json) = resp.json::<EspnResponse>().await {
                    if let Some(events) = json.events {
                        for event in events {
                            if let Some(comp) = event.competitions.first() {
                                let home = comp.competitors.iter().find(|c| c.home_away == "home");
                                let away = comp.competitors.iter().find(|c| c.home_away == "away");

                                if let (Some(h), Some(a)) = (home, away) {
                                    let h_name = h.team.display_name.clone().unwrap_or_default();
                                    let a_name = a.team.display_name.clone().unwrap_or_default();
                                    
                                    let venue_name = comp.venue.as_ref().and_then(|v| v.full_name.clone());
                                    let (venue_city, venue_state) = if let Some(addr) = comp.venue.as_ref().and_then(|v| v.address.as_ref()) {
                                        (addr.city.clone(), addr.state.clone())
                                    } else {
                                        (None, None)
                                    };
                                    
                                    // Extract broadcasts
                                    let broadcasts: Vec<String> = comp.broadcasts.as_ref()
                                        .map(|bs| bs.iter()
                                            .flat_map(|b| b.names.clone().unwrap_or_default())
                                            .collect())
                                        .unwrap_or_default();
                                    
                                    // Extract last play and win probability
                                    let (last_play, home_win_pct, away_win_pct) = if let Some(sit) = &comp.situation {
                                        let lp = sit.last_play.as_ref().and_then(|p| p.text.clone());
                                        let hwp = sit.last_play.as_ref()
                                            .and_then(|p| p.probability.as_ref())
                                            .and_then(|pr| pr.home_win_percentage);
                                        let awp = sit.last_play.as_ref()
                                            .and_then(|p| p.probability.as_ref())
                                            .and_then(|pr| pr.away_win_percentage);
                                        (lp, hwp, awp)
                                    } else {
                                        (None, None, None)
                                    };
                                    
                                    // Extract headline (for post-game recaps)
                                    let headline = comp.headlines.as_ref()
                                        .and_then(|hl| hl.first())
                                        .and_then(|h| h.short_link_text.clone().or(h.description.clone()));
                                    
                                    // Extract series summary (for playoffs)
                                    let series_summary = comp.series.as_ref()
                                        .and_then(|s| s.summary.clone());
                                    
                                    // Extract top scorer (points leader from home team)
                                    let top_scorer = h.leaders.as_ref()
                                        .and_then(|cats| cats.iter().find(|c| c.name.as_deref() == Some("rating")))
                                        .and_then(|cat| cat.leaders.as_ref())
                                        .and_then(|leaders| leaders.first())
                                        .map(|l| {
                                            let name = l.athlete.as_ref()
                                                .and_then(|a| a.display_name.clone())
                                                .unwrap_or_default();
                                            let value = l.display_value.clone().unwrap_or_default();
                                            format!("{} - {}", name, value)
                                        });

                                    all_games.push(ScoreGame {
                                        id: event.id.clone(),
                                        league: league_name.to_string(),
                                        start_time: event.date.clone(),
                                        status_state: event.status.status_type.state.clone(),
                                        status_detail: event.status.status_type.short_detail.clone().unwrap_or(event.status.status_type.detail.clone().unwrap_or_default()),
                                        home_team: h_name,
                                        home_score: h.score.clone().unwrap_or("0".to_string()),
                                        home_abbr: h.team.abbreviation.clone().unwrap_or_default(),
                                        home_color: h.team.color.clone(),
                                        home_record: None, // Records require extra API call
                                        home_logo: h.team.logo.clone(),
                                        away_team: a_name,
                                        away_score: a.score.clone().unwrap_or("0".to_string()),
                                        away_abbr: a.team.abbreviation.clone().unwrap_or_default(),
                                        away_color: a.team.color.clone(),
                                        away_record: None,
                                        away_logo: a.team.logo.clone(),
                                        display_clock: event.status.display_clock.clone().unwrap_or_else(|| "00:00".to_string()),
                                        period: event.status.period.unwrap_or(0),
                                        venue_name,
                                        venue_city,
                                        venue_state,
                                        broadcasts,
                                        last_play,
                                        home_win_pct,
                                        away_win_pct,
                                        headline,
                                        series_summary,
                                        top_scorer,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(all_games)
    }
}
