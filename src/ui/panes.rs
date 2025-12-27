use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, BorderType, Borders, List, ListItem},
    Frame,
};
use crate::app::{App, CurrentScreen};
use crate::parser::{parse_category, country_flag, country_color, parse_stream};
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

            // Icon logic: ONLY show for sports/VIP/Exact Matches
            let mut icon_span = None;
            let upper_display = parsed.display_name.to_uppercase();
            
            if let Some(ct) = parsed.content_type {
                let icon = ct.icon();
                if !icon.is_empty() {
                    icon_span = Some(ratatui::text::Span::raw(format!("{} ", icon)));
                }
            }
            
            // Comprehensive sports mapping (overrides or supplements generic icons)
            // Exclude false positives like '24/7 ONEPLAY' which is not sports
            let is_24_7 = upper_display.starts_with("24/7") || upper_display.contains("ONEPLAY") || upper_display.contains("RELAX");
            if !is_24_7 {
                if upper_display.contains("NBA") || upper_display.contains("NCAAB") || upper_display.contains("BASKETBALL") {
                     icon_span = Some(ratatui::text::Span::styled("\u{1f3c0} ", Style::default().fg(Color::Rgb(255, 140, 0)))); // Basketball (orange)
                } else if upper_display.contains("NFL") || upper_display.contains("NCAAF") {
                     icon_span = Some(ratatui::text::Span::styled("\u{1f3c8} ", Style::default().fg(Color::Rgb(210, 180, 140)))); // Football (light brown)
                } else if upper_display.contains("MLB") || upper_display.contains("MILB") || upper_display.contains("BASEBALL") {
                     icon_span = Some(ratatui::text::Span::raw("\u{26be} ")); // Baseball
                } else if upper_display.contains("NHL") || upper_display.contains("HOCKEY") {
                     icon_span = Some(ratatui::text::Span::raw("\u{1f3d2} ")); // Hockey
                } else if upper_display.contains("UFC") || upper_display.contains("FIGHT") || upper_display.contains("MATCHROOM") || upper_display.contains("BOXING") {
                     icon_span = Some(ratatui::text::Span::raw("\u{1f94a} ")); // Boxing glove
                } else if upper_display.contains("TENNIS") || upper_display.contains("ATP") || upper_display.contains("WTA") {
                     icon_span = Some(ratatui::text::Span::styled("\u{1f3be} ", Style::default().fg(Color::Rgb(144, 238, 144)))); // Tennis (light green)
                } else if upper_display.contains("GOLF") || upper_display.contains("PGA") || upper_display.contains("MASTERS") {
                     icon_span = Some(ratatui::text::Span::raw("\u{26f3} ")); // Golf flag
                } else if upper_display.contains("SUPERCROSS") || upper_display.contains("MOTOCROSS") || upper_display.contains("F1") || upper_display.contains("NASCAR") || upper_display.contains("RACING") {
                     icon_span = Some(ratatui::text::Span::styled("\u{1f3ce} ", Style::default().fg(Color::Rgb(255, 100, 100)))); // Racing car (light red)
                } else if upper_display.contains("SOCCER") || upper_display.contains("EPL") || upper_display.contains("MLS") || upper_display.contains("PREMIER") || upper_display.contains("LALIGA") || upper_display.contains("FIFA") || upper_display.contains("UEFA") {
                     icon_span = Some(ratatui::text::Span::styled("\u{26bd} ", Style::default().fg(Color::Rgb(255, 182, 193)))); // Soccer ball (light pink)
                } else if upper_display.contains("RUGBY") || upper_display.contains("SIX NATIONS") {
                     icon_span = Some(ratatui::text::Span::raw("\u{1f3c9} ")); // Rugby football
                } else if upper_display.contains("EVENT") || upper_display.contains("PPV") || upper_display.contains("SHREDS") {
                     icon_span = Some(ratatui::text::Span::raw("\u{1f3ab} ")); // Ticket
                } else if upper_display.contains("ESPN") || upper_display.contains("DAZN") || upper_display.contains("B/R") || upper_display.contains("BALLY") || upper_display.contains("MAX SPORTS") {
                     icon_span = Some(ratatui::text::Span::raw("\u{1f4fa} ")); // TV (sports network)
                }
            }

            if let Some(s) = icon_span {
                spans.push(s);
            }

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

            let is_league = parsed.country.as_ref().map(|cc| ["NBA", "NFL", "MLB", "NHL", "UFC", "SPORTS", "PPV"].contains(&cc.as_str())).unwrap_or(false);
            let name_color = if is_league {
                parsed.country.as_ref().map(|cc| country_color(cc)).unwrap_or(Color::White)
            } else {
                MATRIX_GREEN
            };
            
            // Icon & Name Style
            let (icon, icon_color) = if app.current_screen == CurrentScreen::VodCategories || app.current_screen == CurrentScreen::VodStreams {
                ("🎬", MATRIX_GREEN)
            } else if app.current_screen == CurrentScreen::SeriesCategories || app.current_screen == CurrentScreen::SeriesStreams {
                ("📺", MATRIX_GREEN)
            } else {
                ("📁", Color::White)
            };
            spans.insert(0, ratatui::text::Span::styled(format!("{} ", icon), Style::default().fg(icon_color)));

            let (styled_name, _) = stylize_channel_name(
                &parsed.display_name,
                parsed.is_vip,
                false,
                parsed.quality,
                parsed.content_type,
                None,
                Style::default().fg(name_color),
            );

            spans.extend(styled_name);

            ListItem::new(Line::from(spans))
        })
        .collect();

    let title = if app.categories.is_empty() {
        " // LIVE_CATEGORIES / [NULL] ".to_string()
    } else {
        format!(" // LIVE_CATEGORIES ({}) ", app.categories.len())
    };
    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(Style::default().fg(border_color)),
        )
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

    f.render_stateful_widget(list, area, &mut adjusted_state);
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
            let parsed = parse_stream(&s.name, app.provider_timezone.as_deref());
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
            let name_color = if is_league {
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

            // 5. Time Reference (e.g. [Today 09:00 PM])
            if let Some(st) = parsed.start_time {
                let time_str = format!(" {} ", format_relative_time(st, &user_tz));
                let mut time_style = Style::default().fg(Color::Rgb(150, 150, 150));
                if is_ended {
                    time_style = time_style.add_modifier(Modifier::CROSSED_OUT);
                }
                spans.push(ratatui::text::Span::styled(time_str, time_style));
            }

            // 6. Live / Ended Badges
            let light_red = Color::Rgb(255, 120, 120);
            let dim_gray = Color::Rgb(60, 60, 60);
            if is_live {
                let live_color = if is_blink_on { light_red } else { dim_gray };
                let dot = if is_blink_on { "\u{1f534} " } else { "  " };
                spans.push(ratatui::text::Span::styled(dot, Style::default().fg(live_color)));
                spans.push(ratatui::text::Span::styled("LIVE NOW ", Style::default().fg(live_color).add_modifier(Modifier::BOLD)));
            } else if is_ended {
                spans.push(ratatui::text::Span::styled("[ENDED] ", Style::default().fg(Color::Rgb(100, 100, 100)).add_modifier(Modifier::BOLD)));
            }

            // 7. Location (LOC) at the end
            if let Some(loc) = parsed.location {
                spans.push(ratatui::text::Span::styled(format!("({})", loc), Style::default().fg(Color::LightBlue)));
            }

            // 8. Network Health (Latency)
            spans.push(latency_to_bars(s.latency_ms));

            ListItem::new(Line::from(spans))
        })
        .collect();

    let title = format!(" // LIVE_STREAMS ({}) / TZ: {} ", app.streams.len(), tz_display);
    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(Style::default().fg(border_color)),
        )
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

    f.render_stateful_widget(list, area, &mut adjusted_state);
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
            let parsed = parse_stream(&s.name, app.provider_timezone.as_deref());
            let mut spans = vec![];
            
            // Icon & Name Style
            let (icon, icon_color) = match s.stream_type.as_str() {
                "movie" => ("🎬 ", MATRIX_GREEN),
                "series" => ("📺 ", MATRIX_GREEN),
                _ => ("📻 ", MATRIX_GREEN),
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
