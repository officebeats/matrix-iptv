use crate::api::{Category, Stream};
use std::collections::HashSet;

pub fn preprocess_categories(
    cats: &mut Vec<Category>,
    favorites: &HashSet<String>,
    modes: &[crate::config::ProcessingMode],
    is_live: bool,
    is_vod: bool,
    account_name: &str,
) {
    if !cats.iter().any(|c| c.category_id == "ALL") {
        cats.insert(0, Category {
            category_id: "ALL".to_string(),
            category_name: if is_live { "All Channels".to_string() } else if is_vod { "All Movies".to_string() } else { "All Series".to_string() },
            is_american: true,
            is_english: true,
            ..Default::default()
        });
    }

    let account_lower = account_name.to_lowercase();
    let use_merica = modes.contains(&crate::config::ProcessingMode::Merica);
    let use_sports = modes.contains(&crate::config::ProcessingMode::Sports);
    let use_all_english = modes.contains(&crate::config::ProcessingMode::AllEnglish);

    // 1. Filter
    cats.retain_mut(|c| {
        // Keep ALL category always
        if c.category_id == "ALL" { return true; }

        let mut keep = true;

        if is_live {
            // Merica Mode Logic
            if use_merica {
                c.is_american = crate::parser::is_american_live(&c.category_name);
                
                // Strong Playlist overrides
                if account_lower.contains("strong") {
                     let name = c.category_name.to_uppercase();
                     if name.starts_with("AR |") || name.starts_with("AR|") || name.starts_with("AR :") {
                         c.is_american = false;
                     }
                     if name.contains("NBA PASS") || name.contains("NBA REAL") || name.contains("NHL REAL") {
                         c.is_american = false;
                     }
                }
                // Trex Playlist overrides
                if account_lower.contains("trex") {
                     let name = c.category_name.to_uppercase();
                     if name.contains("NBA NETWORK") || name.contains("NBA LEAGUE PASS") {
                         c.is_american = false;
                     }
                }
                
                if !c.is_american { keep = false; }
            }

            // All English Logic (if Merica not active, or additive?)
            // If AllEnglish is ON, we only keep English. 
            // If Merica is ALSO on, Merica is stricter, so it implicitly satisfies AllEnglish mostly.
            // But let's treat them as additive filters (AND).
            if use_all_english {
                c.is_english = crate::parser::is_english_live(&c.category_name);
                if !c.is_english { keep = false; }
            }

            // Sports Mode Logic - If ONLY Sports is on, we filter for sports.
            // If Sports AND Merica are on, we filter for American Sports.
            if use_sports {
                if !crate::parser::is_sports_content(&c.category_name) {
                    keep = false;
                }
            }
        } else {
            // VOD/Series
            if use_merica || use_all_english {
                c.is_english = crate::parser::is_english_vod(&c.category_name);
                if !c.is_english { keep = false; }
            }
            if use_sports {
                if !crate::parser::is_sports_content(&c.category_name) {
                    keep = false;
                }
            }
        }

        keep
    });

    // 2. Clean names
    let should_clean = use_merica; // Only clean names if in 'Merica mode
    for c in cats.iter_mut() {
        c.clean_name = if should_clean {
            crate::parser::clean_american_name(&c.category_name)
        } else {
            c.category_name.clone()
        };
        c.search_name = c.clean_name.to_lowercase();
    }

    // 3. Sort
    cats.sort_by(|a, b| {
        if a.category_id == "ALL" { return std::cmp::Ordering::Less; }
        if b.category_id == "ALL" { return std::cmp::Ordering::Greater; }
        let a_fav = favorites.contains(&a.category_id);
        let b_fav = favorites.contains(&b.category_id);
        
        // Sports Mode Hoisting
        if use_sports {
            let a_sport = crate::parser::is_sports_content(&a.category_name);
            let b_sport = crate::parser::is_sports_content(&b.category_name);
            if a_sport && !b_sport { return std::cmp::Ordering::Less; }
            if !a_sport && b_sport { return std::cmp::Ordering::Greater; }
        }

        match (a_fav, b_fav) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.category_name.cmp(&b.category_name),
        }
    });
}

pub fn preprocess_streams(
    streams: &mut Vec<Stream>,
    favorites: &HashSet<String>,
    modes: &[crate::config::ProcessingMode],
    is_live: bool,
    _account_name: &str,
) {
    use rayon::prelude::*;
    use crate::api::get_id_str;

    let use_merica = modes.contains(&crate::config::ProcessingMode::Merica);
    let use_sports = modes.contains(&crate::config::ProcessingMode::Sports);
    let use_all_english = modes.contains(&crate::config::ProcessingMode::AllEnglish);

    // 1. Filter
    streams.retain_mut(|s| {
        let mut keep = true;

        if is_live {
            if use_merica {
                s.is_american = crate::parser::is_american_live(&s.name);
                if !s.is_american { keep = false; }
            }
            if use_all_english {
                s.is_english = crate::parser::is_english_live(&s.name);
                if !s.is_english { keep = false; }
            }
            if use_sports {
                 if !crate::parser::is_sports_content(&s.name) { keep = false; }
            }
        } else {
            if use_merica || use_all_english {
                s.is_english = crate::parser::is_english_vod(&s.name);
                if !s.is_english { keep = false; }
            }
            if use_sports {
                 if !crate::parser::is_sports_content(&s.name) { keep = false; }
            }
        }
        keep
    });

    // 2. Process (Clean names & Metadata) - Parallelized
    let should_clean = use_merica;
    streams.par_iter_mut().for_each(|s| {
        s.clean_name = if should_clean {
            crate::parser::clean_american_name(&s.name)
        } else {
            s.name.clone()
        };
        
        // Sports Mode Icon Prefixing
        if use_sports && is_live {
            if let Some(league) = s.epg_channel_id.as_ref().or(Some(&s.name)) {
                let lower = league.to_lowercase();
                if lower.contains("nba") { s.clean_name = format!("🏀 {}", s.clean_name); }
                else if lower.contains("nfl") { s.clean_name = format!("🏈 {}", s.clean_name); }
                else if lower.contains("mlb") { s.clean_name = format!("⚾ {}", s.clean_name); }
                else if lower.contains("nhl") { s.clean_name = format!("🏒 {}", s.clean_name); }
            }
        }

        s.stream_display_name = Some(s.clean_name.clone());
        s.search_name = s.clean_name.to_lowercase();
        s.account_name = Some(_account_name.to_string());
    });

    // 3. Sort - Parallelized
    streams.par_sort_by(|a, b| {
        let a_id = get_id_str(&a.stream_id);
        let b_id = get_id_str(&b.stream_id);
        let a_fav = favorites.contains(&a_id);
        let b_fav = favorites.contains(&b_id);
        
        match (a_fav, b_fav) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => {
                let a_num = a.num.as_ref().and_then(|v| v.as_u64()).unwrap_or(u64::MAX);
                let b_num = b.num.as_ref().and_then(|v| v.as_u64()).unwrap_or(u64::MAX);
                a_num.cmp(&b_num)
            }
        }
    });
}
