use ratatui::style::Color;

/// Parsed category with extracted metadata
#[derive(Debug, Clone)]
pub struct ParsedCategory {
    pub original_name: String,
    pub country: Option<String>,
    pub quality: Option<Quality>,
    pub content_type: Option<ContentType>,
    pub display_name: String,
    pub is_vip: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Quality {
    UHD4K,
    FHD,
    HD,
    SD,
}

impl Quality {
    pub fn badge(&self) -> &'static str {
        match self {
            Quality::UHD4K => "4K",
            Quality::FHD => "FHD",
            Quality::HD => "HD",
            Quality::SD => "SD",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Quality::UHD4K => Color::Magenta,
            Quality::FHD => Color::Green,
            Quality::HD => Color::LightBlue,
            Quality::SD => Color::DarkGray,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContentType {
    Sports,
    News,
    Movies,
    Kids,
    Music,
    Documentary,
    Entertainment,
    Religious,
    PPV,
    General,
}

impl ContentType {
    pub fn icon(&self) -> &'static str {
        match self {
            ContentType::Sports => "⚽",
            ContentType::News => "📰",
            ContentType::Movies => "🎬",
            ContentType::Kids => "🧸",
            ContentType::Music => "🎵",
            ContentType::Documentary => "📚",
            ContentType::Entertainment => "📺",
            ContentType::Religious => "✝️",
            ContentType::PPV => "💎",
            ContentType::General => "",
        }
    }
}

/// Get color for country/region code
pub fn country_color(country: &str) -> Color {
    match country.to_uppercase().as_str() {
        "US" | "USA" | "AM" => Color::LightBlue,
        "UK" | "GB" | "EU" => Color::Green,
        "FR" | "FRANCE" => Color::Cyan,
        "CA" | "CANADA" => Color::Red,
        "VIP" => Color::Yellow,
        "4K" => Color::Magenta,
        _ => Color::White,
    }
}

/// Get flag emoji for country
pub fn country_flag(country: &str) -> &'static str {
    match country.to_uppercase().as_str() {
        "US" | "USA" | "AM" => "🇺🇸",
        "UK" | "GB" => "🇬🇧",
        "EU" => "🇪🇺",
        "FR" | "FRANCE" => "🇫🇷",
        "CA" | "CANADA" => "🇨🇦",
        "DE" | "GERMANY" => "🇩🇪",
        "ES" | "SPAIN" => "🇪🇸",
        "IT" | "ITALY" => "🇮🇹",
        "NL" | "NETHERLANDS" => "🇳🇱",
        "BE" | "BELGIUM" => "🇧🇪",
        "TR" | "TURKEY" => "🇹🇷",
        "IN" | "INDIA" => "🇮🇳",
        "PT" | "PORTUGAL" => "🇵🇹",
        "BR" | "BRAZIL" => "🇧🇷",
        "SA" | "AR" | "ARABIC" => "🇸🇦",
        "MX" | "MEXICO" => "🇲🇽",
        "PL" | "POLAND" => "🇵🇱",
        "RU" | "RUSSIA" => "🇷🇺",
        "UA" | "UKRAINE" => "🇺🇦",
        "VIP" => "⭐",
        "4K" => "🟣",
        _ => "",
    }
}

/// Check if a name/category is American live content
pub fn is_american_live(name: &str) -> bool {
    let upper = name.to_uppercase();
    // Keywords for USA
    upper.contains("USA") || 
    upper.contains(" US") || 
    upper.starts_with("US") ||
    upper.contains("|US") ||
    upper.contains("AMERICA") ||
    upper.contains(" AM") ||
    upper.starts_with("AM") ||
    upper.contains("|AM")
}

/// Check if a name/category is English VOD content
pub fn is_english_vod(name: &str) -> bool {
    let upper = name.to_uppercase();
    // Keywords for English
    upper.contains("ENGLISH") || 
    upper.contains("|EN|") || 
    upper.contains(" EN") ||
    upper.starts_with("EN") ||
    upper.contains("-EN")
}

/// Parse a category name to extract metadata
pub fn parse_category(name: &str) -> ParsedCategory {
    let original = name.to_string();
    let mut display_name = name.to_string();
    let mut country: Option<String> = None;
    let mut quality: Option<Quality> = None;
    let mut content_type: Option<ContentType> = None;
    let mut is_vip = false;

    // Detect country/region prefix patterns
    let country_patterns = [
        ("US|", "US"),
        ("US |", "US"),
        ("AM |", "US"),
        ("AM|", "US"),
        ("UK|", "UK"),
        ("UK |", "UK"),
        ("EU |", "EU"),
        ("EU|", "EU"),
        ("FR|", "FR"),
        ("FR |", "FR"),
        ("CA|", "CA"),
        ("CA |", "CA"),
        ("DE|", "DE"),
        ("DE |", "DE"),
        ("ES|", "ES"),
        ("ES |", "ES"),
        ("IT|", "IT"),
        ("IT |", "IT"),
        ("VIP |", "VIP"),
        ("VIP|", "VIP"),
        ("4K|", "4K"),
        ("4K |", "4K"),
    ];

    for (pattern, code) in country_patterns {
        if name.to_uppercase().starts_with(pattern) {
            country = Some(code.to_string());
            display_name = name[pattern.len()..].trim().to_string();
            if code == "VIP" {
                is_vip = true;
            }
            break;
        }
    }

    // Also check for ▎ separator (Promax style)
    if country.is_none() {
        if let Some(pos) = name.find('▎') {
            let prefix = name[..pos].trim().to_uppercase();
            if [
                "UK", "US", "EU", "FR", "CA", "VIP", "SPORTS", "PPV", "MULTI",
            ]
            .contains(&prefix.as_str())
            {
                country = Some(prefix.clone());
                display_name = name[pos + "▎".len()..].trim().to_string();
                if prefix == "VIP" {
                    is_vip = true;
                }
            }
        }
    }

    // Detect quality
    let upper = name.to_uppercase();
    if upper.contains("4K")
        || upper.contains("ᵁᴴᴰ")
        || upper.contains("³⁸⁴⁰")
        || upper.contains("UHD")
    {
        quality = Some(Quality::UHD4K);
    } else if upper.contains("FHD") || upper.contains("1080") {
        quality = Some(Quality::FHD);
    } else if upper.contains("HD") || upper.contains("ᴴᴰ") {
        quality = Some(Quality::HD);
    } else if upper.contains("SD") || upper.contains("LQ") {
        quality = Some(Quality::SD);
    }

    // Detect content type
    if upper.contains("SPORT") {
        content_type = Some(ContentType::Sports);
    } else if upper.contains("NEWS") {
        content_type = Some(ContentType::News);
    } else if upper.contains("MOVIE") || upper.contains("CINEMA") {
        content_type = Some(ContentType::Movies);
    } else if upper.contains("KID") || upper.contains("ENFANT") {
        content_type = Some(ContentType::Kids);
    } else if upper.contains("MUSIC") {
        content_type = Some(ContentType::Music);
    } else if upper.contains("DOCUMENTARY")
        || upper.contains("DOCUMENTAIRE")
        || upper.contains("DOC")
    {
        content_type = Some(ContentType::Documentary);
    } else if upper.contains("ENTERTAINMENT") {
        content_type = Some(ContentType::Entertainment);
    } else if upper.contains("RELIGIOUS") || upper.contains("BIBLICAL") {
        content_type = Some(ContentType::Religious);
    } else if upper.contains("PPV") || upper.contains("PAY PER VIEW") {
        content_type = Some(ContentType::PPV);
    } else if upper.contains("GENERAL") {
        content_type = Some(ContentType::General);
    }

    // Check VIP in content
    if upper.contains("VIP") {
        is_vip = true;
    }

    ParsedCategory {
        original_name: original,
        country,
        quality,
        content_type,
        display_name,
        is_vip,
    }
}

use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use regex::Regex;

/// Parsed stream/channel with extracted metadata
#[derive(Debug, Clone)]
pub struct ParsedStream {
    pub original_name: String,
    pub display_name: String,
    pub country: Option<String>,
    pub quality: Option<Quality>,
    pub is_separator: bool,
    pub is_live_event: bool,
    pub location: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub stop_time: Option<DateTime<Utc>>,
    pub sports_event: Option<crate::sports::SportsEvent>,
}

/// Parse a stream/channel name to extract metadata
pub fn parse_stream(name: &str, provider_tz: Option<&str>) -> ParsedStream {
    let original = name.to_string();
    let mut display_name = name.to_string();
    let mut country: Option<String> = None;
    let mut quality: Option<Quality> = None;
    let mut is_separator = false;
    let mut is_live_event = false;
    let mut location: Option<String> = None;
    let mut start_time: Option<DateTime<Utc>> = None;
    let mut stop_time: Option<DateTime<Utc>> = None;
    let mut sports_event: Option<crate::sports::SportsEvent> = None;

    // Check if it's a separator line
    let trimmed = name.trim();
    if (trimmed.starts_with("####") || trimmed.starts_with("═══"))
        && (trimmed.ends_with("####") || trimmed.ends_with("═══"))
    {
        is_separator = true;
        display_name = trimmed
            .trim_matches('#')
            .trim_matches('═')
            .trim()
            .to_string();
    }

    // Detect country/region prefix patterns
    let country_patterns = [
        ("US|", "US"),
        ("US |", "US"),
        ("UK|", "UK"),
        ("UK |", "UK"),
        ("FR|", "FR"),
        ("FR |", "FR"),
        ("CA|", "CA"),
        ("CA |", "CA"),
        ("4K|", "4K"),
        ("4K |", "4K"),
        ("CHRITMAS|", "XMAS"),
        ("CHRISTMAS|", "XMAS"),
    ];

    for (pattern, code) in country_patterns {
        if name.to_uppercase().starts_with(pattern) {
            country = Some(code.to_string());
            display_name = name[pattern.len()..].trim().to_string();
            break;
        }
    }

    // Detect quality
    let upper = name.to_uppercase();
    if upper.contains("4K") || upper.contains("ᵁᴴᴰ") || upper.contains("UHD") {
        quality = Some(Quality::UHD4K);
    } else if upper.contains("FHD") {
        quality = Some(Quality::FHD);
    } else if upper.contains("ᴴᴰ") || upper.contains(" HD") {
        quality = Some(Quality::HD);
    }

    // Check for live event markers
    if upper.contains("[LIVE") || upper.contains("LIVE-EVENT") || upper.contains("[EVENT") {
        is_live_event = true;
    }

    // --- TIME PARSING ---
    // Look for patterns like:
    // 14:00
    // [14:00]
    // 19:30 CET
    // 12/10 16:00
    // (19:00)

    // Regex for Time: HH:MM (required), optional DD/MM before, optional am/pm after, optional TZ after
    // Capture groups: 1=DD(opt), 2=MM(opt), 3=HH, 4=MM, 5=am/pm(opt), 6=TZ(opt)
    let re = Regex::new(r"(?i)(?:(\d{1,2})[/.[:punct:]](\d{1,2})\s+)?\(?\[?(\d{1,2})[:.](\d{2})\s*(am|pm)?\]?\)?\s*([A-Z]{2,4})?").unwrap();

    // We only try to parse time if it looks like a live event or sports channel to avoid false positives in VOD titles
    if is_live_event || upper.contains("SPORT") || upper.contains("VS") {
        if let Some(caps) = re.captures(&display_name) {
            let now = Utc::now();
            let current_year = now.year();

            let day = caps
                .get(1)
                .map_or(now.day(), |m| m.as_str().parse().unwrap_or(now.day()));
            let month = caps
                .get(2)
                .map_or(now.month(), |m| m.as_str().parse().unwrap_or(now.month()));
            let mut hour: u32 = caps.get(3).unwrap().as_str().parse().unwrap_or(0);
            let minute: u32 = caps.get(4).unwrap().as_str().parse().unwrap_or(0);
            let am_pm = caps.get(5).map(|m| m.as_str().to_lowercase());
            let tz_str = caps.get(6).map(|m| m.as_str()).unwrap_or("");

            if let Some(am_pm_val) = am_pm {
                if am_pm_val == "pm" && hour < 12 {
                    hour += 12;
                } else if am_pm_val == "am" && hour == 12 {
                    hour = 0;
                }
            }

            // Construct NaiveDateTime
            if let Some(naive_date) = NaiveDate::from_ymd_opt(current_year, month, day) {
                if let Some(naive_time) = NaiveTime::from_hms_opt(hour, minute, 0) {
                    let naive_dt = NaiveDateTime::new(naive_date, naive_time);

                    // Determine Source Timezone
                    let source_tz: chrono_tz::Tz = match tz_str.to_uppercase().as_str() {
                        "CET" | "MEZ" => chrono_tz::Europe::Paris,
                        "GMT" | "BST" | "UK" => chrono_tz::Europe::London,
                        "ET" | "EST" | "EDT" => chrono_tz::America::New_York,
                        "PT" | "PST" | "PDT" => chrono_tz::America::Los_Angeles,
                        _ => {
                            // Try provider timezone first
                            if let Some(ptz) = provider_tz {
                                if let Ok(tz) = ptz.parse::<chrono_tz::Tz>() {
                                    tz
                                } else {
                                    // Fallback to country logic
                                    if let Some(c) = &country {
                                        match c.as_str() {
                                            "US" => chrono_tz::America::New_York,
                                            "CA" => chrono_tz::America::Toronto,
                                            "FR" => chrono_tz::Europe::Paris,
                                            "DE" => chrono_tz::Europe::Berlin,
                                            _ => chrono_tz::Europe::London,
                                        }
                                    } else {
                                        chrono_tz::Europe::London
                                    }
                                }
                            } else if let Some(c) = &country {
                                match c.as_str() {
                                    "US" => chrono_tz::America::New_York,
                                    "CA" => chrono_tz::America::Toronto,
                                    "FR" => chrono_tz::Europe::Paris,
                                    "DE" => chrono_tz::Europe::Berlin,
                                    _ => chrono_tz::Europe::London,
                                }
                            } else {
                                chrono_tz::Europe::London
                            }
                        }
                    };

                    // Convert to UTC
                    if let Some(dt) = source_tz.from_local_datetime(&naive_dt).single() {
                        // If the parsed time is way in the past (> 24h), maybe it is next year?
                        // Or if detected 'day' is < current day, maybe it's next year (e.g. 01/01 parsed in Dec).
                        // For now, assume current year is safe for typical EPG style names.

                        // Fix: if we defaulted to today's date but the time has passed significantly,
                        // usually these streams are for UPCOMING events.
                        // But if we parsed a specific date, stick to it.
                        // If we didn't parse a date, and the time is < now - 4 hours, maybe it's tomorrow?
                        // Actually, sticking to "Today" is safest for [HH:MM] format.

                        start_time = Some(dt.with_timezone(&Utc));

                        // Clean the name: Remove the time string
                        display_name = display_name
                            .replace(caps.get(0).unwrap().as_str(), "")
                            .trim()
                            .to_string();
                        // Clean extra brackets
                        display_name = display_name
                            .replace("[]", "")
                            .replace("()", "")
                            .trim()
                            .to_string();
                    }
                }
            }
        }
    }

    // Try to extract location (keep existing logic)
    if let Some(start) = display_name.find('(') {
        if let Some(end) = display_name.find(')') {
            if end > start {
                let loc = display_name[start + 1..end].to_string();
                if loc.len() < 20 && !loc.contains("LIVE") && !loc.contains(':') {
                    // Exclude timestamps
                    location = Some(loc);
                }
            }
        }
    }

    // Clean up display name
    let clean_display = display_name
        .replace("ᴴᴰ", "")
        .replace("ᵁᴴᴰ", "")
        .replace("³⁸⁴⁰ᴾ", "")
        .replace("⁶⁰ᶠᵖˢ", "")
        .replace("ᴿᴬᵂ", "")
        .replace("H265", "")
        .replace("HEVC", "")
        .replace("RAW", "")
        .replace("[LIVE]", "") // Remove these after parsing
        .replace("LIVE-EVENT", "")
        .replace("[]", "")
        .replace("()", "")
        .trim()
        .to_string();

    if !clean_display.is_empty() {
        display_name = clean_display;
    }

    // --- SPORTS EVENT PARSING ---
    if let Some(event) = crate::sports::parse_sports_event(&display_name) {
        sports_event = Some(event.clone());
        is_live_event = true;

        // If we have a sports event, we can try to get a better start_time if not already set
        if start_time.is_none() && !event.start_time_raw.is_empty() {
            let parsed_dt = if let Ok(dt) =
                NaiveDateTime::parse_from_str(&event.start_time_raw, "%Y-%m-%d %H:%M:%S")
            {
                Some(dt)
            } else if let Ok(t) = NaiveTime::parse_from_str(
                &event.start_time_raw.to_lowercase().replace(" ", ""),
                "%I:%M%p",
            ) {
                let now_local = if let Some(ptz) = provider_tz {
                    let tz: chrono_tz::Tz = ptz.parse().unwrap_or(chrono_tz::UTC);
                    Utc::now().with_timezone(&tz)
                } else {
                    Utc::now().with_timezone(&chrono_tz::UTC)
                };
                Some(NaiveDateTime::new(now_local.date_naive(), t))
            } else {
                None
            };

            if let Some(naive_dt) = parsed_dt {
                // Triangulate timezone for sports strings
                let source_tz: chrono_tz::Tz = if let Some(ptz) = provider_tz {
                    ptz.parse::<chrono_tz::Tz>().unwrap_or(chrono_tz::UTC)
                } else {
                    chrono_tz::UTC
                };

                start_time = Some(
                    source_tz
                        .from_local_datetime(&naive_dt)
                        .single()
                        .unwrap_or_else(|| Utc::now().with_timezone(&source_tz))
                        .with_timezone(&Utc),
                );
            }
        }
    }

    // Backup: If start_time is still None, try to find a raw 'start: YYYY-MM-DD' in the name anyway
    if start_time.is_none() {
        let st_re =
            regex::Regex::new(r"(?i)start:\s*(\d{4}-\d{2}-\d{2}\s\d{2}:\d{2}:\d{2})").unwrap();
        if let Some(caps) = st_re.captures(&display_name) {
            if let Ok(naive_dt) =
                NaiveDateTime::parse_from_str(caps.get(1).unwrap().as_str(), "%Y-%m-%d %H:%M:%S")
            {
                let source_tz = provider_tz
                    .and_then(|ptz| ptz.parse::<chrono_tz::Tz>().ok())
                    .unwrap_or(chrono_tz::UTC);
                start_time = Some(
                    source_tz
                        .from_local_datetime(&naive_dt)
                        .single()
                        .unwrap_or_else(|| Utc::now().with_timezone(&source_tz))
                        .with_timezone(&Utc),
                );
            }
        }
    }

    // Parse stop time if present (Strong8K format: stop:YYYY-MM-DD HH:MM:SS)
    let stop_re =
        regex::Regex::new(r"(?i)stop:\s*(\d{4}-\d{2}-\d{2}\s\d{2}:\d{2}:\d{2})").unwrap();
    if let Some(caps) = stop_re.captures(&display_name) {
        if let Ok(naive_dt) =
            NaiveDateTime::parse_from_str(caps.get(1).unwrap().as_str(), "%Y-%m-%d %H:%M:%S")
        {
            let source_tz = provider_tz
                .and_then(|ptz| ptz.parse::<chrono_tz::Tz>().ok())
                .unwrap_or(chrono_tz::UTC);
            stop_time = Some(
                source_tz
                    .from_local_datetime(&naive_dt)
                    .single()
                    .unwrap_or_else(|| Utc::now().with_timezone(&source_tz))
                    .with_timezone(&Utc),
            );
        }
    }

    ParsedStream {
        original_name: original,
        display_name,
        country,
        quality,
        is_separator,
        is_live_event,
        location,
        start_time,
        stop_time,
        sports_event,
    }
}

/// Streaming source/platform
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StreamingSource {
    Netflix,
    Disney,
    Apple,
    Amazon,
    HBO,
    Paramount,
    Peacock,
    Hulu,
    Other,
}

impl StreamingSource {
    pub fn icon(&self) -> &'static str {
        match self {
            StreamingSource::Netflix => "🔴",
            StreamingSource::Disney => "🏰",
            StreamingSource::Apple => "🍎",
            StreamingSource::Amazon => "📦",
            StreamingSource::HBO => "🎬",
            StreamingSource::Paramount => "⭐",
            StreamingSource::Peacock => "🦚",
            StreamingSource::Hulu => "💚",
            StreamingSource::Other => "",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            StreamingSource::Netflix => Color::Red,
            StreamingSource::Disney => Color::LightBlue,
            StreamingSource::Apple => Color::White,
            StreamingSource::Amazon => Color::Yellow,
            StreamingSource::HBO => Color::Magenta,
            StreamingSource::Paramount => Color::Cyan,
            StreamingSource::Peacock => Color::Green,
            StreamingSource::Hulu => Color::LightGreen,
            StreamingSource::Other => Color::White,
        }
    }
}

/// Parsed VOD category
#[derive(Debug, Clone)]
pub struct ParsedVodCategory {
    pub original_name: String,
    pub display_name: String,
    pub language: Option<String>,
    pub streaming_source: Option<StreamingSource>,
    pub quality: Option<Quality>,
    pub is_kids: bool,
}

/// Parse a VOD category name
pub fn parse_vod_category(name: &str) -> ParsedVodCategory {
    let original = name.to_string();
    let mut display_name = name.to_string();
    let mut language: Option<String> = None;
    let mut streaming_source: Option<StreamingSource> = None;
    let mut quality: Option<Quality> = None;
    let mut is_kids = false;

    // Detect language prefix patterns
    let lang_patterns = [
        ("|EN|", "EN"),
        ("|EN| ", "EN"),
        ("EN ▎", "EN"),
        ("EN -", "EN"),
        ("|FR|", "FR"),
        ("|FR| ", "FR"),
        ("FR ▎", "FR"),
        ("|DE|", "DE"),
        ("|ES|", "ES"),
        ("|IT|", "IT"),
        ("VOD |", "VOD"),
        ("VOD | ", "VOD"),
    ];

    for (pattern, lang) in lang_patterns {
        if name.to_uppercase().starts_with(pattern) || name.contains(pattern) {
            language = Some(lang.to_string());
            display_name = name.replace(pattern, "").trim().to_string();
            break;
        }
    }

    // Also check ▎ separator
    if language.is_none() {
        if let Some(pos) = name.find('▎') {
            let prefix = name[..pos].trim().to_uppercase();
            if ["EN", "FR", "DE", "ES", "IT", "NL"].contains(&prefix.as_str()) {
                language = Some(prefix);
                display_name = name[pos + "▎".len()..].trim().to_string();
            }
        }
    }

    // Detect streaming source
    let upper = name.to_uppercase();
    if upper.contains("NETFLIX") {
        streaming_source = Some(StreamingSource::Netflix);
    } else if upper.contains("DISNEY") {
        streaming_source = Some(StreamingSource::Disney);
    } else if upper.contains("APPLE") {
        streaming_source = Some(StreamingSource::Apple);
    } else if upper.contains("AMAZON") || upper.contains("PRIME VIDEO") {
        streaming_source = Some(StreamingSource::Amazon);
    } else if upper.contains("HBO") {
        streaming_source = Some(StreamingSource::HBO);
    } else if upper.contains("PARAMOUNT") {
        streaming_source = Some(StreamingSource::Paramount);
    } else if upper.contains("PEACOCK") {
        streaming_source = Some(StreamingSource::Peacock);
    } else if upper.contains("HULU") {
        streaming_source = Some(StreamingSource::Hulu);
    }

    // Detect quality
    if upper.contains("4K") || upper.contains("⁴ᴷ") || upper.contains("UHD") {
        quality = Some(Quality::UHD4K);
    } else if upper.contains("BLURAY") {
        quality = Some(Quality::FHD);
    }

    // Detect kids content
    if upper.contains("KIDS") || upper.contains("CHILDREN") {
        is_kids = true;
    }

    ParsedVodCategory {
        original_name: original,
        display_name,
        language,
        streaming_source,
        quality,
        is_kids,
    }
}

/// Parsed movie/VOD item
#[derive(Debug, Clone)]
pub struct ParsedMovie {
    pub original_name: String,
    pub title: String,
    pub year: Option<u16>,
    pub language: Option<String>,
    pub quality: Option<Quality>,
    pub has_multi_sub: bool,
    pub streaming_source: Option<StreamingSource>,
    pub rating: Option<String>,
}

/// Parse a movie/VOD item name
pub fn parse_movie(name: &str) -> ParsedMovie {
    let original = name.to_string();
    let mut title = name.to_string();
    let mut year: Option<u16> = None;
    let mut language: Option<String> = None;
    let mut quality: Option<Quality> = None;
    let mut has_multi_sub = false;
    let streaming_source: Option<StreamingSource> = None;

    // Detect language prefix: "EN -", "EN ▎", etc.
    let lang_patterns = [
        ("EN - ", "EN"),
        ("EN ▎", "EN"),
        ("EN- ", "EN"),
        ("FR - ", "FR"),
        ("FR ▎", "FR"),
        ("DE - ", "DE"),
        ("ES - ", "ES"),
        ("TOP - ", "TOP"),
        ("NL ▎", "NL"),
    ];

    for (pattern, lang) in lang_patterns {
        if name.starts_with(pattern) {
            language = Some(lang.to_string());
            title = name[pattern.len()..].trim().to_string();
            break;
        }
    }

    // Extract year from title: (2024), (1995), etc.
    // We use a loop to remove ALL instances of (YYYY) from the title if we find one
    let mut found_year = None;
    if let Some(caps) = Regex::new(r"\((\d{4})\)").unwrap().captures(&title) {
        if let Some(m) = caps.get(1) {
            if let Ok(y) = m.as_str().parse::<u16>() {
                if y >= 1900 && y <= 2030 {
                    found_year = Some(y);
                }
            }
        }
    }

    if let Some(y) = found_year {
        year = Some(y);
        // Remove all (YYYY) patterns from title to deduplicate
        let year_regex = Regex::new(r"\(\d{4}\)").unwrap();
        title = year_regex.replace_all(&title, "").trim().to_string();
    }

    // Detect quality
    let upper = name.to_uppercase();
    if upper.contains("4K") || upper.contains("UHD") {
        quality = Some(Quality::UHD4K);
    } else if upper.contains("BLURAY") || upper.contains("BLU-RAY") {
        quality = Some(Quality::FHD);
    } else if upper.contains("HD") {
        quality = Some(Quality::HD);
    }

    // Detect multi-sub
    if upper.contains("MULTI-SUB") || upper.contains("MULTI SUB") || upper.contains("[MULTI") {
        has_multi_sub = true;
        // Clean from title
        title = title
            .replace("[MULTI-SUB]", "")
            .replace("[MULTI SUB]", "")
            .trim()
            .to_string();
    }

    // Clean up common markers from title
    title = title
        .replace("(MULTI SUB)", "")
        .replace("(MULTI-SUB)", "")
        .replace("(PORTUGUESE ENG-SUB)", "")
        .replace("(FHD)", "")
        .replace("(HD)", "")
        .replace("(4K)", "")
        .replace("ᴴᴰ", "")
        .replace("⁴ᴷ", "")
        .replace(" UHD", "")
        .replace(" FHD", "")
        .replace(" HD", "")
        .trim()
        .to_string();

    ParsedMovie {
        original_name: original,
        title,
        year,
        language,
        quality,
        has_multi_sub,
        streaming_source,
        rating: None,
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;



    #[test]
    fn test_parse_us_category() {
        let parsed = parse_category("US| SPORTS NETWORK");
        assert_eq!(parsed.country, Some("US".to_string()));
        assert_eq!(parsed.display_name, "SPORTS NETWORK");
        assert_eq!(parsed.content_type, Some(ContentType::Sports));
    }

    #[test]
    fn test_parse_4k_category() {
        let parsed = parse_category("4K| RELAX ᵁᴴᴰ ³⁸⁴⁰ᴾ");
        assert_eq!(parsed.country, Some("4K".to_string()));
        assert_eq!(parsed.quality, Some(Quality::UHD4K));
    }

    #[test]
    fn test_parse_promax_style() {
        let parsed = parse_category("UK ▎GENERAL");
        assert_eq!(parsed.country, Some("UK".to_string()));
        assert_eq!(parsed.display_name, "GENERAL");
    }

    #[test]
    fn test_parse_sports_event() {
        let name =
            "NBA 01: Hawks (ATL) x Bulls (CHI) start:2025-12-21 20:20:00 stop:2025-12-21 23:00:00";
        let parsed = parse_stream(name, None);
        assert!(parsed.sports_event.is_some());
        let event = parsed.sports_event.as_ref().unwrap();
        assert_eq!(event.team1, "Hawks");
        assert_eq!(event.team1_abbr, Some("ATL".to_string()));
        assert_eq!(event.team2, "Bulls");
        assert_eq!(event.team2_abbr, Some("CHI".to_string()));
        assert_eq!(event.start_time_raw, "2025-12-21 20:20:00");
    }

    #[test]
    fn test_parse_sports_event_no_abbr() {
        let name = "FOOTBALL: Arsenal x Chelsea start:2025-12-21 15:00:00";
        let parsed = parse_stream(name, None);
        assert!(parsed.sports_event.is_some());
        let event = parsed.sports_event.as_ref().unwrap();
        assert_eq!(event.team1, "Arsenal");
        assert_eq!(event.team2, "Chelsea");
    }

    #[test]
    fn test_parse_sports_event_vs() {
        let name = "UFC 299: O'Malley vs Vera start:2025-12-21 22:00:00";
        let parsed = parse_stream(name, None);
        assert!(parsed.sports_event.is_some());
        let event = parsed.sports_event.as_ref().unwrap();
        assert_eq!(event.team1, "O'Malley");
        assert_eq!(event.team2, "Vera");
    }

    #[test]
    fn test_parse_sports_event_mls() {
        let name = "MLS: Inter Miami (MIA) x LAFC (LAFC) start:2025-12-21 19:30:00";
        let parsed = parse_stream(name, None);
        assert!(parsed.sports_event.is_some());
        let event = parsed.sports_event.as_ref().unwrap();
        assert_eq!(event.team1, "Inter Miami");
        assert_eq!(event.team1_abbr, Some("MIA".to_string()));
    }

    #[test]
    fn test_parse_sports_event_at() {
        let name = "NFL: Cowboys @ Eagles [16:00]";
        let parsed = parse_stream(name, None);
        assert!(parsed.sports_event.is_some());
        let event = parsed.sports_event.as_ref().unwrap();
        assert_eq!(event.team1, "Cowboys");
        assert_eq!(event.team2, "Eagles");
    }

    #[test]
    fn test_parse_sports_event_short_time() {
        let name = "UK| LIVE FOOTBALL 01 [EVENT ONLY] Aston Villa vs Sheffield Utd 12:30pm";
        let parsed = parse_stream(name, None);
        assert!(parsed.sports_event.is_some());
        let event = parsed.sports_event.as_ref().unwrap();
        assert_eq!(event.team1, "Aston Villa");
        assert_eq!(event.team2, "Sheffield Utd");
        assert!(parsed.start_time.is_some());
    }

    #[test]
    fn test_parse_timezone_triangulation() {
        let name = "SPORT: Team A x Team B [20:00]";
        // Provider is in New York (EST)
        let parsed = parse_stream(name, Some("America/New_York"));
        assert!(parsed.start_time.is_some());

        let st = parsed.start_time.unwrap();
        // 20:00 EST is definitely not 20:00 UTC
        assert!(st.hour() != 20);
    }
}
