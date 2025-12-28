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
static CLEAN_U_PREFIX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)^[\W_]*u[\s\u{00A0}\u{200B}]*").unwrap());

static CLEAN_PREFIX_COMBINED: Lazy<Regex> = Lazy::new(|| {
    // Order matters: Longest matches first to avoid partial replacements (e.g. USA vs US)
    Regex::new(r"(?i)^(?:UNITED\s+STATES|UNITED\s+KINGDOM|ENGLISH|AMERICA|USA|US|UK|CA|EN/CAM|EN)(?:\s*[-|:/]\s*)").unwrap()
});

static CLEAN_BRACKETS_COMBINED: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\s*[\(\[\{]\s*(?:UNITED\s+STATES|UNITED\s+KINGDOM|ENGLISH|AMERICA|USA|US|UK|CA|EN)\s*[\)\]\}]").unwrap()
});

static CLEAN_END_COMBINED: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\s+(?:UNITED\s+STATES|UNITED\s+KINGDOM|ENGLISH|AMERICA|USA|US|UK|CA|EN)\s*$").unwrap()
});

static CLEAN_START_COMBINED: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^(?:UNITED\s+STATES|UNITED\s+KINGDOM|ENGLISH|AMERICA|USA|US|UK|CA|EN)\s+").unwrap()
});

// Standalone USA mentions (e.g. "USA Sports")
static CLEAN_USA_STANDALONE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\bUSA\b\s*[-|:]*\s*").unwrap());

static CLEAN_TRAILING_PIPE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s*\|\s*$").unwrap());
static CLEAN_LEADING_PIPE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*\|\s*").unwrap());
static CLEAN_MULTI_PIPE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s*\|\s*\|+").unwrap());
static CLEAN_TRAILING_HYPHEN: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+-\s*$").unwrap());
static CLEAN_LEADING_HYPHEN: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*-\s+").unwrap());
static CLEAN_MULTI_SPACE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());

// Combined regex for foreign patterns in is_american_live
static FOREIGN_PATTERNS_REGEX: Lazy<Regex> = Lazy::new(|| {
    let patterns = [
        r"(?i)AR\s*\|", r"(?i)\|AR\|", r"(?i)\sAR\s", r"(?i)ARAB", r"(?i)ARABIC",
        r"(?i)SA\s*\|", r"(?i)\|SA\|", r"(?i)SAUDI", r"(?i)KSA",
        r"(?i)AE\s*\|", r"(?i)\|AE\|", r"(?i)UAE", r"(?i)EMIRATES",
        r"(?i)QA\s*\|", r"(?i)\|QA\|", r"(?i)QATAR",
        r"(?i)KW\s*\|", r"(?i)\|KW\|", r"(?i)KUWAIT",
        r"(?i)IR\s*\|", r"(?i)\|IR\|", r"(?i)IRAN", r"(?i)PERSIAN",
        r"(?i)AF\s*\|", r"(?i)\|AF\|", r"(?i)AFGHAN",
        r"(?i)IL\s*\|", r"(?i)\|IL\|", r"(?i)ISRAEL",
        r"(?i)TR\s*\|", r"(?i)\|TR\|", r"(?i)TURK", r"(?i)TURKEY",
        r"(?i)IN\s*\|", r"(?i)\|IN\|", r"(?i)INDIA", r"(?i)INDIAN", r"(?i)HINDI", r"(?i)PUNJABI", r"(?i)TAMIL", r"(?i)TELUGU", r"(?i)MALAYALAM", r"(?i)KANNADA", r"(?i)MARATHI", r"(?i)BENGALI",
        r"(?i)PK\s*\|", r"(?i)\|PK\|", r"(?i)PAKISTAN", r"(?i)URDU",
        r"(?i)BD\s*\|", r"(?i)\|BD\|", r"(?i)BANGLA", r"(?i)BANGLADESH",
        r"(?i)CN\s*\|", r"(?i)\|CN\|", r"(?i)CHINA", r"(?i)CHINESE", r"(?i)MANDARIN", r"(?i)CANTONESE",
        r"(?i)JP\s*\|", r"(?i)\|JP\|", r"(?i)JAPAN",
        r"(?i)KR\s*\|", r"(?i)\|KR\|", r"(?i)KOREA",
        r"(?i)PH\s*\|", r"(?i)\|PH\|", r"(?i)PHILIPPINES", r"(?i)FILIPINO", r"(?i)PINOY",
        r"(?i)VN\s*\|", r"(?i)\|VN\|", r"(?i)VIETNAM",
        r"(?i)TH\s*\|", r"(?i)\|TH\|", r"(?i)THAILAND",
        r"(?i)ID\s*\|", r"(?i)\|ID\|", r"(?i)INDONESIA",
        r"(?i)MY\s*\|", r"(?i)\|MY\|", r"(?i)MALAYSIA",
        r"(?i)ASIA\s*\|", r"(?i)\|ASIA\|", r"(?i)\sASIA", 
        r"(?i)AM\s*\|", r"(?i)\|AM\|", r"(?i)ARMENIA", r"(?i)ARMENIAN",
        r"(?i)KH\s*\|", r"(?i)KURDISH", r"(?i)KURD",
        r"(?i)AZ\s*\|", r"(?i)AZERBAIJAN",
        r"(?i)GE\s*\|", r"(?i)GEORGIA",
        r"(?i)HK\s*\|", r"(?i)HONG KONG",
        r"(?i)AFRICA", r"(?i)NIGERIA", r"(?i)KENYA", r"(?i)SOMALIA", r"(?i)ZA\s*\|", r"(?i)SOUTH AFRICA", r"(?i)MAROC", r"(?i)MOROCCO", r"(?i)TUNISIA", r"(?i)ALGERIA", r"(?i)EGYPT",
        r"(?i)UK\s*\|", r"(?i)\|UK\|", r"(?i)\sUK\s", r"(?i)UNITED KINGDOM", r"(?i)BRITISH",
        r"(?i)IE\s*\|", r"(?i)\|IE\|", r"(?i)\sIE\s", r"(?i)IRELAND", r"(?i)IRISH",
        r"(?i)SC\s*\|", r"(?i)\|SC\|", r"(?i)SCOTLAND", r"(?i)SPFL",
        r"(?i)FR\s*\|", r"(?i)\|FR\|", r"(?i)\sFR\s", r"(?i)FRANCE", r"(?i)FRENCH",
        r"(?i)DE\s*\|", r"(?i)\|DE\|", r"(?i)\sDE\s", r"(?i)GERMAN", r"(?i)GERMANY", r"(?i)DEUTSCH",
        r"(?i)IT\s*\|", r"(?i)\|IT\|", r"(?i)\sIT\s", r"(?i)ITALY", r"(?i)ITALIAN",
        r"(?i)ES\s*\|", r"(?i)\|ES\|", r"(?i)\sES\s", r"(?i)SPAIN", r"(?i)SPANISH", r"(?i)ESPANA", r"(?i)LATINO",
        r"(?i)PT\s*\|", r"(?i)\|PT\|", r"(?i)\sPT\s", r"(?i)PORTUGAL", r"(?i)PORTUGUESE", r"(?i)BRAZIL", r"(?i)BR\s*\|",
        r"(?i)NL\s*\|", r"(?i)\|NL\|", r"(?i)\sNL\s", r"(?i)DUTCH", r"(?i)NETHERLANDS",
        r"(?i)BE\s*\|", r"(?i)\|BE\|", r"(?i)BELGIUM",
        r"(?i)PL\s*\|", r"(?i)\|PL\|", r"(?i)\sPL\s", r"(?i)POLAND", r"(?i)POLISH",
        r"(?i)RU\s*\|", r"(?i)\|RU\|", r"(?i)\sRU\s", r"(?i)RUSSIA", r"(?i)RUSSIAN",
        r"(?i)GR\s*\|", r"(?i)\|GR\|", r"(?i)\sGR\s", r"(?i)GREECE", r"(?i)GREEK",
        r"(?i)AL\s*\|", r"(?i)\|AL\|", r"(?i)ALBANIA", r"(?i)ALBANIAN", r"(?i)SHQIP",
        r"(?i)RO\s*\|", r"(?i)\|RO\|", r"(?i)ROMANIA", r"(?i)ROMANIAN",
        r"(?i)BG\s*\|", r"(?i)\|BG\|", r"(?i)BULGARIA", r"(?i)BULGARIAN",
        r"(?i)HU\s*\|", r"(?i)\|HU\|", r"(?i)HUNGARY", r"(?i)HUNGARIAN", r"(?i)MAGYAR",
        r"(?i)CZ\s*\|", r"(?i)\|CZ\|", r"(?i)CZECH", r"(?i)CS\s*\|",
        r"(?i)SK\s*\|", r"(?i)\|SK\|", r"(?i)SLOVAKIA",
        r"(?i)HR\s*\|", r"(?i)\|HR\|", r"(?i)CROATIA", r"(?i)HRVATSKA",
        r"(?i)RS\s*\|", r"(?i)\|RS\|", r"(?i)SERBIA", r"(?i)SRPSKI",
        r"(?i)BA\s*\|", r"(?i)\|BA\|", r"(?i)BOSNIA",
        r"(?i)MK\s*\|", r"(?i)\|MK\|", r"(?i)MACEDONIA",
        r"(?i)SI\s*\|", r"(?i)\|SI\|", r"(?i)SLOVENIA",
        r"(?i)EX-YU", r"(?i)BALKAN", r"(?i)YUGOSLAVIA", r"(?i)CG\s*\|", r"(?i)MONTENEGRO",
        r"(?i)CY\s*\|", r"(?i)\|CY\|", r"(?i)CYPRUS",
        r"(?i)MT\s*\|", r"(?i)\|MT\|", r"(?i)MALTA",
        r"(?i)UA\s*\|", r"(?i)\|UA\|", r"(?i)UKRAINE", r"(?i)UKRAINIAN",
        r"(?i)AT\s*\|", r"(?i)\|AT\|", r"(?i)AUSTRIA", r"(?i)AUSTRIAN",
        r"(?i)CH\s*\|", r"(?i)\|CH\|", r"(?i)SWISS", r"(?i)SWITZERLAND",
        r"(?i)SE\s*\|", r"(?i)\|SE\|", r"(?i)SWEDEN", r"(?i)SWEDISH",
        r"(?i)DK\s*\|", r"(?i)\|DK\|", r"(?i)DENMARK", r"(?i)DANISH",
        r"(?i)NO\s*\|", r"(?i)\|NO\|", r"(?i)NORWAY", r"(?i)NORWEGIAN",
        r"(?i)FI\s*\|", r"(?i)\|FI\|", r"(?i)FINLAND", r"(?i)FINNISH",
        r"(?i)IS\s*\|", r"(?i)ICELAND",
        r"(?i)AU\s*\|", r"(?i)\|AU\|", r"(?i)AUSTRALIA",
        r"(?i)NZ\s*\|", r"(?i)\|NZ\|", r"(?i)NEW ZEALAND",
        r"(?i)LATAM", r"(?i)SUR AMERICA", r"(?i)ARGENTINA", r"(?i)COLOMBIA", r"(?i)CHILE", r"(?i)PERU", r"(?i)VENEZUELA", r"(?i)BOLIVIA", r"(?i)ECUADOR", r"(?i)URUGUAY", r"(?i)PARAGUAY", r"(?i)CR\s*\|", r"(?i)CARIBBEAN",
        r"(?i)CANADA", r"(?i)CA\s*\|", r"(?i)\sC\s*\|",
        r"(?i)XXX", r"(?i)ADULT", r"(?i)18\+", r"(?i)PORN",
    ];
    Regex::new(&patterns.join("|")).unwrap()
});

static FOREIGN_VOD_KEYWORDS_REGEX: Lazy<Regex> = Lazy::new(|| {
    let keywords = [
        r"FRANCE", r"FRENCH", r"INDIA", r"INDIAN", r"HINDI", r"TURKISH", r"TURK", 
        r"ARABIC", r"ARAB", r"SPANISH", r"LATINO", r"GERMAN", r"ITALIAN", 
        r"PORTUGUESE", r"RUSSIAN", r"CHINESE", r"KOREAN", r"JAPANESE",
        r"POLISH", r"DUTCH", r"SWEDISH", r"DANISH", r"NORWEGIAN"
    ];
    Regex::new(&format!(r"(?i){}", keywords.join("|"))).unwrap()
});


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
            ContentType::Sports => "\u{26be}", // Default sports
            ContentType::PPV => "\u{1f3df}",    // Stadium/Event
            _ => "",
        }
    }
}

/// Get color for country/region code
pub fn country_color(country: &str) -> Color {
    match country.to_uppercase().as_str() {
        "US" | "USA" | "AM" | "NBA" | "NFL" | "MLB" | "NHL" | "UFC" | "SPORTS" | "PPV" => Color::Rgb(0, 255, 255),
        "UK" | "GB" | "EU" => Color::Rgb(57, 255, 20),
        _ => Color::White,
    }
}

/// Get flag emoji for country
pub fn country_flag(country: &str) -> &'static str {
    match country.to_uppercase().as_str() {
        "US" | "USA" | "AM" | "NBA" | "NFL" | "MLB" | "NHL" | "UFC" | "SPORTS" | "PPV" => "🇺🇸",
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
    
    if FOREIGN_PATTERNS_REGEX.is_match(&n) {
        return false;
    }

    // Default: Allow everything else (Exclusion-based filtering)
    true
}


/// Clean up names in American Mode by removing redundant labels
/// Uses pre-compiled static regexes for performance
pub fn clean_american_name(name: &str) -> String {
    // Normalize special separators first
    let cleaned = name.replace("▎", "|");
    
    // Remove all @ symbols globally, BOM, and hidden characters
    let cleaned = cleaned.replace("@", "")
                        .replace("\u{feff}", "")
                        .replace("\u{200b}", "");

    // Chain replacements to minimize intermediate String allocations
    let cleaned = CLEAN_U_PREFIX.replace(&cleaned, "");
    let cleaned = CLEAN_PREFIX_COMBINED.replace_all(&cleaned, "");
    let cleaned = CLEAN_BRACKETS_COMBINED.replace_all(&cleaned, " ");
    let cleaned = CLEAN_END_COMBINED.replace_all(&cleaned, "");
    let cleaned = CLEAN_START_COMBINED.replace_all(&cleaned, "");
    
    // Final cleanup: remove redundant pipes, hyphens, and double spaces
    let cleaned = CLEAN_TRAILING_PIPE.replace_all(&cleaned, "");
    let cleaned = CLEAN_LEADING_PIPE.replace_all(&cleaned, "");
    let cleaned = CLEAN_MULTI_PIPE.replace_all(&cleaned, " |");
    let cleaned = CLEAN_TRAILING_HYPHEN.replace_all(&cleaned, "");
    let cleaned = CLEAN_LEADING_HYPHEN.replace_all(&cleaned, "");
    let cleaned = CLEAN_MULTI_SPACE.replace_all(&cleaned, " ");
    
    // Remove any remaining standalone USA mentions
    let cleaned = CLEAN_USA_STANDALONE.replace_all(&cleaned, "");
    
    // Extra cleanup for leading/trailing colons and dots
    let cleaned_str = cleaned.trim_start_matches(|c: char| c == ':' || c == '.' || c == '|' || c == '-' || c == ' ')
                      .trim_end_matches(|c: char| c == ':' || c == '.' || c == '|' || c == '-' || c == ' ')
                      .trim();

    if cleaned_str.is_empty() {
        return name.to_string();
    }

    cleaned_str.to_string()
}

/// Check if a name/category is English VOD content
pub fn is_english_vod(name: &str) -> bool {
    let upper = name.to_uppercase();
    
    // If it explicitly matches foreign patterns, it's not English
    if FOREIGN_VOD_KEYWORDS_REGEX.is_match(&upper) || FOREIGN_PATTERNS_REGEX.is_match(&upper) {
        return false;
    }
    
    // For VOD, we assume it is English unless proven otherwise by foreign markers,
    // as movie titles rarely contain "EN" prefixes unlike Live TV channels.
    true
}

/// Check if a name/category is UK live content
pub fn is_uk_live(name: &str) -> bool {
    let n = name.to_uppercase();
    n.contains("UK |") || n.contains("|UK|") || n.contains("UNITED KINGDOM") || n.contains(" BRITISH ") || n.contains("[UK]") || n.contains("(UK)")
}

/// Check if a name/category is Canadian live content
pub fn is_ca_live(name: &str) -> bool {
    let n = name.to_uppercase();
    n.contains("CA |") || n.contains("|CA|") || n.contains("CANADA") || n.contains("CANADIAN") || n.contains("[CA]") || n.contains("(CA)")
}

/// Check if a name/category is English (US/UK/CA) live content
pub fn is_english_live(name: &str) -> bool {
    is_american_live(name) || is_uk_live(name) || is_ca_live(name)
}

/// Check if a name/category is Sports content
pub fn is_sports_content(name: &str) -> bool {
    let n = name.to_uppercase();
    n.contains("SPORT") || n.contains("FOOTBALL") || n.contains("SOCCER") || n.contains("BASKETBALL") || 
    n.contains("NBA") || n.contains("NFL") || n.contains("MLB") || n.contains("NHL") || n.contains("UFC") || 
    n.contains("BOXING") || n.contains("WRESTLING") || n.contains("WWE") || n.contains("AEW") || 
    n.contains("CRICKET") || n.contains("RUGBY") || n.contains("GOLF") || n.contains("TENNIS") || 
    n.contains("RACING") || n.contains("F1") || n.contains("MOTOGP") || n.contains("DAZN") || 
    n.contains("BEIN") || n.contains("SKY SPORTS") || n.contains("TNT SPORTS") || n.contains("BT SPORT") ||
    n.contains("PPV") || n.contains("PEACOCK") || n.contains("ESPN") || n.contains("BALLY") || n.contains("YES NETWORK")
}

/// Parse a category name to extract metadata
pub fn parse_category(name: &str) -> ParsedCategory {
    let original = name.to_string();
    let mut country: Option<String> = None;
    let mut quality: Option<Quality> = None;
    let mut content_type: Option<ContentType> = None;
    let mut is_vip = false;

    // Clean control chars and 'u ' prefix early
    let mut display_name = name.replace(|c: char| c.is_control(), "");
    let re_u = Regex::new(r"(?i)^[\W_]*u\s+").unwrap();
    display_name = re_u.replace(&display_name, "").trim().to_string();

    // Generic Country/Sports Prefix Detection (e.g. "US |", "NBA:", "UK-", "NBA PASS", "S |")
    let re_prefix = Regex::new(r"(?i)^([A-Z0-9]{1,5})(?:\s*[|:-]\s*|\s+)").unwrap();
    if let Some(caps) = re_prefix.captures(&display_name.to_uppercase()) {
        let code = caps.get(1).unwrap().as_str();
        let allowed = ["S", "US", "USA", "AM", "UK", "GB", "CA", "EU", "FR", "DE", "ES", "IT", "VIP", "NBA", "NFL", "MLB", "NHL", "UFC", "PPV", "EN"];
        if allowed.contains(&code) {
             // If it's just a single char category marker like 'S |', we just strip it and don't set country
             if code != "S" {
                country = Some(code.to_string());
             }
             
             if let Some(pos) = display_name.find(|c| c == '|' || c == ':' || c == '-') {
                 display_name = display_name[pos + 1..].trim().to_string();
             } else if let Some(pos) = display_name.find(' ') {
                 display_name = display_name[pos + 1..].trim().to_string();
             }
             if code == "VIP" { is_vip = true; }
        }
    }

    // Also check for ▎ separator (Promax style)
    if country.is_none() {
        if let Some(pos) = display_name.find('▎') {
            let prefix = display_name[..pos].trim().to_uppercase();
            if ["UK", "US", "EU", "FR", "CA", "VIP", "SPORTS", "PPV", "MULTI"].contains(&prefix.as_str())
            {
                country = Some(prefix.clone());
                display_name = display_name[pos + "▎".len()..].trim().to_string();
                if prefix == "VIP" { is_vip = true; }
            }
        }
    }

    // Detect quality
    let upper = display_name.to_uppercase();
    if upper.contains("4K") || upper.contains("ᵁᴴᴰ") || upper.contains("³⁸⁴⁰") || upper.contains("UHD") {
        quality = Some(Quality::UHD4K);
    } else if upper.contains("FHD") || upper.contains("1080") {
        quality = Some(Quality::FHD);
    } else if upper.contains("HD") || upper.contains("ᴴᴰ") {
        quality = Some(Quality::HD);
    } else if upper.contains("SD") || upper.contains("LQ") {
        quality = Some(Quality::SD);
    }

    // Detect content type
    if upper.contains("SPORT") || ["NBA", "NFL", "MLB", "NHL", "UFC", "F1"].iter().any(|s| upper.contains(s)) {
        content_type = Some(ContentType::Sports);
    } else if upper.contains("NEWS") {
        content_type = Some(ContentType::News);
    } else if upper.contains("MOVIE") || upper.contains("CINEMA") {
        content_type = Some(ContentType::Movies);
    } else if upper.contains("KID") || upper.contains("ENFANT") {
        content_type = Some(ContentType::Kids);
    } else if upper.contains("MUSIC") {
        content_type = Some(ContentType::Music);
    } else if upper.contains("DOCUMENTARY") || upper.contains("DOCUMENTAIRE") || upper.contains("DOC") {
        content_type = Some(ContentType::Documentary);
    } else if upper.contains("GENERAL") {
        content_type = Some(ContentType::General);
    }

    // Aggressive cleaning for display_name
    let mut cleaned = display_name;
    let suffixes = [
        "(PPV)", "[PPV]", "PPV", "(USA)", "[USA]", "USA", "(UK)", "[UK]", "UK",
        "(CA)", "[CA]", "CA", "(VIP)", "[VIP]", "VIP", "(4K)", "[4K]", "4K",
        " - ", " | ", " : ",
    ];
    for s in suffixes {
        let upper_c = cleaned.to_uppercase();
        if upper_c.ends_with(s) {
            cleaned = cleaned[..cleaned.len() - s.len()].trim().to_string();
        } else if upper_c.starts_with(s) {
            cleaned = cleaned[s.len()..].trim().to_string();
        }
    }
    
    // Final cleanup: strip leading/trailing pipes, colons, dashes
    cleaned = cleaned.trim_start_matches(|c: char| c == '|' || c == ':' || c == '-' || c.is_whitespace())
                     .trim_end_matches(|c: char| c == '|' || c == ':' || c == '-' || c.is_whitespace())
                     .to_string();
    
    display_name = cleaned;

    if upper.contains("VIP") { is_vip = true; }

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
    pub channel_prefix: Option<String>,
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
    let mut channel_prefix: Option<String> = None;

    // Check if it's a separator line
    let trimmed = name.trim();
    if (trimmed.starts_with("####") || trimmed.starts_with("═══"))
        && (trimmed.ends_with("####") || trimmed.ends_with("═══"))
    {
        is_separator = true;
        display_name = trimmed.trim_matches('#').trim_matches('═').trim().to_string();
    }

    // Capture leading channel numbers (e.g. "01:", "15-", "03 ")
    // STRICT MODE: If colon is used, it MUST be followed by space to distinguish from timestamps (8:20PM)
    let re_chan = Regex::new(r"^\s*([0-9]+)(?:\s*:\s+|\s*[\-|]\s*|\s+)").unwrap();
    if let Some(caps) = re_chan.captures(&display_name) {
        channel_prefix = Some(caps.get(1).unwrap().as_str().trim().to_string());
        // Remove the channel prefix from display_name to avoid duplicate display
        display_name = re_chan.replace(&display_name, "").to_string();
    }

    // Check for "u " prefix (specific to Mega OTT and others)
    let mut clean_loop = true;
    while clean_loop {
        clean_loop = false;
        let start_len = display_name.len();
        
        // Remove "u " prefix and hidden characters
        display_name = CLEAN_U_PREFIX.replace(&display_name, "").to_string();
        
        // Remove "MNF", "TNF", "SNF" game day labels AND league prefixes ONLY when followed by separator or number
        // We want to keep "NBA TV" intact but strip "NBA | Game" or "NBA 01: Something"
        let re_mnf = Regex::new(r"(?i)^(MNF|TNF|SNF|SLING|S)(?:\s*[:|-]\s*|\s+)").unwrap();
        if re_mnf.is_match(&display_name) {
             display_name = re_mnf.replace(&display_name, "").to_string();
             clean_loop = true;
        }
        // Strip league prefix ONLY if followed by separator (:|) or number, NOT regular words like "TV"
        let re_league_prefix = Regex::new(r"(?i)^(NBA|NFL|NHL|MLB|UFC|MLS)(?:\s*[:|-]\s*|\s+\d)").unwrap();
        if re_league_prefix.is_match(&display_name) {
             display_name = re_league_prefix.replace(&display_name, "").to_string();
             clean_loop = true;
        }

        if display_name.len() < start_len {
            clean_loop = true;
        }
    }

    // Second pass: Check for channel numbers again after prefix cleaning (e.g. "UFC | 05")
    if channel_prefix.is_none() {
        if let Some(caps) = re_chan.captures(&display_name) {
             channel_prefix = Some(caps.get(1).unwrap().as_str().trim().to_string());
             display_name = re_chan.replace(&display_name, "").to_string();
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

    // Detect and strip country/region prefix patterns (Aggressive Generic)
    // BUT: Preserve league names like "NBA TV" - only strip if followed by separator or number
    let mut clean_loop = true;
    while clean_loop {
        clean_loop = false;
        let re_prefix = Regex::new(r"(?i)^([A-Z0-9/]{1,7})(?:\s*[|:-]\s*|\s+)").unwrap();
        if let Some(caps) = re_prefix.captures(&display_name) {
            let code = caps.get(1).unwrap().as_str().to_uppercase();
            // Country codes that should always be stripped
            let country_codes = ["S", "US", "USA", "AM", "UK", "GB", "CA", "EN", "EN/CAM", "EU", "FR", "DE", "ES", "IT", "VIP", "PPV"];
            let league_codes = ["NBA", "NFL", "MLB", "NHL", "UFC", "MLS"];
            
            // Normalize special characters like dashes into standard ones before prefix check
            let check_name = display_name.replace(" - ", " ").replace("-", " ").to_uppercase();
            
            if country_codes.iter().any(|&c| check_name.starts_with(c)) {
                 display_name = re_prefix.replace(&display_name, "").to_string();
                 // Extra trim to remove following dashes if any
                 display_name = display_name.trim_start_matches(|c: char| c == '-' || c == '|' || c == ':' || c == ' ').to_string();
                 clean_loop = true;
            } else if league_codes.contains(&code.as_str()) {
                 // For leagues, only strip if followed by separator (: | -) or number, NOT regular words
                 let re_league = Regex::new(r"(?i)^(NBA|NFL|NHL|MLB|UFC|MLS)(?:\s*[|:-]\s*|\s+\d)").unwrap();
                 if re_league.is_match(&display_name) {
                      if country.is_none() {
                         country = Some(code);
                      }
                      display_name = re_league.replace(&display_name, "").to_string();
                      clean_loop = true;
                 }
            }
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
    
    // 0. Explicit 'start:' tag parsing (Priority 1)
    // This allows us to capture full timestamps before cleanup nukes them
    if let Some(caps) = START_TIME_REGEX.captures(&display_name) {
        if let Ok(naive_dt) = NaiveDateTime::parse_from_str(caps.get(1).unwrap().as_str(), "%Y-%m-%d %H:%M:%S") {
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
            // Remove the start tag from name
            display_name = display_name.replace(caps.get(0).unwrap().as_str(), "").trim().to_string();
        }
    }

    // 0b. Explicit 'stop:' tag parsing (Priority 1)
    // Same rationale: capture stop times before cleanup nukes them
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
            // Remove the stop tag from name
            display_name = display_name.replace(caps.get(0).unwrap().as_str(), "").trim().to_string();
        }
    }

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
                        // Priority Check: Only set start_time if NOT already set by START_TIME_REGEX
                        if start_time.is_none() {
                             start_time = Some(dt.with_timezone(&Utc));
                        }

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

    // Fallback: LOOSE TIME PARSING (e.g. "8PM", "12/27 8PM") - When colon is missing
    // We try this if strict TIME_REGEX failed
    if start_time.is_none() && (is_live_event || upper.contains("SPORT") || upper.contains("VS")) {
        let re_loose = Regex::new(r"(?i)(?:(\d{1,2})[/.[:punct:]](\d{1,2})\s+)?(\d{1,2})\s*(am|pm)").unwrap();
        if let Some(caps) = re_loose.captures(&display_name) {
            let now = Utc::now();
            let current_year = now.year();

            // Date capture
            let d1 = caps.get(1).map(|m| m.as_str().parse::<u32>().unwrap_or(0));
            let d2 = caps.get(2).map(|m| m.as_str().parse::<u32>().unwrap_or(0));
            
            // Hour/AMPM
            let mut hour: u32 = caps.get(3).unwrap().as_str().parse().unwrap_or(0);
            let am_pm = caps.get(4).unwrap().as_str().to_lowercase();
            
            if am_pm == "pm" && hour < 12 {
                hour += 12;
            } else if am_pm == "am" && hour == 12 {
                hour = 0;
            }

            // Try to resolve date: US (MM/DD) priority for Trex/English, then DD/MM
            let mut naive_date = NaiveDate::from_ymd_opt(current_year, now.month(), now.day());
            
            if let (Some(v1), Some(v2)) = (d1, d2) {
                if v1 > 0 && v2 > 0 {
                    // Try MM/DD first (v1=Month, v2=Day)
                    if let Some(nd) = NaiveDate::from_ymd_opt(current_year, v1, v2) {
                        naive_date = Some(nd);
                    } else if let Some(nd) = NaiveDate::from_ymd_opt(current_year, v2, v1) {
                         // Fallback DD/MM
                        naive_date = Some(nd);
                    }
                }
            }

            if let Some(nd) = naive_date {
                 if let Some(naive_time) = NaiveTime::from_hms_opt(hour, 0, 0) {
                     let naive_dt = NaiveDateTime::new(nd, naive_time);
                     
                     // Use Provider TZ (default generic USA/Europe logic)
                     let source_tz: chrono_tz::Tz = if let Some(ptz) = provider_tz {
                         ptz.parse().unwrap_or(chrono_tz::America::Chicago) // Default to Cental if fail
                     } else {
                         chrono_tz::Europe::London
                     };

                     if let Some(dt) = source_tz.from_local_datetime(&naive_dt).single() {
                         start_time = Some(dt.with_timezone(&Utc));
                         // Clean match
                         display_name = display_name.replace(caps.get(0).unwrap().as_str(), "").trim().to_string();
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
    let mut clean_display = display_name
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
        .replace("[MULTI-SUB]", "")
        .replace("[MULTISUB]", "")
        .replace("[MULTI-AUDIO]", "")
        .replace("[MULTIAUDIO]", "")
        .replace("[MULTI-LANG]", "")
        .replace("[MULTILANG]", "")
        .replace("[MULTI]", "")
        .replace("[]", "")
        .replace("()", "")
        .replace("  ", " ") // Quick double space fix
        .trim()
        .to_string();

    // Aggressively strip date/time artifacts from name (e.g. "12/27", "8PM", "30PM")
    // We rely on the app's standardized timedisplay [Tomorrow 09:30 AM] instead.
    // Explicitly check for start of string (^ pattern) to catch "30PM Texans"
    let re_time_junk = Regex::new(r"(?i)(?:\b\d{1,2}/\d{1,2}(?:/\d{2,4})?\b|(?:\b|^)\d{1,2}:\d{2}(?:\s*[AP]M)?\b|(?:\b|^)\d{1,2}\s*[AP]M\b)").unwrap();
    clean_display = re_time_junk.replace_all(&clean_display, "").to_string();

    // Cleanup whitespace left gaps
    let re_spaces = Regex::new(r"\s+").unwrap();
    clean_display = re_spaces.replace_all(&clean_display, " ").trim().to_string();

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
            // Clean the name
            display_name = display_name.replace(caps.get(0).unwrap().as_str(), "").trim().to_string();
        }
    }

    // Final attempt to capture leading channel numbers if not already found
    if channel_prefix.is_none() {
        let re_chan_final = Regex::new(r"^\s*(\d+)[\s:|x-]*").unwrap();
        if let Some(caps) = re_chan_final.captures(&display_name) {
            let num = caps.get(1).unwrap().as_str();
            channel_prefix = Some(num.to_string());
            // Optionally clean it from display name if it's there
            let raw_prefix = caps.get(0).unwrap().as_str();
            display_name = display_name.replace(raw_prefix, "").trim().to_string();
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
        channel_prefix,
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
