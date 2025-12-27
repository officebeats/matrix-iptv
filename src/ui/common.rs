use ratatui::style::{Color, Modifier, Style};
use crate::parser::{Quality, ContentType};
use crate::ui::colors::CP_GREEN;
use crate::sports::SportsEvent;
use ratatui::text::Span;

pub fn stylize_channel_name(
    name: &str,
    is_vip: bool,
    is_ended: bool,
    quality: Option<Quality>,
    content_type: Option<ContentType>,
    sports_event: Option<&SportsEvent>,
    base_style: Style,
) -> (Vec<Span<'static>>, Option<&'static str>) {
    let mut spans = Vec::new();
    
    let (t1_color, t2_color, ppv_color, vip_color, raw_color, hd_color, fhd_color, fps_color) = if is_ended {
        let dim = Color::Rgb(100, 100, 100);
        (dim, dim, dim, dim, dim, dim, dim, dim)
    } else {
        (Color::Cyan, CP_GREEN, Color::Rgb(255, 105, 180), Color::Yellow, Color::Cyan, Color::Cyan, CP_GREEN, Color::Yellow)
    };

    let mut base_style = base_style;
    if is_ended {
        base_style = base_style.add_modifier(Modifier::CROSSED_OUT);
    }

    let mut found_vip = false;
    let mut found_ppv = false;
    let mut found_4k = false;
    let mut found_hd = false;
    let mut found_fhd = false;
    let mut detected_sport_icon = "";

    if let Some(event) = sports_event {
        let words: Vec<&str> = name.split_whitespace().collect();
        for word in words {
             let check = word.replace(&['(', ')', '[', ']', '{', '}', ':'][..], "").trim().to_uppercase();
             detected_sport_icon = match check.as_str() {
                 "NBA" => "🏀",
                 "NFL" => "🏈",
                 "MLB" => "⚾",
                 "NHL" => "🏒",
                 "UFC" | "MMA" => "🥊",
                 "F1" | "NASCAR" | "RACING" => "🏎️",
                 "GOLF" | "PGA" => "⛳",
                 "TENNIS" | "ATP" | "WTA" => "🎾",
                 "SOCCER" | "FOOTBALL" | "LEAGUE" | "BUNDESLIGA" | "LALIGA" | "PREMIER" | "UEFA" | "FIFA" => "⚽",
                 "CRICKET" => "🏏",
                 "RUGBY" => "🏉",
                 _ => detected_sport_icon,
             };
             if !detected_sport_icon.is_empty() { break; }
        }

        spans.push(Span::styled(format!("{}", event.team1), base_style.fg(t1_color)));
        spans.push(Span::styled(" vs ", Style::default().fg(Color::Gray)));
        spans.push(Span::styled(format!("{}", event.team2), base_style.fg(t2_color)));
        
    } else {
        let words: Vec<&str> = name.split_whitespace().collect();
        for (i, word) in words.iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw(" "));
            }
            
            let sub_parts: Vec<&str> = word.split('/').collect();
            for (j, sub) in sub_parts.iter().enumerate() {
                if j > 0 {
                    spans.push(Span::styled("/", base_style));
                }

                let upper = sub.replace(&['(', ')', '[', ']', '{', '}', ':'][..], "").trim().to_uppercase();
                let check_word = upper.as_str();

                match check_word {
                    "PPV" => {
                        found_ppv = true;
                        spans.push(Span::styled("(PPV)", base_style.fg(ppv_color).add_modifier(Modifier::BOLD)));
                    }
                    "VIP" => {
                        found_vip = true;
                        spans.push(Span::styled("(VIP)", base_style.fg(vip_color).add_modifier(Modifier::BOLD)));
                    }
                    "RAW" => {
                        spans.push(Span::styled("(RAW)", base_style.fg(raw_color).add_modifier(Modifier::BOLD)));
                    }
                    "HD" | "HQ" => {
                        found_hd = true;
                        spans.push(Span::styled("(HD)", base_style.fg(hd_color).add_modifier(Modifier::BOLD)));
                    }
                    "FHD" | "1080" | "1080P" => {
                        found_fhd = true;
                        spans.push(Span::styled("(FHD)", base_style.fg(fhd_color).add_modifier(Modifier::BOLD)));
                    }
                    val if ["4K", "UHD", "HEVC"].contains(&val) => {
                        found_4k = true;
                        spans.push(Span::styled(format!("({})", val), base_style.fg(fhd_color).add_modifier(Modifier::BOLD)));
                    }
                    val if val.ends_with("FPS") && val.len() > 3 => {
                        spans.push(Span::styled(format!("({})", val.to_lowercase()), base_style.fg(fps_color).add_modifier(Modifier::BOLD)));
                    }
                    _ => {
                        if detected_sport_icon.is_empty() {
                             detected_sport_icon = match check_word {
                                 "NBA" => "🏀",
                                 "NFL" => "🏈",
                                 "MLB" => "⚾",
                                 "NHL" => "🏒",
                                 "UFC" | "MMA" => "🥊",
                                 "F1" | "NASCAR" | "RACING" => "🏎️",
                                 "GOLF" | "PGA" => "⛳",
                                 "TENNIS" | "ATP" | "WTA" => "🎾",
                                 "SOCCER" | "FOOTBALL" | "LEAGUE" | "BUNDESLIGA" | "LALIGA" | "PREMIER" | "UEFA" | "FIFA" => "⚽",
                                 "CRICKET" => "🏏",
                                 "RUGBY" => "🏉",
                                 _ => "",
                             };
                        }
                        
                        spans.push(Span::styled(format!("{}", sub), base_style));
                    }
                }
            }
        }
    }

    // Icon insertion removed here as we handle it in panes.rs for better categorization

    if is_vip && !found_vip {
         spans.push(Span::styled(" (VIP)", base_style.fg(vip_color).add_modifier(Modifier::BOLD)));
    }
    
    if let Some(ct) = content_type {
        if ct == ContentType::PPV && !found_ppv {
             spans.push(Span::styled(" (PPV)", base_style.fg(ppv_color).add_modifier(Modifier::BOLD)));
        }
    }
    
    if let Some(q) = quality {
        if (q == Quality::UHD4K) && !found_4k {
             spans.push(Span::styled(" (4K)", base_style.fg(fhd_color).add_modifier(Modifier::BOLD)));
        } else if (q == Quality::FHD) && !found_fhd {
             spans.push(Span::styled(" (FHD)", base_style.fg(fhd_color).add_modifier(Modifier::BOLD)));
        } else if (q == Quality::HD) && !found_hd {
             spans.push(Span::styled(" (HD)", base_style.fg(hd_color).add_modifier(Modifier::BOLD)));
        }
    }

    let icon_ret = if detected_sport_icon.is_empty() { None } else { Some(detected_sport_icon) };
    (spans, icon_ret)
}
