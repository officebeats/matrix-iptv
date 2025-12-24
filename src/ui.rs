use crate::app::{App, CurrentScreen, Guide, InputMode, LoginField, Pane};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

// ... existing code ...

const MATRIX_GREEN: Color = Color::Rgb(0, 255, 70);
const DARK_GREEN: Color = Color::White;
const BRIGHT_GREEN: Color = Color::Rgb(180, 255, 180);

// Helper function to calculate the maximum display width of category names
fn calculate_max_category_width(categories: &[crate::api::Category]) -> u16 {
    if categories.is_empty() {
        return 25; // Minimum default width
    }
    
    categories
        .iter()
        .map(|c| {
            // Account for: folder icon (2) + space (1) + name + padding (2)
            (c.category_name.len() as u16) + 5
        })
        .max()
        .unwrap_or(25)
        .max(25) // Minimum width
        .min(45) // Maximum width to prevent overly wide columns
}

// Helper function to calculate optimal column split for 2-column layout
fn calculate_two_column_split(categories: &[crate::api::Category], total_width: u16) -> (u16, u16) {
    let cat_width = calculate_max_category_width(categories);
    let min_stream_width = 60; // Minimum width for streams column
    
    // Ensure we have enough space for both columns
    if cat_width + min_stream_width > total_width {
        // If content is too wide, use proportional split
        (total_width * 30 / 100, total_width * 70 / 100)
    } else {
        // Use exact width for categories, rest for streams
        (cat_width, total_width - cat_width)
    }
}

// Helper function for 3-column Series layout
fn calculate_three_column_split(
    categories: &[crate::api::Category],
    series: &[crate::api::Stream],
    total_width: u16,
) -> (u16, u16, u16) {
    let cat_width = calculate_max_category_width(categories);
    
    let series_width = if series.is_empty() {
        35
    } else {
        series
            .iter()
            .map(|s| {
                // Account for: TV icon (2) + space (1) + name + year badge (~8) + padding (2)
                (s.name.len() as u16) + 13
            })
            .max()
            .unwrap_or(35)
            .max(35)
            .min(55) // Cap series column
    };
    
    let min_episode_width = 50;
    let total_needed = cat_width + series_width + min_episode_width;
    
    if total_needed > total_width {
        // Use proportional split if too wide
        (total_width * 25 / 100, total_width * 35 / 100, total_width * 40 / 100)
    } else {
        // Use exact widths
        let remaining = total_width - cat_width - series_width;
        (cat_width, series_width, remaining)
    }
}

pub fn ui(f: &mut Frame, app: &mut App) {
    // FTUE: Matrix rain animation - show ONLY this, nothing else
    if app.show_matrix_rain {
        crate::matrix_rain::render_matrix_rain(f, app, f.area());
        return;
    }
    
    // FTUE: Welcome popup - show ONLY this, nothing else
    if app.show_welcome_popup {
        crate::matrix_rain::render_welcome_popup(f, app, f.area());
        return;
    }
    
    // Create the layout sections.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(3),    // Main Content
            Constraint::Length(3), // Footer/Legend
        ])
        .split(f.area());

    let header_area = chunks[0];
    let main_area = chunks[1];
    let footer_area = chunks[2];

    // Render header
    render_header(f, app, header_area);

    // Draw the active screen
    match app.current_screen {
        CurrentScreen::Home => render_home(f, app, main_area),
        CurrentScreen::Login => render_login(f, app, main_area),
        CurrentScreen::Categories | CurrentScreen::Streams => {
            render_split_view(f, app, main_area, false)
        }
        CurrentScreen::VodCategories | CurrentScreen::VodStreams => {
            render_split_view(f, app, main_area, true)
        }
        CurrentScreen::SeriesCategories | CurrentScreen::SeriesStreams => {
            render_series_view(f, app, main_area)
        }
        CurrentScreen::Settings => render_settings(f, app, main_area),
        CurrentScreen::TimezoneSettings => render_timezone_settings(f, app, main_area),
        CurrentScreen::ContentTypeSelection => render_content_type_selection(f, app, main_area),
        _ => {}
    }

    // Render footer
    render_footer(f, app, footer_area);

    // Help modal overlay
    if app.show_help {
        render_help_popup(f, f.area());
    }

    if app.show_guide.is_some() {
        render_guide_popup(f, app, f.area());
    }

    let size = f.area();

    // Render Loading
    if app.state_loading {
        render_loading(f, app, size);
    }

    // Render Save Confirmation
    if app.show_save_confirmation {
        let block = Block::default()
            .title(" Unsaved Changes ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Yellow));

        let area = centered_rect(40, 20, size);
        f.render_widget(Clear, area);
        f.render_widget(block, area);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(2), Constraint::Length(3)])
            .margin(1)
            .split(area);

        let text = Paragraph::new("You have unsaved changes.\nDo you want to save them?")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White));
        f.render_widget(text, layout[0]);

        let buttons = Paragraph::new(Line::from(vec![
            Span::styled(
                "[Y] Save ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                "[N] Discard ",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled("[Esc] Cancel", Style::default().fg(Color::LightBlue)),
        ]))
        .alignment(Alignment::Center);
        f.render_widget(buttons, layout[1]);
    }
}



fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(38), // Tabs
            Constraint::Min(0),     // Stats (takes remainder)
        ])
        .split(area);

    if app.search_mode {
        let search_text = format!(" // SEARCH_PROTOCOLS: {}_", app.search_query);
        let p = Paragraph::new(search_text)
            .style(
                Style::default()
                    .fg(MATRIX_GREEN)
                    .add_modifier(Modifier::BOLD),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .title(" SYSTEM_SEARCH_OVERRIDE ")
                    .border_style(Style::default().fg(MATRIX_GREEN)),
            );
        f.render_widget(p, area);
        return;
    }

    let current_tab = match app.current_screen {
        CurrentScreen::VodCategories | CurrentScreen::VodStreams => 1,
        CurrentScreen::Settings | CurrentScreen::TimezoneSettings => 2,
        CurrentScreen::SeriesCategories | CurrentScreen::SeriesStreams => 3,
        _ => 0,
    };

    let style_active = Style::default()
        .bg(MATRIX_GREEN)
        .fg(Color::Black)
        .add_modifier(Modifier::BOLD);
    let _style_inactive = Style::default().fg(DARK_GREEN);
    let separator = Span::styled(" / ", Style::default().fg(Color::LightBlue));

    let mut spans = vec![Span::styled(
        " // SYSTEM_NETWORK",
        Style::default()
            .fg(MATRIX_GREEN)
            .add_modifier(Modifier::BOLD),
    )];

    // Live TV Tab
    spans.push(separator.clone());
    if current_tab == 0 {
        spans.push(Span::styled(" [LIVE_UPLINK] ", style_active));
    } else {
        spans.push(Span::styled(
            " LIVE_UPLINK ",
            Style::default().fg(MATRIX_GREEN),
        ));
    }

    // VOD Tab
    spans.push(separator.clone());
    if current_tab == 1 {
        spans.push(Span::styled(" [MOVIE_ACCESS] ", style_active));
    } else {
        spans.push(Span::styled(
            " MOVIE_ACCESS ",
            Style::default().fg(MATRIX_GREEN),
        ));
    }

    // Series Tab (right after VOD)
    spans.push(separator.clone());
    if current_tab == 3 {
        spans.push(Span::styled(" [SERIAL_LOGS] ", style_active));
    } else {
        spans.push(Span::styled(
            " SERIAL_LOGS ",
            Style::default().fg(MATRIX_GREEN),
        ));
    }

    // Settings Tab
    spans.push(separator.clone());
    if current_tab == 2 {
        spans.push(Span::styled(" [CORE_CONFIG] ", style_active));
    } else {
        spans.push(Span::styled(
            " CORE_CONFIG ",
            Style::default().fg(MATRIX_GREEN),
        ));
    }

    // Breadcrumb extension with Node and Stats
    spans.push(separator.clone());
    let node_name = if let Some(_) = &app.current_client {
        app.config
            .accounts
            .get(app.selected_account_index)
            .map(|a| a.name.to_uppercase())
            .unwrap_or("GUEST".to_string())
    } else {
        "UNCONNECTED".to_string()
    };
    spans.push(Span::styled(
        format!("NODE: {}", node_name),
        Style::default()
            .fg(BRIGHT_GREEN)
            .add_modifier(Modifier::BOLD),
    ));

    if let Some(info) = &app.account_info {
        spans.push(separator.clone());
        let active = info
            .active_cons
            .as_ref()
            .map(|v| v.to_string().replace("\"", ""))
            .unwrap_or("0".to_string());
        let total = info
            .max_connections
            .as_ref()
            .map(|v| v.to_string().replace("\"", ""))
            .unwrap_or("1".to_string());
        spans.push(Span::styled(
            format!("CONS: {}/{}", active, total),
            Style::default().fg(MATRIX_GREEN),
        ));
    }

    let tabs = Paragraph::new(Line::from(spans)).block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(DARK_GREEN)),
    );
    f.render_widget(tabs, chunks[0]);
}

fn render_split_view(f: &mut Frame, app: &mut App, area: Rect, is_vod: bool) {
    // Calculate dynamic column widths based on category content
    let categories = if is_vod {
        &app.vod_categories
    } else {
        &app.categories
    };
    
    let (cat_width, stream_width) = calculate_two_column_split(categories, area.width);
    
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(cat_width),  // Categories - dynamic width
            Constraint::Min(stream_width),  // Streams - remaining space
        ])
        .split(area);

    let cat_area = chunks[0];
    let stream_area = chunks[1];

    // Store for mouse tracking
    app.area_categories = cat_area;
    app.area_streams = stream_area;

    // Determine active pane styling
    let cat_border_color = if app.active_pane == Pane::Categories {
        MATRIX_GREEN
    } else {
        DARK_GREEN
    };
    let stream_border_color = if app.active_pane == Pane::Streams {
        MATRIX_GREEN
    } else {
        DARK_GREEN
    };

    // Render categories pane
    if is_vod {
        render_vod_categories_pane(f, app, cat_area, cat_border_color);
        render_vod_streams_pane(f, app, stream_area, stream_border_color);
    } else {
        render_categories_pane(f, app, cat_area, cat_border_color);
        render_streams_pane(f, app, stream_area, stream_border_color);
    }
}

fn render_categories_pane(f: &mut Frame, app: &mut App, area: Rect, border_color: Color) {
    use crate::parser::{country_color, country_flag, parse_category};

    // Calculate visible window to avoid parsing all items
    let visible_height = area.height.saturating_sub(2) as usize; // Account for borders
    let total = app.categories.len();
    let selected = app.selected_category_index;

    // Determine the window of items to render
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

            // Build styled line with flag, name, and badges
            let mut spans = vec![];

            // Favorite Indicator
            if app.config.favorites.categories.contains(&c.category_id) {
                spans.push(Span::styled("★ ", Style::default().fg(Color::Yellow)));
            }

            // Country flag
            if let Some(ref country) = parsed.country {
                let flag = country_flag(country);
                if !flag.is_empty() {
                    spans.push(Span::raw(format!("{} ", flag)));
                }
            }

            // Category name with country color
            let name_color = parsed
                .country
                .as_ref()
                .map(|c| country_color(c))
                .unwrap_or(Color::White);

            spans.push(Span::styled(
                parsed.display_name.clone(),
                Style::default().fg(name_color),
            ));

            // Quality badge
            if let Some(quality) = parsed.quality {
                spans.push(Span::raw(" "));
                spans.push(Span::styled(
                    quality.badge(),
                    Style::default()
                        .fg(quality.color())
                        .add_modifier(Modifier::BOLD),
                ));
            }

            // Content type icon
            if let Some(content) = parsed.content_type {
                let icon = content.icon();
                if !icon.is_empty() {
                    spans.push(Span::raw(format!(" {}", icon)));
                }
            }

            // VIP indicator
            if parsed.is_vip {
                spans.push(Span::styled(" ⭐", Style::default().fg(Color::Yellow)));
            }

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

    // For windowed rendering, we need to adjust the list state offset
    let mut adjusted_state = app.category_list_state.clone();
    if adjusted_start > 0 {
        // Adjust selection to be relative to window
        adjusted_state.select(Some(selected - adjusted_start));
    }

    f.render_stateful_widget(list, area, &mut adjusted_state);
}

fn render_streams_pane(f: &mut Frame, app: &mut App, area: Rect, border_color: Color) {
    use crate::parser::{country_color, country_flag, parse_stream};
    use chrono::Utc;
    use chrono_tz::Tz;
    use std::str::FromStr;

    // Get user timezone
    let tz_str = app.config.get_user_timezone();
    let user_tz: Tz = Tz::from_str(&tz_str).unwrap_or(chrono_tz::Europe::London);
    let now = Utc::now().with_timezone(&user_tz);

    // Calculate visible window to avoid parsing all items
    let visible_height = area.height.saturating_sub(2) as usize; // Account for borders
    let total = app.streams.len();
    let selected = app.selected_stream_index;

    // Determine the window of items to render
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
        .streams
        .iter()
        .enumerate()
        .skip(adjusted_start)
        .take(end - adjusted_start)
        .map(|(_, s)| {
            let effective_name = s.stream_display_name.as_ref().unwrap_or(&s.name);
            // Use current account's server timezone if available
            let provider_tz = app
                .config
                .accounts
                .get(app.selected_account_index)
                .and_then(|a| a.server_timezone.as_deref());
            let mut parsed = parse_stream(effective_name, provider_tz);

            // Revert to original name or scrub if event is stale (> 3 hours old)
            if let Some(start_time_utc) = parsed.start_time {
                // Calculate difference (now is ahead of start if it happened in past)
                let diff_secs = now.timestamp() - start_time_utc.timestamp();

                // If event was more than 3 hours ago (10800 seconds)
                if diff_secs > 10800 {
                    if s.stream_display_name.is_some() {
                        // Fallback to internal name
                        parsed = parse_stream(&s.name, provider_tz);
                    }

                    // Scrubbing: Also remove common LIVE indicators from display_name if it's stale
                    let scrubbed = parsed
                        .display_name
                        .replace("🔴 LIVE", "")
                        .replace("🔴", "")
                        .replace("LIVE NOW", "")
                        .replace("LIVE", "")
                        .trim()
                        .to_string();
                    parsed.display_name = scrubbed;

                    // If it's stale we don't clear sports_event yet because we want team names,
                    // but we ensure is_ended will be true.
                }
            }

            // Handle separator lines differently
            if parsed.is_separator {
                return ListItem::new(Line::from(vec![Span::styled(
                    format!("═══ {} ═══", parsed.display_name),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )]));
            }

            // Calculate timing early for uniform styling
            let mut diff_mins = 0;
            let mut is_ended = false;
            let mut local_time_opt = None;

            if let Some(start_time_utc) = parsed.start_time {
                let local_time = start_time_utc.with_timezone(&user_tz);
                diff_mins = (local_time.timestamp() - now.timestamp()) / 60;
                
                // Use stop_time if available (from Strong8K format) with 30-min buffer for overtime
                // Otherwise fallback to 4 hours past start time (covers most sports events + delays)
                if let Some(stop_time_utc) = parsed.stop_time {
                    // Add 30-minute buffer for overtime, halftime extended, etc.
                    let buffer_mins = 30;
                    let stop_with_buffer = stop_time_utc.timestamp() + (buffer_mins * 60);
                    is_ended = now.timestamp() > stop_with_buffer;
                } else {
                    // Fallback: 4 hours past start (increased from 3 to cover longer games)
                    is_ended = diff_mins < -240;
                }
                local_time_opt = Some(local_time);
            }

            let base_style = if is_ended {
                Style::default()
                    .fg(Color::Gray)
                    .add_modifier(Modifier::ITALIC)
                    .add_modifier(Modifier::CROSSED_OUT)
            } else {
                Style::default()
            };

            let mut spans = vec![];

            // Favorite Indicator
            let id = match &s.stream_id {
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::String(s) => s.clone(),
                _ => s.stream_id.to_string(),
            };
            if app.config.favorites.streams.contains(&id) {
                spans.push(Span::styled(
                    "★ ",
                    Style::default().fg(if is_ended {
                        Color::Gray
                    } else {
                        Color::Yellow
                    }),
                ));
            }

            // Country flag
            if let Some(ref country) = parsed.country {
                let flag = country_flag(country);
                if !flag.is_empty() {
                    spans.push(Span::raw(format!("{} ", flag)));
                }
            } else {
                spans.push(Span::raw("📺 "));
            }

            // Channel name / Sports Event
            if let Some(ref event) = parsed.sports_event {
                use crate::sports::get_team_color;

                let t1_color = if is_ended {
                    Color::Gray
                } else {
                    get_team_color(&event.team1)
                };
                let t2_color = if is_ended {
                    Color::Gray
                } else {
                    get_team_color(&event.team2)
                };

                // Show original channel name (e.g. "NBA 08")
                let short_name = parsed
                    .display_name
                    .split(':')
                    .next()
                    .unwrap_or(&parsed.display_name);
                spans.push(Span::styled(
                    format!("{}: ", short_name.trim()),
                    base_style.fg(Color::Gray),
                ));

                spans.push(Span::styled(
                    format!("{} ", event.team1),
                    base_style.fg(t1_color),
                ));
                if let Some(ref abbr) = event.team1_abbr {
                    let abbr_color = if is_ended {
                        Color::Gray
                    } else {
                        get_team_color(abbr)
                    };
                    spans.push(Span::styled(
                        format!("({}) ", abbr),
                        base_style.fg(abbr_color),
                    ));
                }

                spans.push(Span::styled("x ", base_style.fg(Color::Gray)));

                spans.push(Span::styled(
                    format!("{} ", event.team2),
                    base_style.fg(t2_color),
                ));
                if let Some(ref abbr) = event.team2_abbr {
                    let abbr_color = if is_ended {
                        Color::Gray
                    } else {
                        get_team_color(abbr)
                    };
                    spans.push(Span::styled(
                        format!("({}) ", abbr),
                        base_style.fg(abbr_color),
                    ));
                }
            } else {
                let name_color = if is_ended {
                    Color::Gray
                } else {
                    parsed
                        .country
                        .as_ref()
                        .map(|c| country_color(c))
                        .unwrap_or(Color::White)
                };
                spans.push(Span::styled(
                    parsed.display_name.clone(),
                    base_style.fg(name_color),
                ));
            }

            // EVENT TIME
            if let Some(local_time) = local_time_opt {
                let time_str = local_time.format("%I:%M %p").to_string();

                let (time_color, status_text) = if is_ended {
                    (Color::Gray, " [ENDED]")
                } else if diff_mins <= 0 {
                    (Color::Red, " LIVE NOW")
                } else if diff_mins < 60 {
                    (Color::Yellow, " SOON")
                } else {
                    (Color::White, "")
                };

                let date_label = if local_time.date_naive() == now.date_naive() {
                    "Today".to_string()
                } else if local_time.date_naive()
                    == now.date_naive().succ_opt().unwrap_or(now.date_naive())
                {
                    "Tomorrow".to_string()
                } else if local_time.date_naive()
                    == now.date_naive().pred_opt().unwrap_or(now.date_naive())
                {
                    "Yesterday".to_string()
                } else {
                    local_time.format("%d/%m").to_string()
                };

                let display_str = format!(" [{} {}]", date_label, time_str);

                spans.push(Span::styled(
                    display_str,
                    base_style.fg(time_color).add_modifier(Modifier::BOLD),
                ));
                if !status_text.is_empty() {
                    if status_text == " LIVE NOW" {
                        // Blinking dot effect using loading_tick
                        if app.loading_tick % 10 < 5 {
                            spans.push(Span::styled(" 🔴", Style::default().fg(Color::Red)));
                        } else {
                            spans.push(Span::raw("   "));
                        }
                    }
                    spans.push(Span::styled(
                        status_text,
                        base_style.fg(time_color).add_modifier(Modifier::BOLD),
                    ));
                }
            }

            // Quality badge
            if let Some(quality) = parsed.quality {
                spans.push(Span::raw(" "));
                let q_color = if is_ended {
                    Color::DarkGray
                } else {
                    quality.color()
                };
                spans.push(Span::styled(
                    quality.badge(),
                    base_style.fg(q_color).add_modifier(Modifier::BOLD),
                ));
            }

            // Live event indicator (Explicit)
            if parsed.is_live_event && parsed.start_time.is_none() {
                spans.push(Span::styled(
                    " 🔴 LIVE",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ));
            }

            // Location in dim
            if let Some(ref loc) = parsed.location {
                spans.push(Span::styled(
                    format!(" ({})", loc),
                    Style::default().fg(Color::LightBlue),
                ));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    let title = if app.streams.is_empty() {
        format!(" // LIVE_STREAMS / [NULL] / TZ: {} ", tz_str)
    } else {
        format!(" // LIVE_STREAMS ({}) / TZ: {} ", app.streams.len(), tz_str)
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

    // For windowed rendering, we need to adjust the list state offset
    let mut adjusted_state = app.stream_list_state.clone();
    if adjusted_start > 0 {
        // Adjust selection to be relative to window
        adjusted_state.select(Some(selected - adjusted_start));
    }

    f.render_stateful_widget(list, area, &mut adjusted_state);

    if let Some(err) = &app.player_error {
        let error_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(area)[1];

        let p = Paragraph::new(format!(" ❌ Player Error: {}", err)).style(
            Style::default()
                .fg(Color::White)
                .bg(Color::Red)
                .add_modifier(Modifier::BOLD),
        );
        f.render_widget(p, error_area);
    }
}

fn render_vod_categories_pane(f: &mut Frame, app: &mut App, area: Rect, border_color: Color) {
    use crate::parser::parse_vod_category;

    // Calculate visible window to avoid parsing all items
    let visible_height = area.height.saturating_sub(2) as usize; // Account for borders
    let total = app.vod_categories.len();
    let selected = app.selected_vod_category_index;

    // Determine the window of items to render
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
        .vod_categories
        .iter()
        .enumerate()
        .skip(adjusted_start)
        .take(end - adjusted_start)
        .map(|(_idx, c)| {
            let parsed = parse_vod_category(&c.category_name);
            let mut spans = vec![];

            // Favorite Indicator
            if app.config.favorites.vod_categories.contains(&c.category_id) {
                spans.push(Span::styled("★ ", Style::default().fg(Color::Yellow)));
            }

            // Streaming source icon
            if let Some(source) = parsed.streaming_source {
                let icon = source.icon();
                if !icon.is_empty() {
                    spans.push(Span::styled(
                        format!("{} ", icon),
                        Style::default().fg(source.color()),
                    ));
                }
            }

            // Kids indicator
            if parsed.is_kids {
                spans.push(Span::raw("🧸 "));
            }

            // Category name
            let name_color = parsed
                .streaming_source
                .map(|s| s.color())
                .unwrap_or(Color::White);
            spans.push(Span::styled(
                parsed.display_name.clone(),
                Style::default().fg(name_color),
            ));

            // Quality badge
            if let Some(quality) = parsed.quality {
                spans.push(Span::raw(" "));
                spans.push(Span::styled(
                    quality.badge(),
                    Style::default()
                        .fg(quality.color())
                        .add_modifier(Modifier::BOLD),
                ));
            }

            // Language tag
            if let Some(ref lang) = parsed.language {
                if lang != "VOD" {
                    spans.push(Span::styled(
                        format!(" [{}]", lang),
                        Style::default().fg(Color::LightBlue),
                    ));
                }
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    let title = if app.vod_categories.is_empty() {
        " // VOD_CATEGORIES / [NULL] ".to_string()
    } else {
        format!(" // VOD_CATEGORIES ({}) ", app.vod_categories.len())
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

    // For windowed rendering, we need to adjust the list state offset
    let mut adjusted_state = app.vod_category_list_state.clone();
    if adjusted_start > 0 {
        // Adjust selection to be relative to window
        adjusted_state.select(Some(selected - adjusted_start));
    }

    f.render_stateful_widget(list, area, &mut adjusted_state);
}

fn render_vod_streams_pane(f: &mut Frame, app: &mut App, area: Rect, border_color: Color) {
    use crate::parser::parse_movie;

    // Calculate visible window to avoid parsing all items
    let visible_height = area.height.saturating_sub(2) as usize; // Account for borders
    let total = app.vod_streams.len();
    let selected = app.selected_vod_stream_index;

    // Determine the window of items to render
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

    // Build items only for the visible window
    let items: Vec<ListItem> = app
        .vod_streams
        .iter()
        .enumerate()
        .skip(adjusted_start)
        .take(end - adjusted_start)
        .map(|(_, s)| {
            let mut parsed = parse_movie(&s.name);

            // Extract rating from API data if available
            let api_rating = s.rating.as_ref().and_then(|v| match v {
                serde_json::Value::String(rs) => {
                    if !rs.is_empty() && rs != "0" && rs != "0.0" {
                        Some(rs.clone())
                    } else {
                        None
                    }
                }
                serde_json::Value::Number(rn) => {
                    if rn.as_f64().unwrap_or(0.0) > 0.0 {
                        Some(rn.to_string())
                    } else {
                        None
                    }
                }
                _ => None,
            });
            parsed.rating = api_rating;

            let mut spans = vec![];

            // Favorite Indicator
            let id = match &s.stream_id {
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::String(s) => s.clone(),
                _ => s.stream_id.to_string(),
            };
            if app.config.favorites.vod_streams.contains(&id) {
                spans.push(Span::styled("★ ", Style::default().fg(Color::Yellow)));
            }

            // Movie icon
            spans.push(Span::raw("🎬 "));

            // Language prefix
            if let Some(ref lang) = parsed.language {
                if lang == "TOP" {
                    spans.push(Span::styled("★ ", Style::default().fg(Color::Yellow)));
                } else {
                    spans.push(Span::styled(
                        format!("[{}] ", lang),
                        Style::default().fg(Color::Cyan),
                    ));
                }
            }

            // Rating
            if let Some(ref rating) = parsed.rating {
                spans.push(Span::styled(
                    format!("[{}] ", rating),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ));
            }

            // Title
            spans.push(Span::styled(
                parsed.title.clone(),
                Style::default().fg(Color::White),
            ));

            // Year
            if let Some(year) = parsed.year {
                spans.push(Span::styled(
                    format!(" ({})", year),
                    Style::default().fg(Color::LightBlue),
                ));
            }

            // Quality badge
            if let Some(quality) = parsed.quality {
                spans.push(Span::raw(" "));
                spans.push(Span::styled(
                    quality.badge(),
                    Style::default()
                        .fg(quality.color())
                        .add_modifier(Modifier::BOLD),
                ));
            }

            // Multi-sub indicator
            if parsed.has_multi_sub {
                spans.push(Span::styled(" 🌐", Style::default().fg(Color::Green)));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    let title = if app.vod_streams.is_empty() {
        " // MOVIE_DATA / [NULL] ".to_string()
    } else {
        format!(" // MOVIE_DATA ({}) ", app.vod_streams.len())
    };

    // For windowed rendering, we need to adjust the list state offset
    let mut adjusted_state = app.vod_stream_list_state.clone();
    if adjusted_start > 0 {
        // Adjust selection to be relative to window
        adjusted_state.select(Some(selected - adjusted_start));
    }

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

    f.render_stateful_widget(list, area, &mut adjusted_state);
}

fn render_home(f: &mut Frame, app: &mut App, area: Rect) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8), // Logo/Header
            Constraint::Min(0),    // Content
            Constraint::Length(1), // Footer info
        ])
        .split(area);

    // ASCII Art Logo
    let logo_text = vec![
        "███╗   ███╗ █████╗ ████████╗██████╗ ██╗██╗  ██╗    ██╗██████╗ ████████╗██╗   ██╗     ██████╗██╗     ██╗",
        "████╗ ████║██╔══██╗╚══██╔══╝██╔══██╗██║╚██╗██╔╝    ██║██╔══██╗╚══██╔══╝██║   ██║    ██╔════╝██║     ██║",
        "██╔████╔██║███████║   ██║   ██████╔╝██║ ╚███╔╝     ██║██████╔╝   ██║   ██║   ██║    ██║     ██║     ██║",
        "██║╚██╔╝██║██╔══██║   ██║   ██╔══██╗██║ ██╔██╗     ██║██╔═══╝    ██║   ╚██╗ ██╔╝    ██║     ██║     ██║",
        "██║ ╚═╝ ██║██║  ██║   ██║   ██║  ██║██║██╔╝ ██╗    ██║██║        ██║    ╚████╔╝     ╚██████╗███████╗██║",
        "╚═╝     ╚═╝╚═╝  ╚═╝   ╚═╝   ╚═╝  ╚═╝╚═╝╚═╝  ╚═╝    ╚═╝╚═╝        ╚═╝     ╚═══╝       ╚═════╝╚══════╝╚═╝",
    ];

    let logo = Paragraph::new(logo_text.join("\n"))
        .style(
            Style::default()
                .fg(MATRIX_GREEN)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    f.render_widget(logo, main_layout[0]);

    // Content Split: Sidebar vs Main
    let content_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(40), // Sidebar (Nodes)
            Constraint::Min(0),     // Details
        ])
        .split(main_layout[1]);

    // --- SIDEBAR (Nodes) ---
    let accounts: Vec<ListItem> = app
        .config
        .accounts
        .iter()
        .map(|acc| {
            ListItem::new(Line::from(vec![
                Span::styled(" [NODE] ", Style::default().fg(Color::LightBlue)),
                Span::styled(
                    acc.name.to_uppercase(),
                    Style::default()
                        .fg(MATRIX_GREEN)
                        .add_modifier(Modifier::BOLD),
                ),
            ]))
        })
        .collect();

    let account_list = List::new(accounts)
        .block(
            Block::default()
                .title(" // CATEGORY_NODES ")
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(Style::default().fg(DARK_GREEN)),
        )
        .highlight_style(
            Style::default()
                .bg(MATRIX_GREEN)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" » ");

    app.area_accounts = content_layout[0];
    f.render_stateful_widget(account_list, content_layout[0], &mut app.account_list_state);

    // --- MAIN ZONE (Guides/Disclaimer) ---
    let main_zone_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(12), // Guides
            Constraint::Min(0),     // Footer/System Info
        ])
        .split(content_layout[1]);

    let mut guides_text = Vec::new();

    if app.config.accounts.is_empty() {
        guides_text.extend(vec![
            Line::from(vec![Span::styled(
                " // SYSTEM_GUIDES:",
                Style::default()
                    .fg(BRIGHT_GREEN)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    " [1] ",
                    Style::default()
                        .fg(MATRIX_GREEN)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Why CLI for IPTV?", Style::default().fg(MATRIX_GREEN)),
            ]),
            Line::from(vec![
                Span::styled(
                    " [2] ",
                    Style::default()
                        .fg(MATRIX_GREEN)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "Where do I get playlists for the content?",
                    Style::default().fg(MATRIX_GREEN),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    " [3] ",
                    Style::default()
                        .fg(MATRIX_GREEN)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("What is IPTV?", Style::default().fg(MATRIX_GREEN)),
            ]),
        ]);
    } else {
        guides_text.extend(vec![
            Line::from(vec![Span::styled(
                " // SYSTEM_READY:",
                Style::default()
                    .fg(BRIGHT_GREEN)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                " > Press [Enter] to Load Playlist",
                Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::SLOW_BLINK),
            )]),
        ]);
    }

    guides_text.extend(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " ⚠ ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " DISCLAIMER: ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Matrix IPTV CLI is a client and does not provide any content.",
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(vec![Span::styled(
            "     Content must be sourced from an online playlist.",
            Style::default().fg(Color::Yellow),
        )]),
    ]);
    let guides_widget = Paragraph::new(guides_text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(Style::default().fg(DARK_GREEN))
            .padding(ratatui::widgets::Padding::new(2, 2, 1, 1)),
    );
    f.render_widget(guides_widget, main_zone_chunks[0]);

    // Footer Info Line
    let footer_info = Paragraph::new(vec![Line::from(vec![
        Span::styled(
            " [Esc] ",
            Style::default()
                .fg(MATRIX_GREEN)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Back  ", Style::default().fg(Color::LightBlue)),
        Span::styled(
            " [Enter] ",
            Style::default()
                .fg(MATRIX_GREEN)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Play  ", Style::default().fg(Color::LightBlue)),
        Span::styled(
            " [N] ",
            Style::default()
                .fg(MATRIX_GREEN)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("New Node  ", Style::default().fg(Color::LightBlue)),
        Span::styled(
            " [X] ",
            Style::default()
                .fg(MATRIX_GREEN)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Config", Style::default().fg(Color::LightBlue)),
    ])])
    .alignment(Alignment::Center);
    f.render_widget(footer_info, main_layout[2]);

    // Login Error overlay
    if let Some(err) = &app.login_error {
        let error_msg = Paragraph::new(format!(" // ERROR_TRAP: {}", err))
            .style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            )
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Red)),
            );

        let area = centered_rect(60, 15, area);
        f.render_widget(Clear, area);
        f.render_widget(error_msg, area);
    }
}

fn render_login(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title("ADD NEW PLAYLIST")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Gray));
    f.render_widget(block.clone(), area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // Name
            Constraint::Length(3), // URL
            Constraint::Length(3), // User
            Constraint::Length(3), // Pass
            Constraint::Length(3), // EPG
            Constraint::Min(3),    // Error
        ])
        .split(area);

    fn render_input<'a>(
        label: &'a str,
        value: &'a str,
        is_active: bool,
        is_editing: bool,
    ) -> Paragraph<'a> {
        let title_style = if is_active {
            Style::default()
                .fg(MATRIX_GREEN)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(DARK_GREEN)
        };

        let border_style = if is_editing {
            Style::default().fg(BRIGHT_GREEN)
        } else if is_active {
            Style::default().fg(MATRIX_GREEN)
        } else {
            Style::default().fg(DARK_GREEN)
        };

        let content_style = if is_editing {
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(MATRIX_GREEN)
        };

        Paragraph::new(value)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Thick)
                    .title(Span::styled(
                        format!(" {} ", label.to_uppercase()),
                        title_style,
                    ))
                    .border_style(border_style),
            )
            .style(content_style)
    }

    let active = &app.login_field_focus;
    let mode = app.input_mode == InputMode::Editing;

    f.render_widget(
        render_input(
            "Playlist Name",
            app.input_name.value(),
            matches!(active, LoginField::Name),
            mode,
        ),
        chunks[0],
    );
    f.render_widget(
        render_input(
            "Server URL",
            app.input_url.value(),
            matches!(active, LoginField::Url),
            mode,
        ),
        chunks[1],
    );
    f.render_widget(
        render_input(
            "Username",
            app.input_username.value(),
            matches!(active, LoginField::Username),
            mode,
        ),
        chunks[2],
    );
    let mask: String = app.input_password.value().chars().map(|_| '*').collect();
    f.render_widget(
        render_input(
            "Password",
            &mask,
            matches!(active, LoginField::Password),
            mode,
        ),
        chunks[3],
    );
    f.render_widget(
        render_input(
            "EPG URL (Optional)",
            app.input_epg_url.value(),
            matches!(active, LoginField::EpgUrl),
            mode,
        ),
        chunks[4],
    );

    if let Some(err) = &app.login_error {
        let error_msg = Paragraph::new(format!(" // ERROR_OVERRIDE: {}", err)).style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Red)
                .add_modifier(Modifier::BOLD),
        );
        f.render_widget(error_msg, chunks[5]);
    }
}

fn render_settings(f: &mut Frame, app: &mut App, area: Rect) {
    match app.settings_state {
        crate::app::SettingsState::Main => {
            let items: Vec<ListItem> = app
                .settings_options
                .iter()
                .map(|s| ListItem::new(format!("  {}", s)))
                .collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .title(" // AUTHORIZED_NODES ")
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
                .highlight_symbol(" > ");

            f.render_stateful_widget(list, area, &mut app.settings_list_state);
        }
        crate::app::SettingsState::ManageAccounts => {
            let accounts: Vec<ListItem> = app
                .config
                .accounts
                .iter()
                .map(|acc| {
                    ListItem::new(Line::from(vec![
                        Span::styled(
                            format!("  {} ", acc.name),
                            Style::default()
                                .fg(MATRIX_GREEN)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            format!("({})", acc.base_url),
                            Style::default().fg(Color::LightBlue),
                        ),
                    ]))
                })
                .collect();

            let title = if accounts.is_empty() {
                " MANAGE PLAYLISTS (No playlists) ".to_string()
            } else {
                format!(" MANAGE PLAYLISTS ({}) ", accounts.len())
            };

            let list = List::new(accounts)
                .block(
                    Block::default()
                        .title(title)
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        // Custom hints in the border title or distinct area
                        .border_style(Style::default().fg(Color::Cyan)),
                )
                .highlight_style(
                    Style::default()
                        .bg(Color::Cyan)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(" > ");

            f.render_stateful_widget(list, area, &mut app.account_list_state);
        }
        crate::app::SettingsState::About => {
            let about_lines: Vec<Line> = app
                .about_text
                .lines()
                .map(|line| {
                    if line.starts_with("# ") {
                        Line::from(Span::styled(
                            line.trim_start_matches("# ").trim(),
                            Style::default()
                                .fg(MATRIX_GREEN)
                                .add_modifier(Modifier::BOLD),
                        ))
                    } else if line.contains("Built by") || line.contains("www.") {
                        Line::from(Span::styled(line, Style::default().fg(BRIGHT_GREEN)))
                    } else {
                        Line::from(Span::styled(line, Style::default().fg(MATRIX_GREEN)))
                    }
                })
                .collect();

            let p = Paragraph::new(about_lines)
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .title(" // SYSTEM_MANIFEST ")
                        .borders(Borders::ALL)
                        .border_type(BorderType::Thick)
                        .border_style(Style::default().fg(DARK_GREEN)),
                )
                .scroll((app.about_scroll, 0));

            f.render_widget(Clear, area);
            f.render_widget(p, area);
        }
    }
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let key_style = Style::default()
        .fg(BRIGHT_GREEN)
        .add_modifier(Modifier::BOLD);
    let action_style = Style::default().fg(MATRIX_GREEN);
    let separator = Span::styled(" | ", Style::default().fg(DARK_GREEN));

    let mut spans = vec![Span::styled(
        " root@matrix-terminal:~$ ",
        Style::default().fg(DARK_GREEN),
    )];

    match app.current_screen {
        CurrentScreen::Home => {
            spans.push(Span::styled("[q]", key_style));
            spans.push(Span::styled(":ABORT", action_style));
            spans.push(separator.clone());
            spans.push(Span::styled("[n]", key_style));
            spans.push(Span::styled(":NEW_PLAYLIST_UPLINK", action_style));
            spans.push(separator.clone());
            spans.push(Span::styled("[x]", key_style));
            spans.push(Span::styled(":CONFIG", action_style));
        }
        CurrentScreen::Login => match app.input_mode {
            InputMode::Normal => {
                spans.push(Span::styled("[Esc]", key_style));
                spans.push(Span::styled(":BACK", action_style));
                spans.push(separator.clone());
                spans.push(Span::styled("[Enter]", key_style));
                spans.push(Span::styled(":EDIT_FIELD", action_style));
                spans.push(separator.clone());
                spans.push(Span::styled("[↑/↓]", key_style));
                spans.push(Span::styled(":NAVIGATE", action_style));
            }
            InputMode::Editing => {
                spans.push(Span::styled("[Enter]", key_style));
                spans.push(Span::styled(":COMMIT", action_style));
                spans.push(separator.clone());
                spans.push(Span::styled("[Esc]", key_style));
                spans.push(Span::styled(":CANCEL", action_style));
            }
        },
        CurrentScreen::Categories => {
            spans.push(Span::styled("[Esc]", key_style));
            spans.push(Span::styled(":Home", action_style));
            spans.push(separator.clone());
            spans.push(Span::styled("[Tab/s]", key_style));
            spans.push(Span::styled(":Streams", action_style));
            spans.push(separator.clone());
            spans.push(Span::styled("[Enter]", key_style));
            spans.push(Span::styled(":Select", action_style));
            spans.push(separator.clone());
            spans.push(Span::styled("[x]", key_style));
            spans.push(Span::styled(":Settings", action_style));
        }
        CurrentScreen::Streams => {
            spans.push(Span::styled("[Esc]", key_style));
            spans.push(Span::styled(":Back", action_style));
            spans.push(separator.clone());
            spans.push(Span::styled("[Enter]", key_style));
            spans.push(Span::styled(":Select", action_style));
            spans.push(separator.clone());
            spans.push(Span::styled("[f]", key_style));
            spans.push(Span::styled(":Fav", action_style));
        }
        CurrentScreen::VodCategories => {
            spans.push(Span::styled("[Esc]", key_style));
            spans.push(Span::styled(":Home", action_style));
            spans.push(separator.clone());
            spans.push(Span::styled("[Enter]", key_style));
            spans.push(Span::styled(":Select", action_style));
            spans.push(separator.clone());
            spans.push(Span::styled("[f]", key_style));
            spans.push(Span::styled(":Fav", action_style));
        }
        CurrentScreen::VodStreams => {
            spans.push(Span::styled("[Esc]", key_style));
            spans.push(Span::styled(":Back", action_style));
            spans.push(separator.clone());
            spans.push(Span::styled("[Enter]", key_style));
            spans.push(Span::styled(":Play", action_style));
            spans.push(separator.clone());
            spans.push(Span::styled("[f]", key_style));
            spans.push(Span::styled(":Fav", action_style));
        }
        CurrentScreen::SeriesCategories => {
            spans.push(Span::styled("[Esc]", key_style));
            spans.push(Span::styled(":Home", action_style));
            spans.push(separator.clone());
            spans.push(Span::styled("[Tab]", key_style));
            spans.push(Span::styled(":Switch Pane", action_style));
            spans.push(separator.clone());
            spans.push(Span::styled("[Enter]", key_style));
            spans.push(Span::styled(":Select", action_style));
            spans.push(separator.clone());
            spans.push(Span::styled("[/]", key_style));
            spans.push(Span::styled(":Search", action_style));
            spans.push(separator.clone());
            spans.push(Span::styled("[x]", key_style));
            spans.push(Span::styled(":Settings", action_style));
        }
        CurrentScreen::SeriesStreams => {
            spans.push(Span::styled("[Esc]", key_style));
            spans.push(Span::styled(":Back", action_style));
            spans.push(separator.clone());
            spans.push(Span::styled("[Tab]", key_style));
            spans.push(Span::styled(":Switch Pane", action_style));
            spans.push(separator.clone());
            spans.push(Span::styled("[Enter]", key_style));
            let enter_action = match app.active_pane {
                Pane::Categories => ":Load Series",
                Pane::Streams => ":Load Episodes",
                Pane::Episodes => ":Play",
            };
            spans.push(Span::styled(enter_action, action_style));
            spans.push(separator.clone());
            spans.push(Span::styled("[/]", key_style));
            spans.push(Span::styled(":Search", action_style));
        }
        CurrentScreen::Settings | CurrentScreen::TimezoneSettings => match app.settings_state {
            crate::app::SettingsState::Main => {
                spans.push(Span::styled("[Esc]", key_style));
                spans.push(Span::styled(":Home", action_style));
                spans.push(separator.clone());
                spans.push(Span::styled("[Enter]", key_style));
                spans.push(Span::styled(":Select", action_style));
            }
            crate::app::SettingsState::ManageAccounts => {
                spans.push(Span::styled("[Esc]", key_style));
                spans.push(Span::styled(":Back", action_style));
                spans.push(separator.clone());
                spans.push(Span::styled("[n]", key_style));
                spans.push(Span::styled(":Add New", action_style));
                spans.push(separator.clone());
                spans.push(Span::styled("[d]", key_style));
                spans.push(Span::styled(":Delete", action_style));
                spans.push(separator.clone());
                spans.push(Span::styled("[Enter]", key_style));
                spans.push(Span::styled(":Edit", action_style));
            }
            crate::app::SettingsState::About => {
                spans.push(Span::styled("[Esc]", key_style));
                spans.push(Span::styled(":Back", action_style));
            }
        },
        _ => {
            spans.push(Span::styled("[q]", key_style));
            spans.push(Span::styled(":Quit", action_style));
        }
    };

    let p = Paragraph::new(Line::from(spans))
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
    f.render_widget(p, area);
}

fn render_loading(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(
            Style::default()
                .fg(MATRIX_GREEN)
                .add_modifier(Modifier::BOLD),
        );

    // Centered popup
    let popup_area = centered_rect(50, 15, area);
    f.render_widget(Clear, popup_area);
    f.render_widget(block, popup_area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(1),
            Constraint::Length(3),
        ])
        .margin(1)
        .split(popup_area);

    // Dynamic Digital Rain Artifact
    let rain_chars = vec![
        "ｱ", "ｲ", "ｳ", "ｴ", "ｵ", "ｶ", "ｷ", "ｸ", "ｹ", "ｺ", "ｻ", "ｼ", "ｽ", "ｾ", "ｿ", "ﾀ", "ﾁ", "ﾂ",
        "ﾃ", "ﾄ",
    ];
    let tick = app.loading_tick;

    let mut rain_lines = Vec::new();
    for i in 0..layout[0].height {
        let mut spans = Vec::new();
        for j in 0..layout[0].width {
            let offset = (i as u64 + j as u64 + tick) % rain_chars.len() as u64;
            let char = rain_chars[offset as usize];
            let opacity = if (j as u64 + tick / 2) % 3 == 0 {
                MATRIX_GREEN
            } else {
                DARK_GREEN
            };
            spans.push(Span::styled(char, Style::default().fg(opacity)));
        }
        rain_lines.push(Line::from(spans));
    }

    let rain_para = Paragraph::new(rain_lines).alignment(Alignment::Center);
    f.render_widget(rain_para, layout[0]);

    // Separator line
    f.render_widget(
        Paragraph::new("─".repeat(layout[1].width as usize)).style(Style::default().fg(DARK_GREEN)),
        layout[1],
    );

    let msg = app
        .loading_message
        .as_deref()
        .unwrap_or("SECURE_UPLINK_INITIALIZING...");
    let loading_text = Paragraph::new(format!(" > {} < ", msg.to_uppercase()))
        .style(
            Style::default()
                .fg(BRIGHT_GREEN)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE));

    f.render_widget(loading_text, layout[2]);
}

fn render_guide_popup(f: &mut Frame, app: &App, area: Rect) {
    if let Some(guide) = app.show_guide {
        let content = match guide {
            Guide::WhatIsApp => include_str!("content/what_is_this_app.md"),
            Guide::HowToGetPlaylists => include_str!("content/how_to_get_playlists.md"),
            Guide::WhatIsIptv => include_str!("content/what_is_iptv.md"),
        };

        // Simple markdown parsing
        let lines: Vec<Line> = content
            .lines()
            .map(|l| {
                if l.starts_with("# ") {
                    Line::from(Span::styled(
                        l.trim_start_matches("# ").to_uppercase(),
                        Style::default()
                            .fg(BRIGHT_GREEN)
                            .add_modifier(Modifier::BOLD),
                    ))
                } else if l.starts_with("### ") {
                    Line::from(Span::styled(
                        l.trim_start_matches("### ").to_uppercase(),
                        Style::default()
                            .fg(MATRIX_GREEN)
                            .add_modifier(Modifier::BOLD),
                    ))
                } else if l.starts_with("## ") {
                    Line::from(Span::styled(
                        l.trim_start_matches("## ").to_uppercase(),
                        Style::default()
                            .fg(MATRIX_GREEN)
                            .add_modifier(Modifier::BOLD),
                    ))
                } else if l.starts_with("* ") {
                    Line::from(vec![
                        Span::styled("  // ", Style::default().fg(BRIGHT_GREEN)),
                        Span::raw(l.trim_start_matches("* ").to_string()),
                    ])
                } else if l.starts_with("1. ")
                    || l.starts_with("2. ")
                    || l.starts_with("3. ")
                    || l.starts_with("4. ")
                {
                    Line::from(vec![
                        Span::styled(format!("  {} ", &l[..2]), Style::default().fg(Color::Green)),
                        Span::raw(l[3..].to_string()),
                    ])
                } else {
                    // Handle bold **text** simply
                    let mut spans = vec![];
                    let mut current = l;
                    while let Some(start) = current.find("**") {
                        spans.push(Span::raw(current[..start].to_string()));
                        let rest = &current[start + 2..];
                        if let Some(end) = rest.find("**") {
                            spans.push(Span::styled(
                                rest[..end].to_string(),
                                Style::default().add_modifier(Modifier::BOLD),
                            ));
                            current = &rest[end + 2..];
                        } else {
                            spans.push(Span::raw("**"));
                            current = rest;
                        }
                    }
                    spans.push(Span::raw(current.to_string()));
                    Line::from(spans)
                }
            })
            .collect();

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(Style::default().fg(DARK_GREEN))
            .title(Span::styled(
                " // SYSTEM_PROTOCOLS ",
                Style::default()
                    .fg(MATRIX_GREEN)
                    .add_modifier(Modifier::BOLD),
            ));

        let area = centered_rect(80, 80, area);
        f.render_widget(Clear, area);

        let p = Paragraph::new(lines)
            .block(block)
            .wrap(ratatui::widgets::Wrap { trim: true })
            .scroll((app.guide_scroll, 0));

        f.render_widget(p, area);

        // Footer hint
        let footer_text = vec![Line::from(vec![
            Span::styled(" Scroll with ", Style::default().fg(Color::LightBlue)),
            Span::styled(
                "j/k",
                Style::default()
                    .fg(MATRIX_GREEN)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" or ", Style::default().fg(Color::LightBlue)),
            Span::styled(
                "Arrows",
                Style::default()
                    .fg(MATRIX_GREEN)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  |  Press ", Style::default().fg(Color::LightBlue)),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(MATRIX_GREEN)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" or ", Style::default().fg(Color::LightBlue)),
            Span::styled(
                "Enter",
                Style::default()
                    .fg(MATRIX_GREEN)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" to close ", Style::default().fg(Color::LightBlue)),
        ])];
        let footer = Paragraph::new(footer_text).alignment(Alignment::Right);

        let footer_area = Rect::new(area.x, area.y + area.height - 1, area.width, 1);
        f.render_widget(footer, footer_area);
    }
}

/// Helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn render_help_popup(f: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" // COMMAND_LEGEND ")
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(DARK_GREEN));

    let area = centered_rect(60, 60, area);
    f.render_widget(Clear, area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(8), // Shortcuts
            Constraint::Min(0),    // Legend
        ])
        .split(area);

    // Shortcuts
    let shortcuts = vec![
        "Keyboard Shortcuts:",
        "",
        "  /       - Toggle Search",
        "  Tab     - Switch Panes (Categories/Streams)",
        "  m       - Switch Mode (Live TV / VOD)",
        "  f       - Toggle Favorite",
        "  Enter   - Select / Play",
        "  j / k   - Navigate Down / Up",
        "  q       - Quit",
    ];
    let shortcuts_p = Paragraph::new(shortcuts.join("\n")).style(Style::default().fg(Color::White));
    f.render_widget(shortcuts_p, chunks[0]);

    // Legend
    let legend = vec![
        "Icon Legend:",
        "",
        "  ★       - Favorite",
        "  ⭐       - VIP Content",
        "  🟣       - 4K Content",
        "  🔴 LIVE  - Live Event",
        "  📺       - Regular Channel",
        "  🎬       - Movie / VOD",
        "  🧸       - Kids Content",
        "",
        "Flags:",
        "  🇺🇸 🇬🇧, etc. - Country Origin",
        "",
        "VOD Badges:",
        "  ★ TOP   - Top Rated/Popular",
        "  🌐      - Multi-Subtitle Support",
    ];

    let legend_p = Paragraph::new(legend.join("\n")).style(Style::default().fg(Color::LightBlue));
    f.render_widget(legend_p, chunks[1]);
}

fn render_timezone_settings(f: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .title(" // TEMPORAL_SYNC_CONFIG ")
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(DARK_GREEN));

    // Increased height percentage to ensure space for list
    let area = centered_rect(60, 60, area);
    f.render_widget(Clear, area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // Input
            Constraint::Length(1), // Spacer
            Constraint::Min(1),    // List
        ])
        .split(area);

    // Input field
    let input = Paragraph::new(app.input_timezone.value())
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Custom IANA Timezone"),
        );

    f.render_widget(input, chunks[0]);

    // Timezone List
    let items: Vec<ListItem> = app
        .timezone_list
        .iter()
        .map(|tz| {
            let style = if tz == app.input_timezone.value() {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(Span::styled(format!("  {}", tz), style))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Select Timezone (↑/↓ to Navigate, Enter to Select) ")
                .borders(Borders::TOP)
                .border_style(Style::default().fg(Color::Gray)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" > ");

    f.render_stateful_widget(list, chunks[2], &mut app.timezone_list_state);
}

fn render_series_categories_pane(f: &mut Frame, app: &mut App, area: Rect, border_color: Color) {
    let title = format!(
        " // CATEGORIES ({}) ",
        app.series_categories.len()
    );

    let is_active = app.active_pane == Pane::Categories;
    let block_border = if is_active { BRIGHT_GREEN } else { border_color };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(block_border))
        .border_type(BorderType::Double)
        .title(Span::styled(
            title,
            Style::default()
                .fg(MATRIX_GREEN)
                .add_modifier(Modifier::BOLD),
        ));

    let items: Vec<ListItem> = app
        .series_categories
        .iter()
        .map(|c| {
            let base_color = if is_active {
                MATRIX_GREEN
            } else {
                DARK_GREEN
            };

            ListItem::new(Line::from(vec![
                Span::styled("📁 ", Style::default().fg(base_color)),
                Span::styled(c.category_name.clone(), Style::default().fg(base_color)),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(MATRIX_GREEN)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" » ");

    f.render_stateful_widget(list, area, &mut app.series_category_list_state);
}

fn render_series_streams_pane(f: &mut Frame, app: &mut App, area: Rect, border_color: Color) {
    let title = format!(" // SERIES ({}) ", app.series_streams.len());

    let is_active = app.active_pane == Pane::Streams;
    let block_border = if is_active { BRIGHT_GREEN } else { border_color };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(block_border))
        .border_type(BorderType::Double)
        .title(Span::styled(
            title,
            Style::default()
                .fg(MATRIX_GREEN)
                .add_modifier(Modifier::BOLD),
        ));

    let items: Vec<ListItem> = app
        .series_streams
        .iter()
        .map(|s| {
            let base_color = if is_active {
                MATRIX_GREEN
            } else {
                DARK_GREEN
            };

            // Parse year from name if present (e.g., "Show Name (2021)")
            let (show_name, year) = if let Some(start) = s.name.rfind('(') {
                if let Some(end) = s.name[start..].find(')') {
                    let year_part = &s.name[start + 1..start + end];
                    if year_part.len() == 4 && year_part.chars().all(|c| c.is_numeric()) {
                        (s.name[..start].trim(), Some(year_part))
                    } else {
                        (s.name.as_str(), None)
                    }
                } else {
                    (s.name.as_str(), None)
                }
            } else {
                (s.name.as_str(), None)
            };

            let mut spans = vec![];
            
            // TV icon
            spans.push(Span::styled("📺 ", Style::default().fg(base_color)));
            
            // Show name
            spans.push(Span::styled(
                show_name.to_string(),
                Style::default().fg(base_color).add_modifier(Modifier::BOLD),
            ));
            
            // Year badge if present
            if let Some(yr) = year {
                spans.push(Span::raw(" "));
                spans.push(Span::styled(
                    format!("[{}]", yr),
                    Style::default().fg(Color::Cyan),
                ));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(MATRIX_GREEN)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" » ");

    f.render_stateful_widget(list, area, &mut app.series_stream_list_state);
}

fn render_content_type_selection(f: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .title(" // CHOOSE_PATH ")
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(MATRIX_GREEN));

    let area = centered_rect(70, 50, area);
    f.render_widget(Clear, area);
    f.render_widget(block.clone(), area);

    let inner = block.inner(area);
    
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(10), // List (3 items)
            Constraint::Min(4),    // Quote
        ])
        .margin(1)
        .split(inner);

    // Title
    let title = Paragraph::new("Select Content Type:")
        .alignment(Alignment::Center)
        .style(Style::default().fg(BRIGHT_GREEN).add_modifier(Modifier::BOLD));
    f.render_widget(title, layout[0]);

    let selected = app.selected_content_type_index;

    // Create list items with proper styling
    let items: Vec<ListItem> = vec![
        ListItem::new(Line::from(vec![
            Span::styled("  🔴 ", Style::default().fg(Color::Red)),
            Span::styled("LIVE CHANNELS", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(" [Red Pill]", Style::default().fg(Color::Red)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("  🔵 ", Style::default().fg(Color::Blue)),
            Span::styled("MOVIES (VOD)", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(" [Blue Pill]", Style::default().fg(Color::Blue)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("  🐇 ", Style::default().fg(Color::White)),
            Span::styled("SERIES (VOD)", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(" [White Rabbit]", Style::default().fg(Color::White)),
        ])),
    ];

    let list = List::new(items)
        .highlight_style(
            Style::default()
                .bg(MATRIX_GREEN)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD)
        )
        .highlight_symbol("» ");

    let mut list_state = ratatui::widgets::ListState::default();
    list_state.select(Some(selected));
    
    f.render_stateful_widget(list, layout[1], &mut list_state);

    // Quote based on selection
    let (quote, color) = match selected {
        0 => ("\"You take the red pill... you stay in Wonderland,\nand I show you how deep the rabbit hole goes.\"", Color::Red),
        1 => ("\"You take the blue pill... the story ends,\nyou wake up in your bed and believe whatever you want to believe.\"", Color::Blue),
        _ => ("\"Follow the white rabbit.\"", Color::White),
    };

    let quote_para = Paragraph::new(quote)
        .alignment(Alignment::Center)
        .wrap(ratatui::widgets::Wrap { trim: true })
        .style(Style::default().fg(color).add_modifier(Modifier::ITALIC));
    
    f.render_widget(quote_para, layout[2]);

    // Instructions at bottom
    let instructions = Paragraph::new("↑/↓ or j/k: Navigate  │  Enter: Select  │  Esc: Back")
        .alignment(Alignment::Center)
        .style(Style::default().fg(DARK_GREEN));
    
    let bottom_area = Rect {
        x: inner.x,
        y: inner.y + inner.height - 2,
        width: inner.width,
        height: 1,
    };
    f.render_widget(instructions, bottom_area);
}

fn render_series_view(f: &mut Frame, app: &mut App, area: Rect) {
    // Calculate dynamic column widths based on content
    let (cat_width, series_width, episode_width) = calculate_three_column_split(
        &app.series_categories,
        &app.series_streams,
        area.width,
    );
    
    // 3-column view: Categories | Series | Episodes
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(cat_width),      // Categories - dynamic
            Constraint::Length(series_width),   // Series - dynamic
            Constraint::Min(episode_width),     // Episodes - remaining space
        ])
        .split(area);

    // Store areas for mouse click detection
    app.area_categories = chunks[0];
    app.area_streams = chunks[1];

    let border_color = MATRIX_GREEN;

    // Render categories pane
    render_series_categories_pane(f, app, chunks[0], border_color);

    // Render series (shows) pane
    render_series_streams_pane(f, app, chunks[1], border_color);

    // Render episodes pane
    render_series_episodes_pane(f, app, chunks[2], border_color);
}

fn render_series_episodes_pane(f: &mut Frame, app: &mut App, area: Rect, border_color: Color) {
    let title = format!(" // EPISODES ({}) ", app.series_episodes.len());

    let is_active = app.active_pane == Pane::Episodes;
    let block_border = if is_active { BRIGHT_GREEN } else { border_color };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(block_border))
        .border_type(BorderType::Double)
        .title(Span::styled(
            title,
            Style::default()
                .fg(MATRIX_GREEN)
                .add_modifier(Modifier::BOLD),
        ));

    let items: Vec<ListItem> = app
        .series_episodes
        .iter()
        .map(|ep| {
            let base_color = if is_active {
                MATRIX_GREEN
            } else {
                DARK_GREEN
            };

            // Color code by season for visual grouping
            let season_color = match ep.season % 5 {
                0 => Color::Cyan,
                1 => Color::LightBlue,
                2 => Color::LightMagenta,
                3 => Color::LightYellow,
                _ => Color::LightGreen,
            };

            let episode_title = ep.title.as_deref().unwrap_or("Untitled");
            
            let mut spans = vec![];
            
            // Play icon
            spans.push(Span::styled("▶ ", Style::default().fg(base_color)));
            
            // Season/Episode number with color coding
            spans.push(Span::styled(
                format!("S{:02}E{:02}", ep.season, ep.episode_num),
                Style::default().fg(season_color).add_modifier(Modifier::BOLD),
            ));
            
            // Separator
            spans.push(Span::styled(" │ ", Style::default().fg(DARK_GREEN)));
            
            // Episode title
            spans.push(Span::styled(
                episode_title.to_string(),
                Style::default().fg(base_color),
            ));

            ListItem::new(Line::from(spans))
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(MATRIX_GREEN)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" » ");

    f.render_stateful_widget(list, area, &mut app.series_episode_list_state);
}
