use crate::api::{Category, Stream};
use std::collections::HashSet;

pub fn preprocess_categories(
    cats: &mut Vec<Category>,
    favorites: &HashSet<String>,
    american_mode: bool,
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

    // 1. Categorize and Filter
    if american_mode {
        cats.retain_mut(|c| {
            if is_live {
                c.is_american = c.category_id == "ALL" || crate::parser::is_american_live(&c.category_name);
                // Strong Playlist specific overrides
                if account_lower.contains("strong") && is_live && c.category_id != "ALL" {
                     let name = c.category_name.to_uppercase();
                     if name.starts_with("AR |") || name.starts_with("AR|") || name.starts_with("AR :") {
                         c.is_american = false;
                     }
                     if name.contains("NBA PASS") || name.contains("NBA REAL") || name.contains("NHL REAL") {
                         c.is_american = false;
                     }
                }
                c.is_american
            } else {
                c.is_english = c.category_id == "ALL" || crate::parser::is_english_vod(&c.category_name);
                c.is_english
            }
        });
    } else {
        for c in cats.iter_mut() {
            if is_live {
                c.is_american = c.category_id == "ALL" || crate::parser::is_american_live(&c.category_name);
                if account_lower.contains("strong") && is_live && c.category_id != "ALL" {
                     let name = c.category_name.to_uppercase();
                     if name.starts_with("AR |") || name.starts_with("AR|") || name.starts_with("AR :") {
                         c.is_american = false;
                     }
                }
            } else {
                c.is_english = c.category_id == "ALL" || crate::parser::is_english_vod(&c.category_name);
            }
        }
    }

    // 2. Clean names
    for c in cats.iter_mut() {
        c.clean_name = if american_mode {
            crate::parser::clean_american_name(&c.category_name)
        } else {
            c.category_name.clone()
        };
        c.search_name = c.clean_name.to_lowercase();
    }

    cats.sort_by(|a, b| {
        if a.category_id == "ALL" { return std::cmp::Ordering::Less; }
        if b.category_id == "ALL" { return std::cmp::Ordering::Greater; }
        let a_fav = favorites.contains(&a.category_id);
        let b_fav = favorites.contains(&b.category_id);
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
    american_mode: bool,
    is_live: bool,
    _account_name: &str,
) {
    use crate::api::get_id_str;

    // 1. Initial categorization and filtering
    if american_mode {
        streams.retain_mut(|s| {
            if is_live {
                s.is_american = crate::parser::is_american_live(&s.name);
                s.is_american
            } else {
                s.is_english = crate::parser::is_english_vod(&s.name);
                s.is_english
            }
        });
    } else {
        for s in streams.iter_mut() {
            if is_live {
                s.is_american = crate::parser::is_american_live(&s.name);
            } else {
                s.is_english = crate::parser::is_english_vod(&s.name);
            }
        }
    }

    // 2. Process remaining streams (Clean names and prepare search metadata)
    for s in streams.iter_mut() {
        s.clean_name = if american_mode {
            crate::parser::clean_american_name(&s.name)
        } else {
            s.name.clone()
        };
        s.stream_display_name = Some(s.clean_name.clone());
        s.search_name = s.clean_name.to_lowercase();
    }

    // 3. Sort streams efficiently
    // We pre-extract sort keys to avoid repetitive work in the comparator
    let mut sort_data: Vec<_> = streams
        .drain(..)
        .map(|s| {
            let id = get_id_str(&s.stream_id);
            let is_fav = favorites.contains(&id);
            let num = s.num.as_ref().and_then(|v| v.as_u64()).unwrap_or(u64::MAX);
            (s, is_fav, num)
        })
        .collect();

    sort_data.sort_by(|(_, a_fav, a_num), (_, b_fav, b_num)| {
        match (a_fav, b_fav) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a_num.cmp(b_num),
        }
    });

    for (s, _, _) in sort_data {
        streams.push(s);
    }
}
