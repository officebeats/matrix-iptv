use ratatui::{
    layout::{Rect, Layout, Constraint, Direction},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
    Frame,
};
use crate::app::{App, CurrentScreen};
use crate::parser::{parse_category, parse_stream, country_flag, country_color};
use crate::ui::colors::{MATRIX_GREEN, SOFT_GREEN, HIGHLIGHT_BG, TEXT_PRIMARY, TEXT_SECONDARY, TEXT_DIM};
use crate::ui::common::stylize_channel_name;
use chrono::{Utc, DateTime};
use chrono_tz::Tz;

fn format_relative_time(dt: DateTime<Utc>, user_tz: &Tz) -> String {
    let local = dt.with_timezone(user_tz);
    let now = Utc::now().with_timezone(user_tz);
    
    let day = if local.date_naive() == now.date_naive() {
        "Today".to_string()
    } else if (local.date_naive() - now.date_naive()).num_days() == 1 {
        "Tomorrow".to_string()
    } else {
        local.format("%A").to_string()
    };
    
    format!("{} {}", day, local.format("%I:%M %p"))
}

/// Shared highlight style — subtle left bar + tinted background
fn list_highlight_style() -> Style {
    Style::default()
        .fg(MATRIX_GREEN)
        .bg(HIGHLIGHT_BG)
        .add_modifier(Modifier::BOLD)
}

pub fn render_categories_pane(f: &mut Frame, app: &mut App, area: Rect, border_color: Color) {
    let visible_height = area.height.saturating_sub(2) as usize;
    let total = app.categories.len();
    let selected = app.selected_category_index;

    let half_window = visible_height / 2;
    let start = if selected > half_window {
        selected - half_window
    } else {
        0
    };
    let end = (start + visible_height + half_window).min(total);
    let adjusted_start = if end == total && end > visible_height + half_window {
        end.saturating_sub(visible_height + half_window)
    } else {
        start
    };

    let items: Vec<ListItem> = app
        .categories
        .iter()
        .enumerate()
        .skip(adjusted_start)
        .take(end - adjusted_start)
        .map(|(_, c)| {
            let parsed = parse_category(&c.category_name);
            let mut spans = vec![];

            if app.config.favorites.categories.contains(&c.category_id) {
                spans.push(ratatui::text::Span::styled("★ ", Style::default().fg(MATRIX_GREEN)));
            }

            let is_league = parsed.country.as_ref().map(|cc| ["NBA", "NFL", "MLB", "NHL", "UFC", "SPORTS", "PPV"].contains(&cc.as_str())).unwrap_or(false);
            let name_color = if is_league {
                parsed.country.as_ref().map(|cc| country_color(cc)).unwrap_or(TEXT_PRIMARY)
            } else {
                MATRIX_GREEN
            };

            let (styled_name, _) = stylize_channel_name(
                &parsed.display_name,
                parsed.is_vip,
                false,
                parsed.quality,
                parsed.content_type,
                None,
                Style::default().fg(name_color),
            );

            // Clean icon prefix
            let icon = if app.current_screen == CurrentScreen::VodCategories || app.current_screen == CurrentScreen::VodStreams {
                "◆ "
            } else if app.current_screen == CurrentScreen::SeriesCategories || app.current_screen == CurrentScreen::SeriesStreams {
                "◆ "
            } else {
                "◆ "
            };
            spans.push(ratatui::text::Span::styled(icon, Style::default().fg(SOFT_GREEN)));
            spans.extend(styled_name);

            ListItem::new(Line::from(spans))
        })
        .collect();

    let title = if app.categories.is_empty() {
        " categories ".to_string()
    } else {
        format!(" categories ({}) ", app.categories.len())
    };
    let inner_area = crate::ui::common::render_matrix_box(f, area, &title, border_color);
    
    let list = List::new(items)
        .highlight_style(list_highlight_style())
        .highlight_symbol(" ▎");

    let mut adjusted_state = app.category_list_state.clone();
    if adjusted_start > 0 {
        adjusted_state.select(Some(selected - adjusted_start));
    }

    f.render_stateful_widget(list, inner_area, &mut adjusted_state);
}

pub fn render_streams_pane(f: &mut Frame, app: &mut App, area: Rect, border_color: Color) {
    let visible_height = area.height.saturating_sub(2) as usize;
    let total = app.streams.len();
    let selected = app.selected_stream_index;

    let half_window = visible_height / 2;
    let start = if selected > half_window {
        selected - half_window
    } else {
        0
    };
    let end = (start + visible_height + half_window).min(total);
    let adjusted_start = if end == total && end > visible_height + half_window {
        end.saturating_sub(visible_height + half_window)
    } else {
        start
    };

    let user_tz_str = app.config.get_user_timezone();
    let user_tz: Tz = user_tz_str.parse().unwrap_or(chrono_tz::UTC);
    let tz_display = format!("{} ({})", user_tz_str, Utc::now().with_timezone(&user_tz).format("%Z"));

    let items: Vec<ListItem> = app
        .streams
        .iter()
        .enumerate()
        .skip(adjusted_start)
        .take(end - adjusted_start)
        .map(|(_, s)| {
            let parsed = crate::parser::parse_stream(&s.name, app.provider_timezone.as_deref());
            let now = Utc::now();
            let is_ended = parsed.stop_time.map(|t| now > t).unwrap_or(false);
            let is_live = parsed.start_time.map(|st| now >= st).unwrap_or(false) && !is_ended;
            let is_blink_on = app.loading_tick / 4 % 2 == 0;

            let mut spans = vec![];
            
            // 1. Icon Logic (Sports Only)
            let mut league_icon_span: Option<ratatui::text::Span> = None;
            let upper_name = parsed.display_name.to_uppercase();
            if upper_name.contains("NBA") || upper_name.contains("NCAAB") {
                league_icon_span = Some(ratatui::text::Span::styled("\u{1f3c0} ", Style::default().fg(Color::Rgb(255, 140, 0))));
            } else if upper_name.contains("NFL") || upper_name.contains("NCAAF") {
                league_icon_span = Some(ratatui::text::Span::styled("\u{1f3c8} ", Style::default().fg(Color::Rgb(210, 180, 140))));
            } else if upper_name.contains("MLB") || upper_name.contains("MILB") {
                league_icon_span = Some(ratatui::text::Span::raw("\u{26be} "));
            } else if upper_name.contains("NHL") {
                league_icon_span = Some(ratatui::text::Span::raw("\u{1f3d2} "));
            } else if upper_name.contains("UFC") || upper_name.contains("FIGHT") || upper_name.contains("BOXING") {
                league_icon_span = Some(ratatui::text::Span::raw("\u{1f94a} "));
            } else if upper_name.contains("TENNIS") || upper_name.contains("ATP") || upper_name.contains("WTA") {
                league_icon_span = Some(ratatui::text::Span::styled("\u{1f3be} ", Style::default().fg(Color::Rgb(144, 238, 144))));
            } else if upper_name.contains("GOLF") || upper_name.contains("PGA") || upper_name.contains("MASTERS") {
                league_icon_span = Some(ratatui::text::Span::raw("\u{26f3} "));
            } else if upper_name.contains("SUPERCROSS") || upper_name.contains("MOTOCROSS") || upper_name.contains("F1") || upper_name.contains("NASCAR") || upper_name.contains("RACING") {
                league_icon_span = Some(ratatui::text::Span::styled("\u{1f3ce} ", Style::default().fg(Color::Rgb(255, 100, 100))));
            } else if upper_name.contains("SOCCER") || upper_name.contains("MLS") || upper_name.contains("PREMIER") || upper_name.contains("LALIGA") || upper_name.contains("FIFA") || upper_name.contains("UEFA") {
                league_icon_span = Some(ratatui::text::Span::styled("\u{26bd} ", Style::default().fg(Color::Rgb(255, 182, 193))));
            } else if upper_name.contains("RUGBY") {
                league_icon_span = Some(ratatui::text::Span::raw("\u{1f3c9} "));
            } else if upper_name.contains("EVENT") || upper_name.contains("PPV") {
                league_icon_span = Some(ratatui::text::Span::raw("\u{1f3ab} "));
            } else if upper_name.contains("ESPN") || upper_name.contains("DAZN") || upper_name.contains("B/R") || upper_name.contains("BALLY") {
                league_icon_span = Some(ratatui::text::Span::raw("\u{1f4fa} "));
            }

            if let Some(icon) = league_icon_span {
                 spans.push(icon);
            } else {
                // Extended Auto-Discovery
                let name = parsed.display_name.to_uppercase();
                if name.contains("FOOTBALL") || name.contains("LIGUE") || name.contains("BUNDESLIGA") || name.contains("SERIE A") {
                    spans.push(ratatui::text::Span::styled("\u{26bd} ", Style::default().fg(Color::Rgb(200, 255, 200))));
                } else if name.contains("BASKETBALL") || name.contains("EUROLEAGUE") {
                    spans.push(ratatui::text::Span::styled("\u{1f3c0} ", Style::default().fg(Color::Rgb(255, 140, 0))));
                } else if name.contains("AUTO") || name.contains("MOTOR") {
                    spans.push(ratatui::text::Span::raw("\u{1f3ce} "));
                } else if name.contains("CRICKET") || name.contains("IPL") {
                    spans.push(ratatui::text::Span::raw("\u{1f3cf} "));
                }
            }

            // 2. Channel Number
            if let Some(ref cp) = parsed.channel_prefix {
                 let num_only = cp.chars().filter(|c| c.is_digit(10)).collect::<String>();
                 if !num_only.is_empty() {
                     spans.push(ratatui::text::Span::styled(format!("{}: ", num_only), Style::default().fg(TEXT_DIM)));
                 }
            }

            // 3. Favorite indicator
            let s_id = crate::api::get_id_str(&s.stream_id);
            if app.config.favorites.streams.contains(&s_id) {
                spans.push(ratatui::text::Span::styled("★ ", Style::default().fg(MATRIX_GREEN)));
            }

            // 3. Country Flag
            if let Some(ref country) = parsed.country {
                let is_us_en = country == "US" || country == "USA" || country == "AM" || country == "EN";
                if !(app.config.playlist_mode.is_merica_variant() && is_us_en) {
                    let flag = country_flag(country);
                    if !flag.is_empty() {
                        spans.push(ratatui::text::Span::raw(format!("{} ", flag)));
                    }
                }
            }

            // 4. Name / Matchup
            let is_league = parsed.country.as_ref().map(|cc| ["NBA", "NFL", "MLB", "NHL", "UFC", "SPORTS", "PPV"].contains(&cc.as_str())).unwrap_or(false);
            
            let score_data_for_color = app.get_score_for_stream(&parsed.display_name);
            let name_color = if is_league {
                parsed.country.as_ref().map(|cc| country_color(cc)).unwrap_or(TEXT_PRIMARY)
            } else {
                MATRIX_GREEN
            };
            
            // Icon for VOD/Series Results
            if app.current_screen == CurrentScreen::VodStreams || app.current_screen == CurrentScreen::SeriesStreams {
                let icon = if app.current_screen == CurrentScreen::VodStreams { "◆ " } else { "◆ " };
                spans.insert(0, ratatui::text::Span::styled(icon, Style::default().fg(SOFT_GREEN)));
            }
            
            // SPLIT TEAM COLORING
            if let Some(score) = score_data_for_color {
                let h_color = crate::sports::get_team_color_with_fallback(&score.home_team, true);
                let a_color = crate::sports::get_team_color_with_fallback(&score.away_team, false);
                
                let h_full = format!("{} [{}]", score.home_team, score.home_score);
                let a_full = format!("[{}] {}", score.away_score, score.away_team);

                let width = 28;
                
                let h_display = if h_full.len() > width {
                    let name_avail = width.saturating_sub(score.home_score.len() + 3);
                    let truncated_name = if score.home_team.len() > name_avail {
                        format!("{}..", &score.home_team[..name_avail.saturating_sub(2)])
                    } else {
                        score.home_team.clone()
                    };
                    format!("{:>width$}", format!("{} [{}]", truncated_name, score.home_score), width=width)
                } else {
                    format!("{:>width$}", h_full, width=width)
                };

                let a_display = if a_full.len() > width {
                    let name_avail = width.saturating_sub(score.away_score.len() + 3);
                    let truncated_name = if score.away_team.len() > name_avail {
                        format!("{}..", &score.away_team[..name_avail.saturating_sub(2)])
                    } else {
                        score.away_team.clone()
                    };
                    format!("{:<width$}", format!("[{}] {}", score.away_score, truncated_name), width=width)
                } else {
                    format!("{:<width$}", a_full, width=width)
                };

                spans.push(ratatui::text::Span::styled(h_display, Style::default().fg(h_color).add_modifier(Modifier::BOLD)));
                spans.push(ratatui::text::Span::styled(" vs ", Style::default().fg(TEXT_DIM)));
                spans.push(ratatui::text::Span::styled(a_display, Style::default().fg(a_color).add_modifier(Modifier::BOLD)));
                
                 if let Some(q) = &parsed.quality {
                     spans.push(ratatui::text::Span::styled(format!("   {:?}", q), Style::default().fg(TEXT_DIM)));
                 }
                 
                 spans.push(ratatui::text::Span::raw("   "));
            } else {
                // Standard rendering
                let max_width = area.width.saturating_sub(4) as usize; 
                let mut display_name = parsed.display_name.clone();
                let mut year_suffix = String::new();
                
                if let Some(y) = &parsed.year {
                    if !y.is_empty() {
                        let y_str = format!(" [{}]", y);
                        let total_len = display_name.len() + y_str.len();
                        
                        if total_len > max_width {
                            let name_avail = max_width.saturating_sub(y_str.len());
                            if name_avail > 3 {
                                display_name = format!("{}...", &display_name[..name_avail.saturating_sub(3)]);
                            } else {
                                display_name = display_name.chars().take(name_avail).collect();
                            }
                        }
                        year_suffix = y_str;
                    }
                } else {
                    if display_name.len() > max_width {
                         if max_width > 3 {
                             display_name = format!("{}...", &display_name[..max_width.saturating_sub(3)]);
                         } else {
                             display_name = display_name.chars().take(max_width).collect();
                         }
                    }
                }

                let (mut styled_name, _) = stylize_channel_name(
                    &display_name,
                    false,
                    is_ended,
                    parsed.quality,
                    None,
                    parsed.sports_event.as_ref(),
                    Style::default().fg(name_color),
                );
                
                if !year_suffix.is_empty() {
                   styled_name.push(ratatui::text::Span::styled(year_suffix, Style::default().fg(TEXT_SECONDARY)));
                }
                
                spans.extend(styled_name);
            }

            // 5. Score/Clock Logic
            let score_data = app.get_score_for_stream(&parsed.display_name);
            
            let (final_is_live, final_is_ended, status_text, score_text) = if let Some(score) = score_data {
                let is_active = score.status_state == "in";
                let is_finished = score.status_state == "post";
                let clock = if is_finished {
                    "Final".to_string()
                } else if is_active {
                     if !score.display_clock.is_empty() && score.display_clock != "00:00" {
                         score.display_clock.split(' ').next().unwrap_or(&score.display_clock).to_string()
                     } else {
                         "LIVE".to_string()
                     }
                } else {
                    score.status_detail.clone()
                };
                let display_score = format!("[{}-{}]", score.home_score, score.away_score);
                (is_active, is_finished, Some(clock), Some(display_score))
            } else {
                (is_live, is_ended, None, None)
            };

            if score_data.is_none() {
                if let Some(clock) = status_text {
                     let mut clock_style = Style::default().fg(Color::Rgb(255, 200, 80)).add_modifier(Modifier::BOLD);
                     if final_is_ended {
                         clock_style = Style::default().fg(TEXT_DIM).add_modifier(Modifier::CROSSED_OUT); 
                     }
                     spans.push(ratatui::text::Span::styled(format!(" [{}] ", clock), clock_style));
                } else if let Some(st) = parsed.start_time {
                    let time_str = format!(" {} ", format_relative_time(st, &user_tz));
                    let mut time_style = Style::default().fg(TEXT_SECONDARY);
                    if final_is_ended {
                        time_style = time_style.add_modifier(Modifier::CROSSED_OUT);
                    }
                    spans.push(ratatui::text::Span::styled(time_str, time_style));
                }
            }

            if score_data.is_none() {
                if let Some(score) = score_text {
                    spans.push(ratatui::text::Span::styled(score, Style::default().fg(TEXT_PRIMARY).add_modifier(Modifier::BOLD)));
                }
            }

            // 6. Live / Ended Badges
            if final_is_live {
                let live_color = if is_blink_on { Color::Rgb(255, 100, 100) } else { TEXT_DIM };
                let dot = if is_blink_on { "● " } else { "  " };
                spans.push(ratatui::text::Span::styled(dot, Style::default().fg(live_color)));
                spans.push(ratatui::text::Span::styled("LIVE", Style::default().fg(live_color).add_modifier(Modifier::BOLD)));
            } else if final_is_ended {
                spans.push(ratatui::text::Span::styled(" ended", Style::default().fg(TEXT_DIM)));
            }

            // 7. Location
            if parsed.sports_event.is_none() {
                if let Some(loc) = &parsed.location {
                    spans.push(ratatui::text::Span::styled(format!(" ({})", loc), Style::default().fg(TEXT_SECONDARY)));
                }
            }

            // 8. Network Health
            spans.push(latency_to_bars(s.latency_ms));

            ListItem::new(Line::from(spans))
        })
        .collect();

    let title = format!(" streams ({}) · {} ", app.streams.len(), tz_display);
    let inner_area = crate::ui::common::render_matrix_box(f, area, &title, border_color);

    let list = List::new(items)
        .highlight_style(list_highlight_style())
        .highlight_symbol(" ▎");

    let mut adjusted_state = app.stream_list_state.clone();
    if adjusted_start > 0 {
        adjusted_state.select(Some(selected - adjusted_start));
    }

    f.render_stateful_widget(list, inner_area, &mut adjusted_state);
}

pub fn render_global_search_pane(f: &mut Frame, app: &mut App, area: Rect) {
    let visible_height = area.height.saturating_sub(2) as usize;
    let total = app.global_search_results.len();
    let selected = app.selected_stream_index;

    let half_window = visible_height / 2;
    let start = if selected > half_window {
        selected - half_window
    } else {
        0
    };
    let end = (start + visible_height + half_window).min(total);
    let adjusted_start = if end == total && end > visible_height + half_window {
        end.saturating_sub(visible_height + half_window)
    } else {
        start
    };

    let user_tz_str = app.config.get_user_timezone();
    let user_tz: Tz = user_tz_str.parse().unwrap_or(chrono_tz::UTC);

    let items: Vec<ListItem> = app
        .global_search_results
        .iter()
        .enumerate()
        .skip(adjusted_start)
        .take(end - adjusted_start)
        .map(|(_, s)| {
            let parsed = crate::parser::parse_stream(&s.name, app.provider_timezone.as_deref());
            let mut spans = vec![];
            
            // Clean type icon
            let icon = match s.stream_type.as_str() {
                "movie" => "◆ ",
                "series" => "◇ ",
                _ => "  ",
            };
            spans.push(ratatui::text::Span::styled(icon, Style::default().fg(SOFT_GREEN)));

            let (mut styled_name, _) = stylize_channel_name(
                &parsed.display_name,
                false,
                false,
                parsed.quality,
                None,
                parsed.sports_event.as_ref(),
                Style::default().fg(MATRIX_GREEN),
            );
            
            if let Some(y) = &parsed.year {
                if !y.is_empty() {
                    styled_name.push(ratatui::text::Span::styled(format!(" [{}]", y), Style::default().fg(TEXT_SECONDARY)));
                }
            }
            
            spans.extend(styled_name);

            if s.stream_type == "live" {
                if let Some(st) = parsed.start_time {
                    let time_str = format!(" {} ", format_relative_time(st, &user_tz));
                    spans.push(ratatui::text::Span::styled(time_str, Style::default().fg(TEXT_SECONDARY)));
                }
            }

            spans.push(latency_to_bars(s.latency_ms));

            if let Some(account) = &s.account_name {
                spans.push(ratatui::text::Span::styled(format!(" [{}]", account), Style::default().fg(TEXT_DIM)));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    let title = if app.search_state.query.is_empty() {
        format!(" search ({}) — type to filter ", app.global_search_results.len())
    } else {
        format!(" search: \"{}\" ({}) ", app.search_state.query, app.global_search_results.len())
    };
    let list = List::new(items)
        .block(
            Block::default()
                .title(Span::styled(format!(" {} ", title), Style::default().fg(SOFT_GREEN).add_modifier(Modifier::BOLD)))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(SOFT_GREEN)),
        )
        .highlight_style(list_highlight_style())
        .highlight_symbol(" ▎");

    let mut adjusted_state = app.global_search_list_state.clone();
    if adjusted_start > 0 {
        adjusted_state.select(Some(selected - adjusted_start));
    }

    f.render_stateful_widget(list, area, &mut adjusted_state);
}

fn latency_to_bars(latency: Option<u64>) -> ratatui::text::Span<'static> {
    match latency {
        Some(l) if l < 200 => ratatui::text::Span::styled(" ▰▰▰", Style::default().fg(MATRIX_GREEN)),
        Some(l) if l < 600 => ratatui::text::Span::styled(" ▰▰░", Style::default().fg(Color::Rgb(255, 200, 80))),
        Some(_) => ratatui::text::Span::styled(" ▰░░", Style::default().fg(Color::Rgb(255, 100, 100))),
        None => ratatui::text::Span::raw(""),
    }
}

pub fn render_stream_details_pane(f: &mut Frame, app: &mut App, area: Rect, border_color: Color) {
    let selected_idx = app.selected_stream_index;
    let focused_stream = if app.current_screen == CurrentScreen::GlobalSearch {
        app.global_search_results.get(selected_idx)
    } else {
        app.streams.get(selected_idx)
    };

    let Some(s) = focused_stream else { return };
    let parsed = parse_stream(&s.name, app.provider_timezone.as_deref());
    
    let score_data = app.get_score_for_stream(&parsed.display_name);
    
    let event = parsed.sports_event.clone();
    if event.is_none() && score_data.is_none() { return }
    
    let (team1, team2) = if let Some(ref ev) = event {
        (ev.team1.clone(), ev.team2.clone())
    } else if let Some(ref sd) = score_data {
        (sd.home_team.clone(), sd.away_team.clone())
    } else {
        return;
    };

    let title = " match intelligence ";
    let inner_area = crate::ui::common::render_matrix_box(f, area, title, border_color);

    let sub_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(55),
            Constraint::Percentage(45),
        ])
        .split(inner_area);

    // --- LEFT SIDE ---
    let mut left = Vec::new();
    
    let (h_score, a_score, _h_abbr, _a_abbr) = if let Some(score) = score_data {
        (score.home_score.as_str(), score.away_score.as_str(), 
         score.home_abbr.as_str(), score.away_abbr.as_str())
    } else {
        ("-", "-", "", "")
    };
    
    let h_num: i32 = h_score.parse().unwrap_or(0);
    let a_num: i32 = a_score.parse().unwrap_or(0);
    
    let is_game_active = score_data.map(|s| s.status_state == "in" || s.status_state == "post").unwrap_or(false);
    let has_scoring = h_num > 0 || a_num > 0;

    if let Some(score) = score_data {
        if score.status_state == "in" {
            let display_clock = if !score.display_clock.is_empty() && score.display_clock != "00:00" {
                format!("{} — {}", score.display_clock, score.status_detail)
            } else {
                score.status_detail.clone()
            };
            
            left.push(Line::from(vec![
                Span::styled("⏱ ", Style::default().fg(Color::Rgb(255, 100, 100))),
                Span::styled(display_clock, Style::default().fg(Color::Rgb(255, 150, 150)).add_modifier(Modifier::BOLD)),
            ]));
        } else if score.status_state == "post" {
            left.push(Line::from(vec![
                Span::styled("✓ final", Style::default().fg(TEXT_SECONDARY)),
            ]));
        }
    }

    let mut team1_spans = vec![
        Span::styled(&team1, Style::default().fg(crate::sports::get_team_color_with_fallback(&team1, true)).add_modifier(Modifier::BOLD)),
    ];
    if has_scoring || is_game_active {
        team1_spans.push(Span::styled(format!("  {} ", h_score), Style::default().fg(Color::Rgb(255, 200, 80)).add_modifier(Modifier::BOLD)));
    }
    left.push(Line::from(team1_spans));
    
    let mut team2_spans = vec![
        Span::styled(&team2, Style::default().fg(crate::sports::get_team_color_with_fallback(&team2, false)).add_modifier(Modifier::BOLD)),
    ];
    if has_scoring || is_game_active {
        team2_spans.push(Span::styled(format!("  {} ", a_score), Style::default().fg(Color::Rgb(255, 200, 80)).add_modifier(Modifier::BOLD)));
    }
    left.push(Line::from(team2_spans));

    f.render_widget(Paragraph::new(left), sub_chunks[0]);

    // --- RIGHT SIDE ---
    let mut right = Vec::new();
    
    if let Some(score) = score_data {
        if let (Some(hwp), Some(awp)) = (score.home_win_pct, score.away_win_pct) {
            right.push(Line::from(vec![
                Span::styled("odds ", Style::default().fg(TEXT_SECONDARY)),
                Span::styled(format!("{} ", score.home_abbr), Style::default().fg(TEXT_PRIMARY)),
                Span::styled(format!("{:.0}%", hwp * 100.0), Style::default().fg(if hwp > 0.5 { MATRIX_GREEN } else { TEXT_SECONDARY })),
                Span::styled(" · ", Style::default().fg(TEXT_DIM)),
                Span::styled(format!("{} ", score.away_abbr), Style::default().fg(TEXT_PRIMARY)),
                Span::styled(format!("{:.0}%", awp * 100.0), Style::default().fg(if awp > 0.5 { MATRIX_GREEN } else { TEXT_SECONDARY })),
            ]));
        }
        
        if let Some(series) = &score.series_summary {
            right.push(Line::from(vec![
                Span::styled("series ", Style::default().fg(TEXT_SECONDARY)),
                Span::styled(series, Style::default().fg(TEXT_PRIMARY)),
            ]));
        }
        
        if let Some(scorer) = &score.top_scorer {
            right.push(Line::from(vec![
                Span::styled("star ", Style::default().fg(TEXT_SECONDARY)),
                Span::styled(scorer, Style::default().fg(TEXT_PRIMARY)),
            ]));
        }
        
        if score.status_state == "post" {
            if let Some(hl) = &score.headline {
                let truncated = if hl.len() > 45 { format!("{}...", &hl[..42]) } else { hl.clone() };
                right.push(Line::from(vec![
                    Span::styled("recap ", Style::default().fg(TEXT_SECONDARY)),
                    Span::styled(truncated, Style::default().fg(TEXT_PRIMARY)),
                ]));
            }
        }
        
        if score.status_state == "in" {
            if let Some(lp) = &score.last_play {
                let truncated = if lp.len() > 35 { format!("{}...", &lp[..32]) } else { lp.clone() };
                right.push(Line::from(vec![
                    Span::styled("▶ ", Style::default().fg(MATRIX_GREEN)),
                    Span::styled(truncated, Style::default().fg(TEXT_PRIMARY)),
                ]));
            }
        }
        
        if !score.broadcasts.is_empty() {
            let channels = score.broadcasts.iter().take(2).cloned().collect::<Vec<_>>().join(", ");
            right.push(Line::from(vec![
                Span::styled("tv ", Style::default().fg(TEXT_SECONDARY)),
                Span::styled(channels, Style::default().fg(TEXT_PRIMARY)),
            ]));
        }
    } else {
        let user_tz_str = app.config.get_user_timezone();
        let user_tz: Tz = user_tz_str.parse().unwrap_or(chrono_tz::UTC);
        
        if let Some(st) = parsed.start_time {
            let time_str = format_relative_time(st, &user_tz);
            right.push(Line::from(vec![
                Span::styled("⏰ ", Style::default().fg(MATRIX_GREEN)),
                Span::styled(time_str, Style::default().fg(MATRIX_GREEN)),
            ]));
        }
    }

    right.push(Line::from(vec![
        Span::styled("enter", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
        Span::styled(" to watch", Style::default().fg(TEXT_SECONDARY)),
    ]));
    f.render_widget(Paragraph::new(right), sub_chunks[1]);
}
