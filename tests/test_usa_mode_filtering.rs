use matrix_iptv_lib::parser::{is_american_live, is_english_vod, clean_american_name};

#[test]
fn test_american_live_channels_allowed() {
    let american_cases = vec![
        "US: ESPN HD (60fps Live)",
        "USA | NBC Sports HD",
        "US - FOX News Live",
        "US | ABC East FHD",
        "US | CBS Sports Network",
        "US | HBO East 4K HDR",
        "US | Showtime East FHD",
        "US LOCALS: WABC New York 7",
        "AM | ABC News Live",
        "AMERICA | CNN News Desk",
        "UNITED STATES | Discovery Channel",
        "NBA TV 60FPS Live",
        "NFL RedZone (Commercial Free)",
        "MLB Network HD Live",
        "NHL Center Ice 1",
        "UFC 305 PPV Main Card",
        "NCAA Basketball Live",
        "VIP | Cinema 4K",
        "4K | FOX Sports 1 UHD",
        "24/7 Friends TV",
        "USA 24/7 ON-DEMAND",
        "USA 24/7 SEREIS",
        "USA ABC",
        "USA APPLE TV+",
        "USA BTN",
        "USA CBS",
        "USA CINEMA",
        "USA CINEMANIA",
        "USA CW",
        "USA DIRTVISION",
        "USA DOCUMENTARY",
        "USA ENTERTAINMENT",
        "USA ESPN",
        "USA ESPN PLUS",
        "USA FANDUEL",
        "USA NHL",
        "USA OHL",
        "USA PARAMOUNT",
        "USA PBS",
        "USA PEACOCK",
        "USA PLAY KIDZ",
        "USA PPV-LIVE EVENTS",
        "USA QMJHL",
        "USA SPECTRUM",
        "USA SPORTS",
        "USA STREAMED CHANNELS",
        "USA TELEMUNDO",
        "USA TRILLER TV",
        "USA UNIVISION",
        "USA VICTORY+",
        "USA VIX",
        "USA WHL",
        "F1 AND MOTOGP",
        "FIFA PLUS",
        "OLYMPICS",
        "PPV-SPORT EVENTS",
        "CHRISTMAS",
    ];

    for name in &american_cases {
        assert!(
            is_american_live(name),
            "Expected American content '{}' to be ALLOWED, but it was filtered OFF",
            name
        );
    }
}

#[test]
fn test_screenshot_foreign_categories_blocked() {
    let foreign_screenshot_cases = vec![
        // Canada
        "CANADA CINEMA",
        "CANADA DOCUMENTARY",
        "CANADA ENTERTAINMENT",
        "CANADA FRANÇAIS",
        "CANADA FRANÇAIS CINEMA",
        "CANADA FRANÇAIS DOCUMENTARY",
        "CANADA FRANÇAIS ENTRTENMENT",
        "CANADA FRANÇAIS SPORT",
        "CANADA GENERAL",
        "CANADA KIDS",
        "CANADA LIVE EVENT-PPV",
        "CANADA LOCAL CHANNELS",
        "CANADA MUSIC",
        "CANADA OTHERS",
        "CANADA SPORTS/EVENTS",
        // Latin America / Caribbean
        "ARGENTINA",
        "BOLIVIA",
        "BRAZIL FHD",
        "BRAZIL HD",
        "CARIBBEAN",
        "CHILE",
        "COLOMBIA",
        "COSTA RICA",
        "DOMINICAN REP",
        "ECUADOR",
        "EL SALVADOR",
        "HAITI",
        "HONDURAS",
        "LATINO",
        "MEXICO",
        "PANAMA",
        "PARAGUAY",
        "PERU",
        "SURINAME",
        "URUGUAY",
        "VENEZUELA",
        // Other Foreign
        "TURK ULUSAL",
        "FIFA WC 26 ASIAN",
        "FIFA WC26 EAST EUROP",
        "FIFA WC26 WEST EUROP",
        "RAMADAN 2026",
        "SHAHID",
        "VIP-PREMIUN AR CINEMA",
        "WAFCON",
    ];

    for name in &foreign_screenshot_cases {
        assert!(
            !is_american_live(name),
            "Expected screenshot foreign category '{}' to be BLOCKED, but it was ALLOWED",
            name
        );
    }
}

#[test]
fn test_foreign_live_channels_blocked() {
    let foreign_cases = vec![
        "FR | TF1 HD",
        "FR: France 2 FHD",
        "DE | Sky Sport Bundesliga 1",
        "DE: ZDF HD",
        "ES | LaLiga TV HD",
        "ES: Movistar+",
        "IT | Rai 1 FHD",
        "IT: Sky Sport Uno",
        "AR | beIN Sports 1 Arabic",
        "AR: Al Jazeera Arabic",
        "TR | TRT 1 HD",
        "TR: Bein Sports Haber",
        "PT | Sport TV 1 HD",
        "PT: RTP 1",
        "BR | Premiere Clubes HD",
        "BR: Globo SP HD",
        "IN | Star Sports 1 Hindi",
        "IN: Sony Ten 1",
        "RU | Match TV HD",
        "RU: Channel One Russia",
        "PL | TVP 1 HD",
        "PL: Canal+ Sport",
        "NL | Ziggo Sport Select",
        "BE | Play Sports 1 HD",
        "AL | SuperSport 1 HD",
        "GR | Cosmote Sport 1 HD",
        "RO | Digi Sport 1 HD",
        "CZ | Nova Sport 1",
        "HU | M4 Sport HD",
        "SE | SVT 1 HD",
        "NO | TV 2 Direkte",
        "DK | TV 2 Sport",
        "FI | Yle TV1",
        "IL | Kan 11 HD",
        "LATAM | ESPN Deportes",
    ];

    for name in &foreign_cases {
        assert!(
            !is_american_live(name),
            "Expected foreign content '{}' to be BLOCKED, but it was allowed",
            name
        );
    }
}

#[test]
fn test_english_vod_movies_allowed() {
    let english_vod = vec![
        "EN | The Matrix (1999) [4K HDR]",
        "EN: Dune: Part Two (2024)",
        "US | Oppenheimer (2023) [4K UHD]",
        "USA | Blade Runner 2049 (2017)",
        "[EN] Interstellar (2014) [1080p BluRay]",
        "[US] The Dark Knight (2008)",
        "(EN) Inception (2010)",
        "VIP | 4K UHD Master Cinema",
        "The Matrix Resurrections (2021)",
        "Severance S01 (2022)",
    ];

    for name in &english_vod {
        assert!(
            is_english_vod(name),
            "Expected English VOD '{}' to be ALLOWED, but it was filtered OFF",
            name
        );
    }
}

#[test]
fn test_foreign_vod_movies_blocked() {
    let foreign_vod = vec![
        "FR | Le Fabuleux Destin d'Amelie Poulain",
        "FR: Intouchables (2011)",
        "ES | El Secreto de sus Ojos (2009)",
        "ES: La Casa de Papel Pelicula",
        "DE | Das Leben der Anderen (2006)",
        "DE: Der Untergang (2004)",
        "IT | La Vita e Bella (1997)",
        "IT: Cinema Paradiso (1988)",
        "TR | Kurulus Osman Film (2023)",
        "AR | Aflam Arabiya Jadida (2024)",
        "RU | Brat 2 Remaster (2000)",
        "PL | Zimna Wojna (2018)",
        "PT | Cidade de Deus (2002)",
        "[FR] La Haine (1995)",
        "[ES] Roma (2018)",
        "[DE] Good Bye Lenin!",
        "[IT] Gomorra Film",
        "(FR) Les Miserables",
        "(ES) El Laberinto del Fauno",
        "TURKISH MOVIES | Ask Tesadufleri Sever",
        "ARABIC CINEMA | Al Ikhtiyar",
        "HINDI MOVIES | 3 Idiots",
        "GERMAN FILME | Das Boot",
        "FRENCH CINEMA | Portrait de la jeune fille en feu",
        "SPANISH PELICULAS | Todo sobre mi madre",
        "ITALIAN CINEMA | La Grande Bellezza",
    ];

    for name in &foreign_vod {
        assert!(
            !is_english_vod(name),
            "Expected foreign VOD '{}' to be BLOCKED, but it was allowed",
            name
        );
    }
}

#[test]
fn test_clean_american_name() {
    assert_eq!(clean_american_name("US: ESPN HD (60fps)"), "ESPN HD (60fps)");
    assert_eq!(clean_american_name("USA | NBC Sports FHD"), "NBC Sports FHD");
    assert_eq!(clean_american_name("US - FOX News"), "FOX News");
    assert_eq!(clean_american_name("EN | The Matrix"), "The Matrix");
}
#[test]
fn test_parse_stream_preserves_espn_and_short_channel_names() {
    use matrix_iptv_lib::parser::parse_stream;

    let cases = vec![
        ("AM | ESPN HD", "ESPN"),
        ("AM | ESPN 2 HD", "ESPN 2"),
        ("AM | ESPN [BK1] HD", "ESPN [BK1]"),
        ("AM | ESPNEWS HD", "ESPNEWS"),
        ("AM | ESPN NETWORK", "ESPN NETWORK"),
        ("AM | ESPNU HD", "ESPNU"),
        ("US | SHOWTIME EAST", "SHOWTIME EAST"),
        ("US | STARZ 4K", "STARZ"),
        ("US | SYFY HD", "SYFY"),
        ("AM | SEC NETWORK HD", "SEC NETWORK"),
        ("US: ESPN HD (60fps Live)", "ESPN"),
        ("USA | NBC Sports HD", "NBC Sports"),
        ("US - FOX News Live", "FOX News"),
        ("US LOCALS: WABC New York 7", "WABC New York 7"),
    ];

    for (raw, expected_substr) in cases {
        let parsed = parse_stream(raw, None);
        assert!(
            !parsed.display_name.trim().is_empty(),
            "Stream '{}' parsed to empty display_name!",
            raw
        );
        assert!(
            parsed.display_name.contains(expected_substr),
            "Stream '{}' parsed as '{}', expected to contain '{}'",
            raw,
            parsed.display_name,
            expected_substr
        );
    }
}
