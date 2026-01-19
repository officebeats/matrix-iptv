use ratatui::{
    layout::{Rect, Layout, Constraint, Direction},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
    Frame,
};
use crate::app::{App, CurrentScreen};
use crate::parser::{parse_category, parse_stream, country_flag, country_color};
use crate::ui::colors::MATRIX_GREEN;
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
    
    format!("[{} {}]", day, local.format("%I:%M %p"))
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
                parsed.country.as_ref().map(|cc| country_color(cc)).unwrap_or(Color::White)
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

            // Icon & Name Style
            let (icon, icon_color) = if app.current_screen == CurrentScreen::VodCategories || app.current_screen == CurrentScreen::VodStreams {
                ("🎬", MATRIX_GREEN)
            } else if app.current_screen == CurrentScreen::SeriesCategories || app.current_screen == CurrentScreen::SeriesStreams {
                ("📺", MATRIX_GREEN)
            } else {
                ("📁", Color::White)
            };
            spans.push(ratatui::text::Span::styled(format!("{} ", icon), Style::default().fg(icon_color)));

            spans.extend(styled_name);

            ListItem::new(Line::from(spans))
        })
        .collect();

    let title = if app.categories.is_empty() {
        " // CATEGORIES / [NULL] ".to_string()
    } else {
        format!(" // CATEGORIES ({}) ", app.categories.len())
    };
    let inner_area = crate::ui::common::render_matrix_box(f, area, &title, border_color);
    
    let list = List::new(items)
        .highlight_style(
            Style::default()
                .bg(MATRIX_GREEN)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" » ");

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
                league_icon_span = Some(ratatui::text::Span::styled("\u{1f3c0} ", Style::default().fg(Color::Rgb(255, 140, 0)))); // Basketball (orange)
            } else if upper_name.contains("NFL") || upper_name.contains("NCAAF") {
                league_icon_span = Some(ratatui::text::Span::styled("\u{1f3c8} ", Style::default().fg(Color::Rgb(210, 180, 140)))); // Football (light brown)
            } else if upper_name.contains("MLB") || upper_name.contains("MILB") {
                league_icon_span = Some(ratatui::text::Span::raw("\u{26be} ")); // Baseball
            } else if upper_name.contains("NHL") {
                league_icon_span = Some(ratatui::text::Span::raw("\u{1f3d2} ")); // Hockey
            } else if upper_name.contains("UFC") || upper_name.contains("FIGHT") || upper_name.contains("BOXING") {
                league_icon_span = Some(ratatui::text::Span::raw("\u{1f94a} ")); // Boxing glove
            } else if upper_name.contains("TENNIS") || upper_name.contains("ATP") || upper_name.contains("WTA") {
                league_icon_span = Some(ratatui::text::Span::styled("\u{1f3be} ", Style::default().fg(Color::Rgb(144, 238, 144)))); // Tennis (light green)
            } else if upper_name.contains("GOLF") || upper_name.contains("PGA") || upper_name.contains("MASTERS") {
                league_icon_span = Some(ratatui::text::Span::raw("\u{26f3} ")); // Golf flag
            } else if upper_name.contains("SUPERCROSS") || upper_name.contains("MOTOCROSS") || upper_name.contains("F1") || upper_name.contains("NASCAR") || upper_name.contains("RACING") {
                league_icon_span = Some(ratatui::text::Span::styled("\u{1f3ce} ", Style::default().fg(Color::Rgb(255, 100, 100)))); // Racing car (light red)
            } else if upper_name.contains("SOCCER") || upper_name.contains("MLS") || upper_name.contains("PREMIER") || upper_name.contains("LALIGA") || upper_name.contains("FIFA") || upper_name.contains("UEFA") {
                league_icon_span = Some(ratatui::text::Span::styled("\u{26bd} ", Style::default().fg(Color::Rgb(255, 182, 193)))); // Soccer (light pink)
            } else if upper_name.contains("RUGBY") {
                league_icon_span = Some(ratatui::text::Span::raw("\u{1f3c9} ")); // Rugby football
            } else if upper_name.contains("EVENT") || upper_name.contains("PPV") {
                league_icon_span = Some(ratatui::text::Span::raw("\u{1f3ab} ")); // Ticket
            } else if upper_name.contains("ESPN") || upper_name.contains("DAZN") || upper_name.contains("B/R") || upper_name.contains("BALLY") {
                league_icon_span = Some(ratatui::text::Span::raw("\u{1f4fa} ")); // TV (sports network)
            }

            if let Some(icon) = league_icon_span {
                 spans.push(icon);
            }

            // 2. Channel Number
            if let Some(ref cp) = parsed.channel_prefix {
                 let num_only = cp.chars().filter(|c| c.is_digit(10)).collect::<String>();
                 if !num_only.is_empty() {
                     spans.push(ratatui::text::Span::styled(format!("{}: ", num_only), Style::default().fg(Color::Gray)));
                 }
            }

            // 3. Favorite indicator
            let s_id = crate::api::get_id_str(&s.stream_id);
            if app.config.favorites.streams.contains(&s_id) {
                spans.push(ratatui::text::Span::styled("★ ", Style::default().fg(MATRIX_GREEN)));
            }

            // 3. Country Flag
            if let Some(ref country) = parsed.country {
                // In American Mode, don't show the US or EN flag (redundant)
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
            
            // Try to get team color from ESPN data first, otherwise use league color
            let score_data_for_color = app.get_score_for_stream(&parsed.display_name);
            let name_color = if let Some(score) = score_data_for_color {
                // Use home team color from ESPN (already lightened by get_team_color_with_fallback)
                crate::sports::get_team_color_with_fallback(&score.home_team, true)
            } else if is_league {
                parsed.country.as_ref().map(|cc| country_color(cc)).unwrap_or(Color::White)
            } else {
                MATRIX_GREEN
            };
            
            // Icon for VOD/Series Results
            if app.current_screen == CurrentScreen::VodStreams || app.current_screen == CurrentScreen::SeriesStreams {
                let icon = if app.current_screen == CurrentScreen::VodStreams { "🎬 " } else { "📺 " };
                spans.insert(0, ratatui::text::Span::styled(icon, Style::default().fg(MATRIX_GREEN)));
            }
            
            let (styled_name, _) = stylize_channel_name(
                &parsed.display_name,
                false,
                is_ended,
                parsed.quality,
                None,
                parsed.sports_event.as_ref(),
                Style::default().fg(name_color),
            );
            spans.extend(styled_name);

            // 5. Intelligent Score Overlay (ESPN Integration)
            // If we have a live score match, use it to drive the UI state (Live/Ended/Score)
            let score_data = app.get_score_for_stream(&parsed.display_name);
            
            // Logic override based on Score Data
            let (final_is_live, final_is_ended, status_text, score_text) = if let Some(score) = score_data {
                let is_active = score.status_state == "in";
                let is_finished = score.status_state == "post";
                let clock = if is_finished {
                    "Final".to_string()
                } else if is_active {
                     // Use specific clock if available, otherwise fallback to detail
                     if !score.display_clock.is_empty() && score.display_clock != "00:00" {
                         format!("{} {}", score.display_clock, score.status_detail.replace(&score.display_clock, "").trim())
                     } else {
                         score.status_detail.clone()
                     }
                } else {
                    score.status_detail.clone() // e.g. "12:00 1st"
                };
                let display_score = format!(" {} - {} ", score.home_score, score.away_score);
                (is_active, is_finished, Some(clock), Some(display_score))
            } else {
                (is_live, is_ended, None, None)
            };

            // 5b. Time Reference (or Game Clock)
            if let Some(clock) = status_text {
                 // Show Game Clock instead of Start Time
                 let mut clock_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
                 if final_is_ended {
                     clock_style = Style::default().fg(Color::Red).add_modifier(Modifier::CROSSED_OUT); 
                 }
                 spans.push(ratatui::text::Span::styled(format!(" [{}] ", clock), clock_style));
            } else if let Some(st) = parsed.start_time {
                let time_str = format!(" {} ", format_relative_time(st, &user_tz));
                let mut time_style = Style::default().fg(Color::Rgb(150, 150, 150));
                if final_is_ended {
                    time_style = time_style.add_modifier(Modifier::CROSSED_OUT);
                }
                spans.push(ratatui::text::Span::styled(time_str, time_style));
            }

            // 5c. Score Display
            if let Some(score) = score_text {
                spans.push(ratatui::text::Span::styled(score, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)));
            }

            // 6. Live / Ended Badges
            let light_red = Color::Rgb(255, 120, 120);
            let dim_gray = Color::Rgb(60, 60, 60);
            
            if final_is_live {
                let live_color = if is_blink_on { light_red } else { dim_gray };
                let dot = if is_blink_on { "\u{1f534} " } else { "  " };
                spans.push(ratatui::text::Span::styled(dot, Style::default().fg(live_color)));
                spans.push(ratatui::text::Span::styled("LIVE NOW ", Style::default().fg(live_color).add_modifier(Modifier::BOLD)));
            } else if final_is_ended {
                spans.push(ratatui::text::Span::styled("[ENDED] ", Style::default().fg(Color::Rgb(100, 100, 100)).add_modifier(Modifier::BOLD)));
            }

            // 7. Location (LOC) at the end - Skip for sports events (team abbrs handled by Intelligence)
            if parsed.sports_event.is_none() {
                if let Some(loc) = &parsed.location {
                    spans.push(ratatui::text::Span::styled(format!("({})", loc), Style::default().fg(Color::LightBlue)));
                }
            }

            // 8. Network Health (Latency)
            spans.push(latency_to_bars(s.latency_ms));

            ListItem::new(Line::from(spans))
        })
        .collect();

    let title = format!(" // STREAMS ({}) / TZ: {} ", app.streams.len(), tz_display);
    let inner_area = crate::ui::common::render_matrix_box(f, area, &title, border_color);

    let list = List::new(items)
        .highlight_style(
            Style::default()
                .bg(MATRIX_GREEN)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" » ");

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
            
            // Icon & Name Style
            let (icon, icon_color) = match s.stream_type.as_str() {
                "movie" => ("🎬 ", MATRIX_GREEN),
                "series" => ("📺 ", MATRIX_GREEN),
                _ => ("", MATRIX_GREEN),
            };
            spans.push(ratatui::text::Span::styled(icon, Style::default().fg(icon_color)));

            // Name
            let (styled_name, _) = stylize_channel_name(
                &parsed.display_name,
                false,
                false,
                parsed.quality,
                None,
                parsed.sports_event.as_ref(),
                Style::default().fg(MATRIX_GREEN),
            );
            spans.extend(styled_name);

            // Time Reference (Live only)
            if s.stream_type == "live" {
                if let Some(st) = parsed.start_time {
                    let time_str = format!(" {} ", format_relative_time(st, &user_tz));
                    spans.push(ratatui::text::Span::styled(time_str, Style::default().fg(Color::Rgb(150, 150, 150))));
                }
            }

            // Latency
            spans.push(latency_to_bars(s.latency_ms));

            // Playlist Source
            if let Some(account) = &s.account_name {
                spans.push(ratatui::text::Span::styled(format!(" [{}]", account), Style::default().fg(Color::DarkGray)));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    let title = format!(" // GLOBAL_SEARCH ({}) - Type to Filter / Enter to Play ", app.global_search_results.len());
    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(Style::default().fg(MATRIX_GREEN)),
        )
        .highlight_style(
            Style::default()
                .bg(MATRIX_GREEN)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" » ");

    let mut adjusted_state = app.global_search_list_state.clone();
    if adjusted_start > 0 {
        adjusted_state.select(Some(selected - adjusted_start));
    }

    f.render_stateful_widget(list, area, &mut adjusted_state);
}

fn latency_to_bars(latency: Option<u64>) -> ratatui::text::Span<'static> {
    match latency {
        Some(l) if l < 200 => ratatui::text::Span::styled(" [📶📶📶]", Style::default().fg(Color::Green)),
        Some(l) if l < 600 => ratatui::text::Span::styled(" [📶📶  ]", Style::default().fg(Color::Yellow)),
        Some(_) => ratatui::text::Span::styled(" [📶    ]", Style::default().fg(Color::Red)),
        None => ratatui::text::Span::raw(""), // No latency data - hide indicator
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
    
    // Attempt to get Live Score Info if available
    let score_data = app.get_score_for_stream(&parsed.display_name);
    
    // Need either a parsed sports event OR ESPN score data to render
    let event = parsed.sports_event.clone();
    if event.is_none() && score_data.is_none() { return }
    
    // Create synthetic event from ESPN data if no parsed event
    let (team1, team2) = if let Some(ref ev) = event {
        (ev.team1.clone(), ev.team2.clone())
    } else if let Some(ref sd) = score_data {
        (sd.home_team.clone(), sd.away_team.clone())
    } else {
        return;
    };

    let title = " // MATCH_INTELLIGENCE ";
    let inner_area = crate::ui::common::render_matrix_box(f, area, title, border_color);

    let sub_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(55),
            Constraint::Percentage(45),
        ])
        .split(inner_area);

    // --- LEFT SIDE: TEAMS, SCORE & VISUAL BAR ---
    let mut left = Vec::new();
    
    // Scores if available
    let (h_score, a_score, _h_abbr, _a_abbr) = if let Some(score) = score_data {
        (score.home_score.as_str(), score.away_score.as_str(), 
         score.home_abbr.as_str(), score.away_abbr.as_str())
    } else {
        ("-", "-", "", "")
    };
    
    // Parse scores for lead indicator
    let h_num: i32 = h_score.parse().unwrap_or(0);
    let a_num: i32 = a_score.parse().unwrap_or(0);
    
    let is_game_active = score_data.map(|s| s.status_state == "in" || s.status_state == "post").unwrap_or(false);
    let has_scoring = h_num > 0 || a_num > 0;

    // 1. Status Line (Top Left - Moved from Right)
    if let Some(score) = score_data {
        if score.status_state == "in" {
            // Live Game with clock
            let display_clock = if !score.display_clock.is_empty() && score.display_clock != "00:00" {
                format!("{} - {}", score.display_clock, score.status_detail)
            } else {
                score.status_detail.clone()
            };
            
            left.push(Line::from(vec![
                Span::styled("⏱ CLOCK: ", Style::default().fg(Color::Rgb(255, 120, 120))),
                Span::styled(display_clock, Style::default().fg(Color::Rgb(255, 150, 150)).add_modifier(Modifier::BOLD)),
            ]));
        } else if score.status_state == "post" {
            left.push(Line::from(vec![
                Span::styled("✓ FINAL", Style::default().fg(Color::Rgb(180, 180, 180))),
            ]));
        }
    }

    // Team rows with score
    let mut team1_spans = vec![
        Span::styled(&team1, Style::default().fg(crate::sports::get_team_color_with_fallback(&team1, true)).add_modifier(Modifier::BOLD)),
    ];
    if has_scoring || is_game_active {
        team1_spans.push(Span::styled(format!("  {} ", h_score), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
    }
    left.push(Line::from(team1_spans));
    
    let mut team2_spans = vec![
        Span::styled(&team2, Style::default().fg(crate::sports::get_team_color_with_fallback(&team2, false)).add_modifier(Modifier::BOLD)),
    ];
    if has_scoring || is_game_active {
        team2_spans.push(Span::styled(format!("  {} ", a_score), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
    }
    left.push(Line::from(team2_spans));

    f.render_widget(Paragraph::new(left), sub_chunks[0]);

    // --- RIGHT SIDE: INTELLIGENCE DATA ---
    let mut right = Vec::new();
    
    // Win Probability / Betting Odds (its own line item)
    if let Some(score) = score_data {
        
        // Win Probability / Betting Odds (its own line item)
        if let (Some(hwp), Some(awp)) = (score.home_win_pct, score.away_win_pct) {
            right.push(Line::from(vec![
                Span::styled("📊 ODDS: ", Style::default().fg(Color::Rgb(150, 200, 255))),
                Span::styled(format!("{} ", score.home_abbr), Style::default().fg(Color::Rgb(200, 220, 255))),
                Span::styled(format!("{:.0}%", hwp * 100.0), Style::default().fg(if hwp > 0.5 { Color::Rgb(150, 255, 150) } else { Color::Rgb(180, 180, 180) })),
                Span::styled(" | ", Style::default().fg(Color::Rgb(100, 100, 100))),
                Span::styled(format!("{} ", score.away_abbr), Style::default().fg(Color::Rgb(200, 220, 255))),
                Span::styled(format!("{:.0}%", awp * 100.0), Style::default().fg(if awp > 0.5 { Color::Rgb(150, 255, 150) } else { Color::Rgb(180, 180, 180) })),
            ]));
        }
        
        // Series Summary (for playoffs)
        if let Some(series) = &score.series_summary {
            right.push(Line::from(vec![
                Span::styled("🏆 SERIES: ", Style::default().fg(Color::Rgb(255, 220, 100))),
                Span::styled(series, Style::default().fg(Color::Rgb(255, 230, 150))),
            ]));
        }
        
        // Top Scorer / Game Leader
        if let Some(scorer) = &score.top_scorer {
            right.push(Line::from(vec![
                Span::styled("⭐ STAR: ", Style::default().fg(Color::Rgb(255, 150, 255))),
                Span::styled(scorer, Style::default().fg(Color::Rgb(255, 180, 255))),
            ]));
        }
        
        // Headline (post-game recap)
        if score.status_state == "post" {
            if let Some(hl) = &score.headline {
                let truncated = if hl.len() > 45 { format!("{}...", &hl[..42]) } else { hl.clone() };
                right.push(Line::from(vec![
                    Span::styled("📰 RECAP: ", Style::default().fg(Color::Rgb(220, 220, 220))),
                    Span::styled(truncated, Style::default().fg(Color::Rgb(240, 240, 240))),
                ]));
            }
        }
        
        // Last Play (live games)
        if score.status_state == "in" {
            if let Some(lp) = &score.last_play {
                let truncated = if lp.len() > 35 { format!("{}...", &lp[..32]) } else { lp.clone() };
                right.push(Line::from(vec![
                    Span::styled("▶ PLAY: ", Style::default().fg(Color::Rgb(100, 220, 255))),
                    Span::styled(truncated, Style::default().fg(Color::Rgb(150, 230, 255))),
                ]));
            }
        }
        
        // Broadcasts
        if !score.broadcasts.is_empty() {
            let channels = score.broadcasts.iter().take(2).cloned().collect::<Vec<_>>().join(", ");
            right.push(Line::from(vec![
                Span::styled("📺 TV: ", Style::default().fg(Color::Rgb(120, 180, 255))),
                Span::styled(channels, Style::default().fg(Color::Rgb(150, 200, 255))),
            ]));
        }
    } else {
        // No live data - show start time if available
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
        Span::styled("[Enter] ", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
        Span::styled("Watch", Style::default().fg(Color::Gray)),
    ]));
    f.render_widget(Paragraph::new(right), sub_chunks[1]);
}
