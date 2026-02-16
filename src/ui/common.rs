use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph},
    Frame,
};
use crate::parser::{Quality, ContentType};
use crate::ui::colors::{MATRIX_GREEN, SOFT_GREEN, TEXT_PRIMARY, TEXT_SECONDARY, TEXT_DIM};
use crate::sports::SportsEvent;

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
    
    let (ppv_color, vip_color, raw_color, hd_color, fhd_color, fps_color) = if is_ended {
        let dim = TEXT_DIM;
        (dim, dim, dim, dim, dim, dim)
    } else {
        // Restrained palette: green accents + white text
        (Color::Rgb(255, 105, 180), Color::Rgb(255, 200, 80), MATRIX_GREEN, MATRIX_GREEN, MATRIX_GREEN, Color::Rgb(255, 200, 80))
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

        // Sports matchup: Home green, Away white
        spans.push(Span::styled(format!("{}", event.team1), base_style.fg(MATRIX_GREEN)));
        spans.push(Span::styled(" vs ", Style::default().fg(TEXT_SECONDARY)));
        spans.push(Span::styled(format!("{}", event.team2), base_style.fg(TEXT_PRIMARY)));
        
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
                    "MULTI-SUB" | "MULTISUB" | "MULTI-AUDIO" | "MULTIAUDIO" | "MULTILANG" | "MULTI-LANG" | "MULTI" => {
                        continue;
                    }
                    "PPV" => {
                        found_ppv = true;
                        spans.push(Span::styled("PPV", base_style.fg(ppv_color).add_modifier(Modifier::BOLD)));
                    }
                    "VIP" => {
                        found_vip = true;
                        spans.push(Span::styled("VIP", base_style.fg(vip_color).add_modifier(Modifier::BOLD)));
                    }
                    "RAW" => {
                        spans.push(Span::styled("RAW", base_style.fg(raw_color)));
                    }
                    "HD" | "HQ" => {
                        found_hd = true;
                        spans.push(Span::styled("HD", base_style.fg(hd_color)));
                    }
                    "FHD" | "1080" | "1080P" => {
                        found_fhd = true;
                        spans.push(Span::styled("FHD", base_style.fg(fhd_color)));
                    }
                    val if ["4K", "UHD", "HEVC"].contains(&val) => {
                        found_4k = true;
                        spans.push(Span::styled(format!("{}", val), base_style.fg(fhd_color).add_modifier(Modifier::BOLD)));
                    }
                    val if val.ends_with("FPS") && val.len() > 3 => {
                        spans.push(Span::styled(format!("{}", val.to_lowercase()), base_style.fg(fps_color)));
                    }
                    _ => {
                        let is_year = ((sub.starts_with('(') && sub.ends_with(')')) || (sub.starts_with('[') && sub.ends_with(']'))) && sub.len() == 6 && sub[1..5].chars().all(|c| c.is_digit(10));
                        
                        if is_year {
                            spans.push(Span::styled(format!("{}", sub), Style::default().fg(TEXT_SECONDARY)));
                        } else {
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
    }

    if is_vip && !found_vip {
         spans.push(Span::styled(" VIP", base_style.fg(vip_color).add_modifier(Modifier::BOLD)));
    }
    
    if let Some(ct) = content_type {
        if ct == ContentType::PPV && !found_ppv {
             spans.push(Span::styled(" PPV", base_style.fg(ppv_color).add_modifier(Modifier::BOLD)));
        }
    }
    
    if let Some(q) = quality {
        if (q == Quality::UHD4K) && !found_4k {
             spans.push(Span::styled(" 4K", base_style.fg(fhd_color).add_modifier(Modifier::BOLD)));
        } else if (q == Quality::FHD) && !found_fhd {
             spans.push(Span::styled(" FHD", base_style.fg(fhd_color)));
        } else if (q == Quality::HD) && !found_hd {
             spans.push(Span::styled(" HD", base_style.fg(hd_color)));
        }
    }

    let icon_ret = if detected_sport_icon.is_empty() { None } else { Some(detected_sport_icon) };
    (spans, icon_ret)
}

/// Modern minimal box (Claude Code style) — left border only for active panes
pub fn render_matrix_box(f: &mut Frame, area: Rect, title: &str, border_color: Color) -> Rect {
    let clean_title = title.trim().trim_matches('/');

    // Title line at top, then content below
    if !clean_title.is_empty() {
        let title_area = Rect { x: area.x, y: area.y, width: area.width, height: 1 };
        let title_widget = Paragraph::new(Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(clean_title, Style::default().fg(border_color).add_modifier(Modifier::BOLD)),
        ]));
        f.render_widget(title_widget, title_area);

        Rect {
            x: area.x + 1,
            y: area.y + 1,
            width: area.width.saturating_sub(1),
            height: area.height.saturating_sub(1),
        }
    } else {
        Rect {
            x: area.x + 1,
            y: area.y,
            width: area.width.saturating_sub(1),
            height: area.height,
        }
    }
}

/// Minimal block with optional title — no borders
pub fn render_composite_block(f: &mut Frame, area: Rect, title: Option<&str>) -> Rect {
    let t = title.unwrap_or("");
    render_matrix_box(f, area, t, SOFT_GREEN)
}
