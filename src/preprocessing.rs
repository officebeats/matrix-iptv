use crate::api::{Category, Stream};
use std::collections::HashSet;

pub fn preprocess_categories(
    cats: &mut Vec<Category>,
    favorites: &HashSet<String>,
    modes: &[crate::config::ProcessingMode],
    is_live: bool,
    is_vod: bool,
    _account_name: &str,
) {
    if !cats.iter().any(|c| c.category_id == "ALL") {
        cats.insert(
            0,
            Category {
                category_id: "ALL".to_string(),
                category_name: if is_live {
                    "All Channels".to_string()
                } else if is_vod {
                    "All Movies".to_string()
                } else {
                    "All Series".to_string()
                },
                is_american: true,
                is_english: true,
                ..Default::default()
            },
        );
    }

    let use_merica = modes.contains(&crate::config::ProcessingMode::Merica);
    let use_sports = modes.contains(&crate::config::ProcessingMode::Sports);
    let use_all_english = modes.contains(&crate::config::ProcessingMode::AllEnglish);

    // 1. Filter
    cats.retain_mut(|c| {
        // Keep ALL category always
        if c.category_id == "ALL" {
            return true;
        }

        let mut keep = true;

        if is_live {
            // Merica Mode Logic
            if use_merica {
                c.is_american = crate::parser::is_american_live(&c.category_name);

                if !c.is_american {
                    keep = false;
                }
            }

            // All English Logic (if Merica not active, or additive?)
            // If AllEnglish is ON, we only keep English.
            // If Merica is ALSO on, Merica is stricter, so it implicitly satisfies AllEnglish mostly.
            // But let's treat them as additive filters (AND).
            if use_all_english {
                c.is_english = crate::parser::is_english_live(&c.category_name);
                if !c.is_english {
                    keep = false;
                }
            }

            // Sports Mode Logic - If ONLY Sports is on, we filter for sports.
            // If Sports AND Merica are on, we filter for American Sports.
            if use_sports && !crate::parser::is_sports_content(&c.category_name) {
                keep = false;
            }
        } else {
            // VOD/Series
            if use_merica || use_all_english {
                c.is_english = crate::parser::is_english_vod(&c.category_name);
                if !c.is_english {
                    keep = false;
                }
            }
            if use_sports && !crate::parser::is_sports_content(&c.category_name) {
                keep = false;
            }
        }

        keep
    });

    // 2. Process (Clean names & Metadata) - Parallelized
    use rayon::prelude::*;
    cats.par_iter_mut().for_each(|c| {
        if use_merica {
            c.clean_name = crate::parser::clean_american_name(&c.category_name);
            c.category_name = c.clean_name.clone(); // Update for display
        } else {
            c.clean_name = c.category_name.clone();
        }
        c.search_name = c.clean_name.to_lowercase();

        // Cache parsed metadata to enable O(1) TUI rendering
        if c.cached_parsed.is_none() {
            c.cached_parsed = Some(Box::new(crate::parser::parse_category(&c.category_name)));
        }
    });

    // 3. Sort - Parallelized
    cats.par_sort_by(|a, b| {
        if a.category_id == "ALL" {
            return std::cmp::Ordering::Less;
        }
        if b.category_id == "ALL" {
            return std::cmp::Ordering::Greater;
        }
        let a_fav = favorites.contains(&a.category_id);
        let b_fav = favorites.contains(&b.category_id);

        // Sports Mode Hoisting
        if use_sports {
            let a_sport = crate::parser::is_sports_content(&a.category_name);
            let b_sport = crate::parser::is_sports_content(&b.category_name);
            if a_sport && !b_sport {
                return std::cmp::Ordering::Less;
            }
            if !a_sport && b_sport {
                return std::cmp::Ordering::Greater;
            }
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
    tx: Option<tokio::sync::mpsc::Sender<crate::app::AsyncAction>>,
) {
    use crate::api::get_id_str;

    let use_merica = modes.contains(&crate::config::ProcessingMode::Merica);
    let use_sports = modes.contains(&crate::config::ProcessingMode::Sports);
    let use_all_english = modes.contains(&crate::config::ProcessingMode::AllEnglish);

    if let Some(ref tx) = tx {
        let _ = tx.try_send(crate::app::AsyncAction::LoadingMessage(
            "Phase 1/3: Stripping provider separators...".to_string(),
        ));
    }

    // 0. Strip out provider-injected separator/header entries.
    streams.retain(|s| {
        let name = s.name.trim();
        if name.is_empty() {
            return false;
        }

        let sep_chars: &[char] = &[
            '❖', '#', '═', '●', '◆', '■', '▬', '━', '─', '☆', '★', '◇', '◈', '▶', '▷',
        ];
        let sep_count = name.chars().filter(|c| sep_chars.contains(c)).count();
        if sep_count >= 2 {
            return false;
        }
        if name
            .chars()
            .all(|c| sep_chars.contains(&c) || c.is_whitespace())
        {
            return false;
        }
        true
    });

    if let Some(ref tx) = tx {
        let _ = tx.try_send(crate::app::AsyncAction::LoadingMessage(format!(
            "Phase 2/3: Cleaning metadata for {} streams...",
            streams.len()
        )));
    }

    // 1. Filter
    streams.retain_mut(|s| {
        let mut keep = true;

        if is_live {
            if use_merica {
                s.is_american = crate::parser::is_american_live(&s.name);
                if !s.is_american {
                    keep = false;
                }
            }
            if use_all_english {
                s.is_english = crate::parser::is_english_live(&s.name);
                if !s.is_english {
                    keep = false;
                }
            }
            if use_sports && !crate::parser::is_sports_content(&s.name) {
                keep = false;
            }
        } else {
            if use_merica || use_all_english {
                s.is_english = crate::parser::is_english_vod(&s.name);
                if !s.is_english {
                    keep = false;
                }
            }
            if use_sports && !crate::parser::is_sports_content(&s.name) {
                keep = false;
            }
        }
        keep
    });

    // 2. Process (Clean names & Metadata) - Parallelized via Rayon
    use rayon::prelude::*;
    let should_clean = use_merica;

    if let Some(ref tx) = tx {
        let _ = tx.try_send(crate::app::AsyncAction::LoadingMessage(format!(
            "Phase 2/3: Cleaning metadata for {} streams (Multi-Core)...",
            streams.len()
        )));
    }

    streams.par_iter_mut().for_each(|s| {
        if should_clean {
            s.clean_name = crate::parser::clean_american_name(&s.name);
            s.name = s.clean_name.clone();
        } else {
            s.clean_name = s.name.clone();
        }

        // Sports Mode Icon Prefixing
        if use_sports && is_live {
            if let Some(league) = s.epg_channel_id.as_ref().or(Some(&s.name)) {
                let lower = league.to_lowercase();
                if lower.contains("nba") {
                    s.clean_name = format!("🏀 {}", s.clean_name);
                } else if lower.contains("nfl") {
                    s.clean_name = format!("🏈 {}", s.clean_name);
                } else if lower.contains("mlb") {
                    s.clean_name = format!("⚾ {}", s.clean_name);
                } else if lower.contains("nhl") {
                    s.clean_name = format!("🏒 {}", s.clean_name);
                }
            }
        }

        s.stream_display_name = Some(s.clean_name.clone());
        s.search_name = s.clean_name.to_lowercase();
        s.account_name = Some(_account_name.to_string());

        // Cache parsed metadata to enable O(1) TUI rendering
        if s.cached_parsed.is_none() {
            s.cached_parsed = Some(Box::new(crate::parser::parse_stream(&s.name, None)));
        }
    });

    // 3. Sort - Zero-Copy Architectural Pattern
    streams.sort_by(|a, b| {
        let a_id = get_id_str(&a.stream_id);
        let b_id = get_id_str(&b.stream_id);
        let a_fav = favorites.contains(&a_id);
        let b_fav = favorites.contains(&b_id);

        // Tier 1: Favorites Hoisting
        match (a_fav, b_fav) {
            (true, false) => return std::cmp::Ordering::Less,
            (false, true) => return std::cmp::Ordering::Greater,
            _ => {}
        }

        // Tier 2: Numerical Order (Provider Num)
        let a_num = a.num.as_ref().and_then(|v| v.as_i64()).unwrap_or(i64::MAX);
        let b_num = b.num.as_ref().and_then(|v| v.as_i64()).unwrap_or(i64::MAX);
        match a_num.cmp(&b_num) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
        }

        // Tier 3: Lexicographical fallback (O(1) reference comparison)
        a.name.cmp(&b.name)
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::Category;
    use crate::config::ProcessingMode;

    fn category(id: &str, name: &str) -> Category {
        Category {
            category_id: id.to_string(),
            category_name: name.to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn test_merica_category_filter_keeps_us_packages_ppv_and_247() {
        let mut categories = vec![
            category("1", "NBA Package"),
            category("2", "NBA League Pass"),
            category("3", "NHL Real"),
            category("4", "PPV Events"),
            category("5", "24/7 Channels"),
            category("6", "UK PPV"),
            category("7", "CA 24/7"),
            category("8", "IN | 24/7 Cricket"),
            category("9", "International PPV"),
        ];

        preprocess_categories(
            &mut categories,
            &HashSet::new(),
            &[ProcessingMode::Merica],
            true,
            false,
            "Trex",
        );

        let names: Vec<_> = categories
            .iter()
            .map(|c| c.category_name.as_str())
            .collect();

        assert!(names.contains(&"All Channels"));
        assert!(names.contains(&"NBA Package"));
        assert!(names.contains(&"NBA League Pass"));
        assert!(names.contains(&"NHL Real"));
        assert!(names.contains(&"PPV Events"));
        assert!(names.contains(&"24/7 Channels"));
        assert!(!names.contains(&"UK PPV"));
        assert!(!names.contains(&"CA 24/7"));
        assert!(!names.contains(&"IN | 24/7 Cricket"));
        assert!(!names.contains(&"International PPV"));
    }
}
