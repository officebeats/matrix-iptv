use ratatui::style::Color;
use regex::Regex;
use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use once_cell::sync::Lazy;

// Pre-compiled regexes for performance - only compiled once
static TIME_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(?:(\d{1,2})[/.[:punct:]](\d{1,2})\s+)?\(?\[?(\d{1,2})[:.](\\d{2})\s*(am|pm)?\]?\)?\s*([A-Z]{2,4})?").unwrap()
});
static START_TIME_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)start:\s*(\d{4}-\d{2}-\d{2}\s\d{2}:\d{2}:\d{2})").unwrap()
});
static STOP_TIME_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)stop:\s*(\d{4}-\d{2}-\d{2}\s\d{2}:\d{2}:\d{2})").unwrap()
});
static YEAR_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\((\d{4})\)").unwrap()
});
static YEAR_STRIP_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\(\d{4}\)").unwrap()
});

// American Mode cleaning regexes - pre-compiled and combined for performance
static CLEAN_U_PREFIX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)^[\W_]*u\s+").unwrap());

static CLEAN_PREFIX_COMBINED: Lazy<Regex> = Lazy::new(|| {
    // Order matters: Longest matches first to avoid partial replacements (e.g. USA vs US)
    Regex::new(r"(?i)^(?:UNITED\s+STATES|ENGLISH|AMERICA|USA|US|EN)(?:\s*[-|:]\s*)").unwrap()
});

static CLEAN_BRACKETS_COMBINED: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\s*[\(\[\{]\s*(?:UNITED\s+STATES|ENGLISH|AMERICA|USA|US|EN)\s*[\)\]\}]").unwrap()
});

static CLEAN_END_COMBINED: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\s+(?:UNITED\s+STATES|ENGLISH|AMERICA|USA|US|EN)\s*$").unwrap()
});

static CLEAN_START_COMBINED: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^(?:UNITED\s+STATES|ENGLISH|AMERICA|USA|US|EN)\s+").unwrap()
});

// Standalone USA mentions (e.g. "USA Sports")
static CLEAN_USA_STANDALONE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\bUSA\b\s*[-|:]*\s*").unwrap());

static CLEAN_TRAILING_PIPE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s*\|\s*$").unwrap());
static CLEAN_LEADING_PIPE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*\|\s*").unwrap());
static CLEAN_MULTI_PIPE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s*\|\s*\|+").unwrap());
static CLEAN_TRAILING_HYPHEN: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+-\s*$").unwrap());
static CLEAN_LEADING_HYPHEN: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*-\s+").unwrap());
static CLEAN_MULTI_SPACE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());


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
            Quality::UHD4K => Color::Rgb(255, 0, 255), // Neon Magenta
            Quality::FHD => Color::Rgb(57, 255, 20),   // Neon Green
            Quality::HD => Color::Rgb(0, 255, 255),    // Bright Cyan
            Quality::SD => Color::White,               // Safe White
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
            _ => "",
        }
    }
}

/// Get color for country/region code
pub fn country_color(country: &str) -> Color {
    match country.to_uppercase().as_str() {
        "US" | "USA" | "AM" => Color::Rgb(0, 255, 255),
        "UK" | "GB" | "EU" => Color::Rgb(57, 255, 20),
        "FR" | "FRANCE" => Color::Rgb(255, 105, 180),
        "CA" | "CANADA" => Color::Rgb(255, 69, 0), // Orange Red
        "VIP" => Color::Rgb(255, 255, 0),
        "4K" => Color::Rgb(255, 0, 255),
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
        _ => "",
    }
}

/// Check if a name/category is American live content
// Generic Country Prefix Regex (e.g. "AZ |", "BR |", "C |")
static GENERIC_COUNTRY_PREFIX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)^([A-Z]{1,3})\s*[|:]").unwrap());

/// Check if a name/category is American live content
pub fn is_american_live(name: &str) -> bool {
    // Normalize special separators first
    let n = name.to_uppercase().replace("▎", "|").replace("︳", "|");

    // 0. Strict Blocker for known international junk that slips through (e.g. "AR|")
    // This handles cases where a prefix like Arabic (AR) is used without a space.
    if (n.starts_with("AR |") || n.starts_with("AR|") || n.starts_with("AR :") || n.starts_with("AR:")) && !n.contains("USA") {
        return false;
    }

    // 1. Prefix Blocker (First line of defense)
    // If it starts with "XX |" or "XXX |", we verify if XX is explicitly a US or Sports marker.
    // If not, we block it unless the name contains a strong US keyword (like "USA").
    if let Some(caps) = GENERIC_COUNTRY_PREFIX.captures(&n) {
        if let Some(match_str) = caps.get(1) {
             let p = match_str.as_str();
             // Whitelist strict US/Sports/Quality prefixes
             let allowed_prefixes = [
                 "US", "USA", "VIP", "4K", "3D", "XXX", "PPV", 
                 "NBA", "NFL", "UFC", "MLB", "NHL", "F1", "UHD", "FHD", "RAW"
             ];
             
             if !allowed_prefixes.contains(&p) {
                 // It has a prefix like "UK", "CA", "FR". 
                 // We block it UNLESS it explicitly mentions USA inside the name.
                 if !n.contains("USA") && !n.contains("US LOCALS") && !n.contains("AMERICA") && !n.contains("UNITED STATES") {
                    return false; 
                 }
             }
        }
    }

    // 2. Explicit Positive check
    // If it passed the prefix check (or has no prefix), we check if it's definitively American.
    let positive_keywords = [
        "USA", "U.S.A", " US ", "[US]", "(US)", "|US|", "AMERICA", "UNITED STATES", "LOCAL", "LOCALS", 
        "NFL", "NBA", "MLB", "NHL", "NCAA", "ESPN", "BALLY", "YES NETWORK", "MSG", "ABC", "NBC", "CBS", "FOX",
        "4K", "UHD", "FHD", "VIP", "PPV", "UFC"
    ];
    if positive_keywords.iter().any(|&k| n.contains(k)) || n.starts_with("US ") || n.starts_with("US|") || n.starts_with("US:") || n.starts_with("US-") {
        return true;
    }
    
    // 2. Explicit exclusions for international/foreign content (Backup for non-prefixed items)
    let foreign_patterns = [
        // Arabic/Middle East/Asia
        "AR |", "AR|", "|AR|", " AR ", "ARAB", "ARABIC",
        "SA |", "SA|", "|SA|", "SAUDI", "KSA",
        "AE |", "AE|", "|AE|", "UAE", "EMIRATES",
        "QA |", "QA|", "|QA|", "QATAR",
        "KW |", "KW|", "|KW|", "KUWAIT",
        "IR |", "IR|", "|IR|", "IRAN", "PERSIAN",
        "AF |", "AF|", "|AF|", "AFGHAN",
        "IL |", "IL|", "|IL|", "ISRAEL",
        "TR |", "TR|", "|TR|", "TURK", "TURKEY",
        "IN |", "IN|", "|IN|", "INDIA", "INDIAN", "HINDI", "PUNJABI", "TAMIL", "TELUGU", "MALAYALAM", "KANNADA", "MARATHI", "BENGALI",
        "PK |", "PK|", "|PK|", "PAKISTAN", "URDU",
        "BD |", "BD|", "|BD|", "BANGLA", "BANGLADESH",
        "CN |", "CN|", "|CN|", "CHINA", "CHINESE", "MANDARIN", "CANTONESE",
        "JP |", "JP|", "|JP|", "JAPAN",
        "KR |", "KR|", "|KR|", "KOREA",
        "PH |", "PH|", "|PH|", "PHILIPPINES", "FILIPINO", "PINOY",
        "VN |", "VN|", "|VN|", "VIETNAM",
        "TH |", "TH|", "|TH|", "THAILAND",
        "ID |", "ID|", "|ID|", "INDONESIA",
        "MY |", "MY|", "|MY|", "MALAYSIA",
        "ASIA |", "ASIA|", "|ASIA|", " ASIA", 
        "AM |", "AM|", "|AM|", "ARMENIA", "ARMENIAN",
        "KH |", "KH|", "KURDISH", "KURD",
        "AZ |", "AZ|", "AZERBAIJAN",
        "GE |", "GE|", "GEORGIA",
        "HK |", "HK|", "HONG KONG",

        // Africa
        "AFRICA", "NIGERIA", "KENYA", "SOMALIA", "ZA |", "ZA|", "SOUTH AFRICA", "MAROC", "MOROCCO", "TUNISIA", "ALGERIA", "EGYPT",

        // Europe (Extended)
        "UK |", "UK|", "|UK|", " UK ", "UNITED KINGDOM", "BRITISH",
        "IE |", "IE|", "|IE|", " IE ", "IRELAND", "IRISH",
        "SC |", "SC|", "|SC|", "SCOTLAND", "SPFL",
        "FR |", "FR|", "|FR|", " FR ", "FRANCE", "FRENCH",
        "DE |", "DE|", "|DE|", " DE ", "GERMAN", "GERMANY", "DEUTSCH",
        "IT |", "IT|", "|IT|", " IT ", "ITALY", "ITALIAN",
        "ES |", "ES|", "|ES|", " ES ", "SPAIN", "SPANISH", "ESPANA", "LATINO",
        "PT |", "PT|", "|PT|", " PT ", "PORTUGAL", "PORTUGUESE", "BRAZIL", "BR |", "BR|",
        "NL |", "NL|", "|NL|", " NL ", "DUTCH", "NETHERLANDS",
        "BE |", "BE|", "|BE|", "BELGIUM",
        "PL |", "PL|", "|PL|", " PL ", "POLAND", "POLISH",
        "RU |", "RU|", "|RU|", " RU ", "RUSSIA", "RUSSIAN",
        "GR |", "GR|", "|GR|", " GR ", "GREECE", "GREEK",
        "AL |", "AL|", "|AL|", "ALBANIA", "ALBANIAN", "SHQIP",
        "RO |", "RO|", "|RO|", "ROMANIA", "ROMANIAN",
        "BG |", "BG|", "|BG|", "BULGARIA", "BULGARIAN",
        "HU |", "HU|", "|HU|", "HUNGARY", "HUNGARIAN", "MAGYAR",
        "CZ |", "CZ|", "|CZ|", "CZECH", "CS |", "CS|",
        "SK |", "SK|", "|SK|", "SLOVAKIA",
        "HR |", "HR|", "|HR|", "CROATIA", "HRVATSKA",
        "RS |", "RS|", "|RS|", "SERBIA", "SRPSKI",
        "BA |", "BA|", "|BA|", "BOSNIA",
        "MK |", "MK|", "|MK|", "MACEDONIA",
        "SI |", "SI|", "|SI|", "SLOVENIA",
        "EX-YU", "BALKAN", "YUGOSLAVIA", "CG |", "CG|", "MONTENEGRO",
        "CY |", "CY|", "|CY|", "CYPRUS",
        "MT |", "MT|", "|MT|", "MALTA",
        "UA |", "UA|", "|UA|", "UKRAINE", "UKRAINIAN",
        "AT |", "AT|", "|AT|", "AUSTRIA", "AUSTRIAN",
        "CH |", "CH|", "|CH|", "SWISS", "SWITZERLAND",
        "SE |", "SE|", "|SE|", "SWEDEN", "SWEDISH",
        "DK |", "DK|", "|DK|", "DENMARK", "DANISH",
        "NO |", "NO|", "|NO|", "NORWAY", "NORWEGIAN",
        "FI |", "FI|", "|FI|", "FINLAND", "FINNISH",
        "IS |", "IS|", "ICELAND",

        // Oceania
        "AU |", "AU|", "|AU|", "AUSTRALIA",
        "NZ |", "NZ|", "|NZ|", "NEW ZEALAND",

        // Latin/South America (Generic)
        "LATAM", "SUR AMERICA", "ARGENTINA", "COLOMBIA", "CHILE", "PERU", "VENEZUELA", "BOLIVIA", "ECUADOR", "URUGUAY", "PARAGUAY", "CR |", "CR|", "CARIBBEAN",
        
        // Canada
        "CANADA", "CA |", "CA|", " C |", "C |", 
        
        // Adult
        "XXX", "ADULT", "18+", "PORN",
    ];
    
    for pattern in foreign_patterns {
        if n.contains(pattern) {
            return false;
        }
    }

    // Default: Allow everything else (Exclusion-based filtering)
    true
}


/// Clean up names in American Mode by removing redundant labels
/// Uses pre-compiled static regexes for performance
pub fn clean_american_name(name: &str) -> String {
    // Normalize special separators first
    let mut cleaned = name.replace("▎", "|");
    
    // Remove all @ symbols globally
    cleaned = cleaned.replace("@", "");

    // Remove BOM and common hidden characters
    cleaned = cleaned.replace("\u{feff}", "").replace("\u{200b}", "");

    // Remove "u " prefix using pre-compiled regex (handles Mega OTT)
    cleaned = CLEAN_U_PREFIX.replace(&cleaned, "").to_string();
    
    // Remove common leading language prefixes
    cleaned = CLEAN_PREFIX_COMBINED.replace_all(&cleaned, "").to_string();
    
    // Remove keywords in brackets: (USA), [USA], {USA}, (EN), etc.
    cleaned = CLEAN_BRACKETS_COMBINED.replace_all(&cleaned, " ").to_string();
    
    // Remove keywords at end/start of string
    cleaned = CLEAN_END_COMBINED.replace_all(&cleaned, "").to_string();
    cleaned = CLEAN_START_COMBINED.replace_all(&cleaned, "").to_string();
    
    // Final cleanup: remove redundant pipes, hyphens, and double spaces
    cleaned = CLEAN_TRAILING_PIPE.replace_all(&cleaned, "").to_string();
    cleaned = CLEAN_LEADING_PIPE.replace_all(&cleaned, "").to_string();
    cleaned = CLEAN_MULTI_PIPE.replace_all(&cleaned, " |").to_string();
    cleaned = CLEAN_TRAILING_HYPHEN.replace_all(&cleaned, "").to_string();
    cleaned = CLEAN_LEADING_HYPHEN.replace_all(&cleaned, "").to_string();
    cleaned = CLEAN_MULTI_SPACE.replace_all(&cleaned, " ").to_string();
    
    // Remove any remaining standalone USA mentions
    cleaned = CLEAN_USA_STANDALONE.replace_all(&cleaned, "").to_string();
    
    // Extra cleanup for leading/trailing colons and dots
    cleaned = cleaned.trim_start_matches(|c: char| c == ':' || c == '.' || c == '|' || c == '-' || c == ' ')
                     .trim_end_matches(|c: char| c == ':' || c == '.' || c == '|' || c == '-' || c == ' ')
                     .trim()
                     .to_string();

    if cleaned.is_empty() {
        return name.to_string();
    }

    cleaned
}

/// Check if a name/category is English VOD content
pub fn is_english_vod(name: &str) -> bool {
    let upper = name.to_uppercase();
    
    // Explicit exclusions for foreign language content
    let foreign_keywords = ["FRANCE", "FRENCH", "INDIA", "INDIAN", "HINDI", "TURKISH", "TURK", 
                            "ARABIC", "ARAB", "SPANISH", "LATINO", "GERMAN", "ITALIAN", 
                            "PORTUGUESE", "RUSSIAN", "CHINESE", "KOREAN", "JAPANESE",
                            "POLISH", "DUTCH", "SWEDISH", "DANISH", "NORWEGIAN"];
    for kw in foreign_keywords {
        if upper.contains(kw) {
            return false;
        }
    }
    
    // Keywords for English
    upper.contains("ENGLISH") || 
    upper.contains("|EN|") || 
    upper.contains("| EN |") ||
    upper.contains("EN |") ||
    upper.starts_with("EN ") ||
    upper.starts_with("EN-") ||
    upper.starts_with("EN|") ||
    upper.contains("-EN-") ||
    upper.contains(" EN ") ||
    upper.ends_with(" EN") ||
    upper.ends_with("|EN")
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

    // Check for "u " prefix (specific to Mega OTT and others)
    // We do this loop to handle cases like "u u NFL" or "u [u] NFL"
    let mut clean_loop = true;
    while clean_loop {
        clean_loop = false;
        let start_len = display_name.len();
        
        // Remove "u " prefix
        if display_name.trim_start().to_lowercase().starts_with("u ") {
            if let Some(rest) = display_name.trim_start()[2..].trim_start().to_string().into() {
                display_name = rest;
                clean_loop = true;
            }
        }
        
        // Remove "u " with regex (for hidden chars)
        let re_u = Regex::new(r"(?i)^[\W_]*u\s+").unwrap();
        if re_u.is_match(&display_name) {
             display_name = re_u.replace(&display_name, "").to_string();
             clean_loop = true;
        }
        
        // Remove "MNF", "TNF", "SNF" from start if it's duplicated later
        // e.g. "MNF 8: Saints x Titans" -> "Saints x Titans"
        let re_mnf = Regex::new(r"(?i)^(MNF|TNF|SNF|NBA|NFL)\s+\d*[:|-]?\s*").unwrap();
        if re_mnf.is_match(&display_name) {
             display_name = re_mnf.replace(&display_name, "").to_string();
             clean_loop = true;
        }

        if display_name.len() < start_len {
            clean_loop = true;
        }
    }

    // Aggressive cleanup for pipe separators | often used to separate "Channel" from "Event"
    // Example: "NFL 01 - ... : NFL | 01 x 12/25 ..."
    // We want to discard the left side if it looks like channel info
    if let Some(idx) = display_name.rfind('|') {
        let suffix = display_name[idx + 1..].trim();
        // Heuristic: If suffix starts with a digit and "x" or "vs", it's likely the event part "01 x Team"
        // And the prefix is just channel spam.
        let re_event_start = Regex::new(r"(?i)^\d+\s*(x|vs|at|-)\s+").unwrap();
        if re_event_start.is_match(suffix) {
            display_name = suffix.to_string();
        } 
        // Heuristic 2: If the part before the pipe is super long compared to after? 
        // Or if the part after contains " x " or " vs "
        else if (suffix.contains(" x ") || suffix.contains(" vs ") || suffix.contains(" at ")) && suffix.len() > 5 {
             display_name = suffix.to_string();
        }
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
    // Using pre-compiled static TIME_REGEX for performance

    // We only try to parse time if it looks like a live event or sports channel to avoid false positives in VOD titles
    if is_live_event || upper.contains("SPORT") || upper.contains("VS") {
        if let Some(caps) = TIME_REGEX.captures(&display_name) {
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
        if let Some(caps) = START_TIME_REGEX.captures(&display_name) {
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
    if let Some(caps) = STOP_TIME_REGEX.captures(&display_name) {
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
            StreamingSource::Netflix => Color::Rgb(255, 50, 50),
            StreamingSource::Disney => Color::Rgb(100, 200, 255),
            StreamingSource::Apple => Color::White,
            StreamingSource::Amazon => Color::Rgb(255, 255, 0),
            StreamingSource::HBO => Color::Rgb(255, 0, 255),
            StreamingSource::Paramount => Color::Rgb(0, 255, 255),
            StreamingSource::Peacock => Color::Rgb(57, 255, 20),
            StreamingSource::Hulu => Color::Rgb(50, 255, 50),
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
    if let Some(caps) = YEAR_REGEX.captures(&title) {
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
        title = YEAR_STRIP_REGEX.replace_all(&title, "").trim().to_string();
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
        use chrono::Timelike;
        let name = "SPORT: Team A x Team B [20:00]";
        // Provider is in New York (EST)
        let parsed = parse_stream(name, Some("America/New_York"));
        assert!(parsed.start_time.is_some());

        let st = parsed.start_time.unwrap();
        // 20:00 EST is definitely not 20:00 UTC
        assert!(st.hour() != 20);
    }

    #[test]
    fn test_clean_american_name() {
        assert_eq!(clean_american_name("ALGERIE +6H USA"), "ALGERIE +6H");
        assert_eq!(clean_american_name("ENGLISH KIDS"), "KIDS");
        assert_eq!(clean_american_name("EN | Breaking Bad"), "Breaking Bad");
        assert_eq!(clean_american_name("Breaking Bad (US)"), "Breaking Bad");
        assert_eq!(clean_american_name("Breaking Bad [USA]"), "Breaking Bad");
        assert_eq!(clean_american_name("Breaking Bad - EN"), "Breaking Bad");
        assert_eq!(clean_american_name("USA: Movie Name"), "Movie Name");
        assert_eq!(clean_american_name("UNITED STATES - Movie"), "Movie");
    }

    #[test]
    fn test_redundancy_stripping_exact() {
        let input = "NFL 01 - 12/25 1PM Cowboys at Commanders: NFL | 01 x 12/25 1PM Cowboys at Commanders";
        let parsed = parse_stream(input, None);
        // Current logic strips before last pipe
        // Expected: "01 x 12/25 1PM Cowboys at Commanders"
        assert_eq!(parsed.display_name, "01 x 12/25 1PM Cowboys at Commanders");
    }

    #[test]
    fn test_redundancy_stripping_u_prefix() {
        let input = "u NFL 02 - 12/25 4: NFL | 02 x 12/25 [Today 10:30 AM]";
        let parsed = parse_stream(input, None);
        // "u " should be stripped, then pipe logic applies
        assert_eq!(parsed.display_name, "02 x 12/25 [Today 10:30 AM]");
    }
}
