use crate::api::Category;
use crate::app::{App, CurrentScreen};
use crate::parser::{country_color, country_flag, parse_category, parse_stream};
use crate::ui::colors::{
    HIGHLIGHT_BG, MATRIX_GREEN, SOFT_GREEN, TEXT_DIM, TEXT_PRIMARY, TEXT_SECONDARY,
};
use crate::ui::common::stylize_channel_name;
use crate::ui::utils::visible_window;
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};
use std::sync::Arc;

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
    if app.category_grid_view {
        render_categories_grid(f, app, area, border_color);
    } else {
        render_categories_list(f, app, area, border_color);
    }
}

fn get_category_data(app: &App) -> (&Vec<Arc<Category>>, usize, &ratatui::widgets::TableState) {
    match app.current_screen {
        CurrentScreen::VodCategories | CurrentScreen::VodStreams => (
            &app.vod_categories,
            app.selected_vod_category_index,
            &app.vod_category_list_state,
        ),
        CurrentScreen::SeriesCategories | CurrentScreen::SeriesStreams => (
            &app.series_categories,
            app.selected_series_category_index,
            &app.series_category_list_state,
        ),
        _ => (
            &app.categories,
            app.selected_category_index,
            &app.category_list_state,
        ),
    }
}

fn render_categories_grid(f: &mut Frame, app: &mut App, area: Rect, border_color: Color) {
    let row_height: u16 = 3;
    let title = " categories ";
    let is_active = app.active_pane == crate::app::Pane::Categories;
    let inner_area =
        crate::ui::common::render_matrix_box_active(f, area, title, border_color, is_active);

    let max_name_len = app.session.max_category_name_len;
    let min_cell_width = (max_name_len as u16 + 10).max(14);
    let cols = ((inner_area.width / min_cell_width) as usize).clamp(1, 8);
    app.grid_cols = cols; // UPDATE BEFORE BORROWING categories_ref

    let (categories_ref, selected, _) = get_category_data(app);

    let max_rows = (inner_area.height / row_height).max(1) as usize;
    let items_per_page = max_rows * cols;
    let total = categories_ref.len();

    let page = selected / items_per_page.max(1);
    let start_idx = page * items_per_page;
    let end_idx = (start_idx + items_per_page).min(total);
    let page_items = end_idx - start_idx;
    let num_rows = ((page_items as f32) / (cols as f32)).ceil() as usize;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Length(row_height); num_rows])
        .split(inner_area);

    for (i, chunk_row) in chunks.iter().enumerate() {
        let cells = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Ratio(1, cols as u32); cols])
            .split(*chunk_row);

        for (j, cell_area) in cells.iter().enumerate() {
            let idx = start_idx + (i * cols) + j;
            if idx >= end_idx {
                break;
            }

            let category = &categories_ref[idx];
            let is_selected = idx == selected;

            let block_style = if is_selected {
                Style::default().fg(Color::Black).bg(MATRIX_GREEN)
            } else {
                Style::default().fg(Color::White).bg(Color::Rgb(0, 0, 0))
            };

            let parsed = if let Some(ref cached) = category.cached_parsed {
                cached.as_ref().clone()
            } else {
                parse_category(&category.category_name)
            };
            let upper_cat = &category.upper_clean_name;

            let card_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White))
                .style(block_style);

            let mut name_style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            // Apply cyan color to NEW RELEASES if not selected
            if upper_cat.contains("NEW RELEASES") && !is_selected {
                name_style = name_style.fg(Color::Cyan);
            }

            let spans = vec![Span::styled(parsed.display_name.as_str(), name_style)];

            let content = Paragraph::new(vec![Line::from(spans)])
                .block(card_block)
                .alignment(ratatui::layout::Alignment::Left);

            f.render_widget(content, *cell_area);
        }
    }
}

fn render_categories_list(f: &mut Frame, app: &mut App, area: Rect, border_color: Color) {
    let title = " categories ";
    let is_active = app.active_pane == crate::app::Pane::Categories;
    let inner_area =
        crate::ui::common::render_matrix_box_active(f, area, title, border_color, is_active);

    let max_name_len = app.session.max_category_name_len as u16;
    let min_col_width = (max_name_len + 15).max(30);
    let cols = (inner_area.width / min_col_width).max(1) as usize;

    app.grid_cols = cols; // UPDATE BEFORE BORROWING categories_ref

    let (categories_ref, selected, list_state_ref) = get_category_data(app);
    let total = categories_ref.len();
    if total == 0 {
        return;
    }

    let mut constraints = Vec::new();
    for i in 0..cols {
        constraints.push(ratatui::layout::Constraint::Ratio(1, cols as u32));
        if i < cols - 1 {
            constraints.push(ratatui::layout::Constraint::Length(1));
        }
    }

    let rows = total.div_ceil(cols);
    let full_cols = if total % cols == 0 {
        cols
    } else {
        total % cols
    };

    // Virtualization: only render visible rows
    let mut selected_row = selected;
    for col in 0..cols {
        let col_size = if col < full_cols { rows } else { rows - 1 };
        if selected_row < col_size {
            break;
        }
        selected_row -= col_size;
    }
    let (start_row, end_row) =
        crate::ui::utils::visible_window(selected_row, rows, inner_area.height as usize);

    let mut list_items = Vec::new();

    for r in start_row..end_row {
        let mut cells = Vec::new();

        for c in 0..cols {
            let col_size = if c < full_cols { rows } else { rows - 1 };

            if r < col_size {
                let mut base_idx = 0;
                for i in 0..c {
                    base_idx += if i < full_cols { rows } else { rows - 1 };
                }
                let abs_idx = base_idx + r;
                let cat = &categories_ref[abs_idx];

                let is_selected = abs_idx == selected;

                let upper_cat = &cat.upper_clean_name;

                let fav_marker = if app.config.favorites.categories.contains(&cat.category_id) {
                    "*"
                } else {
                    " "
                };
                let pre_pad = if is_selected && is_active {
                    "█ "
                } else {
                    "  "
                };
                let name_clean = format!("{}{}{}", pre_pad, fav_marker, upper_cat);

                let mut style = if is_selected && is_active {
                    list_highlight_style()
                } else if is_selected && !is_active {
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::REVERSED)
                } else if !is_active {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default().fg(TEXT_PRIMARY)
                };

                // Apply cyan color to NEW RELEASES if not selected (which has its own highlight)
                if upper_cat.contains("NEW RELEASES") && !is_selected {
                    style = style.fg(Color::Cyan);
                }

                cells.push(Cell::from(name_clean).style(style));
            } else {
                cells.push(Cell::from(""));
            }

            if c < cols - 1 {
                cells.push(Cell::from(""));
            }
        }

        list_items.push(Row::new(cells));
    }

    let list_widget = Table::new(list_items, constraints).column_spacing(0);

    // Convert selected row index to a relative selection for TableState
    let mut state = *list_state_ref;
    state.select(Some(selected_row - start_row));

    // Update the app's state for the specific screen
    match app.current_screen {
        CurrentScreen::VodCategories | CurrentScreen::VodStreams => {
            app.vod_category_list_state = state;
        }
        CurrentScreen::SeriesCategories | CurrentScreen::SeriesStreams => {
            app.series_category_list_state = state;
        }
        _ => {
            app.category_list_state = state;
        }
    }

    f.render_stateful_widget(list_widget, inner_area, &mut state);
}

pub fn render_streams_pane(f: &mut Frame, app: &mut App, area: Rect, border_color: Color) {
    let visible_height = area.height.saturating_sub(2) as usize;
    let (start, end) = visible_window(app.selected_stream_index, app.streams.len(), visible_height);
    let selected = app.selected_stream_index;

    let user_tz_str = app.config.get_user_timezone();
    let user_tz: Tz = user_tz_str.parse().unwrap_or(chrono_tz::UTC);
    let tz_display = format!(
        "{} ({})",
        user_tz_str,
        Utc::now().with_timezone(&user_tz).format("%Z")
    );

    let items: Vec<Row> = app
        .streams
        .iter()
        .enumerate()
        .skip(start)
        .take(end - start)
        .map(|(idx, s)| {
            let _s_id = crate::api::get_id_str(&s.stream_id);
            let display_name = &s.name;

            // Use cached parsed metadata (pre-computed). Fallback computes on the fly.
            let parsed = if let Some(ref cached) = s.cached_parsed {
                cached.as_ref().clone()
            } else {
                crate::parser::parse_stream(display_name, app.session.provider_timezone.as_deref())
            };

            if parsed.is_separator {
                let label = if parsed.display_name.is_empty() {
                    "───".to_string()
                } else {
                    format!("─── {} ───", parsed.display_name)
                };
                return Row::new(vec![
                    Cell::from(""),
                    Cell::from(""),
                    Cell::from(""), // Quality
                    Cell::from(Line::from(vec![
                        ratatui::text::Span::styled("     ", Style::default()),
                        ratatui::text::Span::styled("│", Style::default().fg(TEXT_DIM)),
                        ratatui::text::Span::styled(
                            label,
                            Style::default().fg(TEXT_DIM).add_modifier(Modifier::DIM),
                        ),
                    ])),
                    Cell::from(""), // NOW PLAYING
                    Cell::from(""), // HEALTH
                ]);
            }

            let mut spans = vec![];

            // 0. Quality badge (Structured Column)
            let (quality_text, quality_color) = match parsed.quality {
                Some(crate::parser::Quality::UHD4K) => ("[4K] ", Color::Rgb(255, 0, 255)),
                Some(crate::parser::Quality::FHD) => ("[FHD]", Color::Rgb(255, 215, 0)),
                Some(crate::parser::Quality::HD) => ("[HD] ", Color::Rgb(0, 255, 255)),
                Some(crate::parser::Quality::SD) => ("[SD] ", Color::White),
                None => ("     ", Color::White),
            };
            let quality_span = Span::styled(
                quality_text,
                Style::default()
                    .fg(quality_color)
                    .add_modifier(Modifier::BOLD),
            );

            // 1. Channel number column
            let ch_num_str = format!("{}", idx + 1);

            // 2. Live Status / Icon
            let now = Utc::now();
            let is_ended = parsed.stop_time.map(|t| now > t).unwrap_or(false);
            let is_live = parsed.start_time.map(|st| now >= st).unwrap_or(false) && !is_ended;
            let is_blink_on = (app.session.loading_tick / 4).is_multiple_of(2);

            let status_span = if is_live {
                let live_color = if is_blink_on {
                    Color::Rgb(255, 100, 100)
                } else {
                    TEXT_DIM
                };
                ratatui::text::Span::styled(" ● ", Style::default().fg(live_color))
            } else {
                ratatui::text::Span::styled("   ", Style::default())
            };

            // 3. Icon Logic (Pre-calculated)
            if let Some(ref icon) = parsed.league_icon {
                let style = match icon.trim() {
                    "NBA" => Style::default().fg(Color::Rgb(255, 140, 0)),
                    "NFL" => Style::default().fg(Color::Rgb(210, 180, 140)),
                    "TENNIS" => Style::default().fg(Color::Rgb(144, 238, 144)),
                    "RACE" => Style::default().fg(Color::Rgb(255, 100, 100)),
                    _ => Style::default(),
                };
                spans.push(ratatui::text::Span::styled(icon.clone(), style));
            }

            // 4. Favorite indicator
            let s_id = crate::api::get_id_str(&s.stream_id);
            if app.config.favorites.streams.contains(&s_id) {
                spans.push(ratatui::text::Span::styled(
                    "* ",
                    Style::default().fg(MATRIX_GREEN),
                ));
            }

            // 5. Country Flag
            if let Some(ref country) = parsed.country {
                let is_us_en =
                    country == "US" || country == "USA" || country == "AM" || country == "EN";
                if !(app.config.playlist_mode.is_merica_variant() && is_us_en) {
                    let flag = country_flag(country);
                    if !flag.is_empty() {
                        spans.push(ratatui::text::Span::raw(format!("{} ", flag)));
                    }
                }
            }

            // 6. Name / Matchup
            let is_league = parsed
                .country
                .as_ref()
                .map(|cc| {
                    ["NBA", "NFL", "MLB", "NHL", "UFC", "SPORTS", "PPV"].contains(&cc.as_str())
                })
                .unwrap_or(false);

            let score_data_for_color = app.get_score_for_stream(&parsed.display_name);
            let name_color = if is_league {
                parsed
                    .country
                    .as_ref()
                    .map(|cc| country_color(cc))
                    .unwrap_or(TEXT_PRIMARY)
            } else {
                MATRIX_GREEN
            };

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
                    format!(
                        "{:>width$}",
                        format!("{} [{}]", truncated_name, score.home_score),
                        width = width
                    )
                } else {
                    format!("{:>width$}", h_full, width = width)
                };

                let a_display = if a_full.len() > width {
                    let name_avail = width.saturating_sub(score.away_score.len() + 3);
                    let truncated_name = if score.away_team.len() > name_avail {
                        format!("{}..", &score.away_team[..name_avail.saturating_sub(2)])
                    } else {
                        score.away_team.clone()
                    };
                    format!(
                        "{:<width$}",
                        format!("[{}] {}", score.away_score, truncated_name),
                        width = width
                    )
                } else {
                    format!("{:<width$}", a_full, width = width)
                };

                spans.push(ratatui::text::Span::styled(
                    h_display,
                    Style::default().fg(h_color).add_modifier(Modifier::BOLD),
                ));
                spans.push(ratatui::text::Span::styled(
                    " vs ",
                    Style::default().fg(TEXT_DIM),
                ));
                spans.push(ratatui::text::Span::styled(
                    a_display,
                    Style::default().fg(a_color).add_modifier(Modifier::BOLD),
                ));

                spans.push(ratatui::text::Span::raw("   "));
            } else {
                // Standard rendering
                let max_width = area.width.saturating_sub(15) as usize;
                let mut display_name = parsed.display_name.clone();
                let mut year_suffix = String::new();

                if let Some(y) = &parsed.year {
                    if !y.is_empty() {
                        let y_str = format!(" [{}]", y);
                        let total_len = display_name.len() + y_str.len();

                        if total_len > max_width {
                            let name_avail = max_width.saturating_sub(y_str.len());
                            if name_avail > 3 {
                                display_name =
                                    format!("{}...", &display_name[..name_avail.saturating_sub(3)]);
                            } else {
                                display_name = display_name.chars().take(name_avail).collect();
                            }
                        }
                        year_suffix = y_str;
                    }
                } else if display_name.len() > max_width {
                    if max_width > 3 {
                        display_name =
                            format!("{}...", &display_name[..max_width.saturating_sub(3)]);
                    } else {
                        display_name = display_name.chars().take(max_width).collect();
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
                    false, // show_quality: Disable in Live Streams with QUAL column
                );

                if !year_suffix.is_empty() {
                    styled_name.push(ratatui::text::Span::styled(
                        year_suffix,
                        Style::default().fg(TEXT_SECONDARY),
                    ));
                }

                spans.extend(styled_name);

                // EPG "Now Playing" — rendered in its own column below
            }

            // 7. Score/Clock Logic
            let score_data = app.get_score_for_stream(&parsed.display_name);

            let (_final_is_live, final_is_ended, status_text, score_text) =
                if let Some(score) = score_data {
                    let is_active = score.status_state == "in";
                    let is_finished = score.status_state == "post";
                    let clock = if is_finished {
                        "Final".to_string()
                    } else if is_active {
                        if !score.display_clock.is_empty() && score.display_clock != "00:00" {
                            score
                                .display_clock
                                .split(' ')
                                .next()
                                .unwrap_or(&score.display_clock)
                                .to_string()
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
                    let mut clock_style = Style::default()
                        .fg(Color::Rgb(255, 200, 80))
                        .add_modifier(Modifier::BOLD);
                    if final_is_ended {
                        clock_style = Style::default()
                            .fg(TEXT_DIM)
                            .add_modifier(Modifier::CROSSED_OUT);
                    }
                    spans.push(ratatui::text::Span::styled(
                        format!(" [{}] ", clock),
                        clock_style,
                    ));
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
                    spans.push(ratatui::text::Span::styled(
                        score,
                        Style::default()
                            .fg(TEXT_PRIMARY)
                            .add_modifier(Modifier::BOLD),
                    ));
                }
            }

            if final_is_ended {
                spans.push(ratatui::text::Span::styled(
                    " ended",
                    Style::default().fg(TEXT_DIM),
                ));
            }

            // 8. Location
            if parsed.sports_event.is_none() {
                if let Some(loc) = &parsed.location {
                    spans.push(ratatui::text::Span::styled(
                        format!(" ({})", loc),
                        Style::default().fg(TEXT_SECONDARY),
                    ));
                }
            }

            // 9. Network Health
            let health = app
                .sports
                .stream_health_cache
                .get(&s_id)
                .copied()
                .or(s.latency_ms);
            let health_span = latency_to_bars(health);

            // 10. EPG "Now Playing" column
            let s_id_str = crate::api::get_id_str(&s.stream_id);
            let epg_cell = if let Some(epg_title) = app.epg_cache.get(&s_id_str) {
                if !epg_title.is_empty() {
                    Cell::from(Span::styled(
                        epg_title.clone(),
                        Style::default().fg(Color::Rgb(140, 140, 180)),
                    ))
                } else {
                    Cell::from("")
                }
            } else {
                Cell::from("")
            };

            // Construct Table Row
            let is_selected = idx == selected;
            let row_style = if is_selected {
                Style::default().bg(HIGHLIGHT_BG)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(format!("{:>5}", ch_num_str)).style(Style::default().fg(TEXT_DIM)),
                Cell::from(status_span),
                Cell::from(quality_span),
                Cell::from(Line::from(spans)),
                epg_cell,
                Cell::from(health_span),
            ])
            .style(row_style)
        })
        .collect();

    let title = format!("streams · {}", tz_display);
    let is_active = app.active_pane == crate::app::Pane::Streams;
    let inner_area =
        crate::ui::common::render_matrix_box_active(f, area, &title, border_color, is_active);

    // Empty state messaging
    if app.streams.is_empty() {
        let msg = if app.session.loading_message.is_some() {
            "Loading channels..."
        } else if app.search_mode && !app.search_state.query.is_empty() {
            "No matches — press esc to clear search"
        } else {
            "No channels — press esc to go back"
        };
        let empty_msg = Paragraph::new(msg)
            .style(Style::default().fg(TEXT_DIM))
            .alignment(ratatui::layout::Alignment::Center);
        let centered = Rect {
            x: inner_area.x,
            y: inner_area.y + inner_area.height / 2,
            width: inner_area.width,
            height: 1,
        };
        f.render_widget(empty_msg, centered);
        return;
    }

    // ── Table Layout (Advanced Alignment) ──
    let constraints = [
        Constraint::Length(6), // Ch #
        Constraint::Length(3), // Status dot
        Constraint::Length(7), // Quality [FHD]
        Constraint::Fill(3),   // Name / Info (primary)
        Constraint::Fill(1),   // NOW PLAYING (EPG)
        Constraint::Length(6), // Health bars
    ];

    let header_style = Style::default()
        .fg(TEXT_SECONDARY)
        .add_modifier(Modifier::BOLD);
    let header = Row::new(vec![
        Cell::from("  CH#").style(header_style),
        Cell::from("").style(header_style),
        Cell::from(" QUAL").style(header_style),
        Cell::from(" CHANNEL").style(header_style),
        Cell::from(" NOW PLAYING").style(header_style),
        Cell::from("HEALTH").style(header_style),
    ])
    .height(1)
    .style(Style::default().bg(Color::Rgb(10, 25, 10))); // Subtle dark green header bg

    let table = Table::new(items, constraints)
        .header(header)
        .column_spacing(1)
        .row_highlight_style(list_highlight_style())
        .highlight_symbol("▎");

    // Map the ListState selection to a TableState for rendering
    let mut table_state = ratatui::widgets::TableState::default();
    table_state.select(Some(selected - start));

    f.render_stateful_widget(table, inner_area, &mut table_state);
}

pub fn render_global_search_pane(f: &mut Frame, app: &mut App, area: Rect) {
    let visible_height = area.height.saturating_sub(2) as usize;
    let (start, end) = visible_window(
        app.selected_stream_index,
        app.global_search_results.len(),
        visible_height,
    );
    let selected = app.selected_stream_index;

    let user_tz_str = app.config.get_user_timezone();
    let user_tz: Tz = user_tz_str.parse().unwrap_or(chrono_tz::UTC);

    let items: Vec<Row> = app
        .global_search_results
        .iter()
        .enumerate()
        .skip(start)
        .take(end - start)
        .map(|(idx, s)| {
            let s_id = crate::api::get_id_str(&s.stream_id);
            let display_name = &s.name;

            let parsed = if let Some(ref cached) = s.cached_parsed {
                cached.as_ref().clone()
            } else {
                crate::parser::parse_stream(display_name, app.session.provider_timezone.as_deref())
            };

            if parsed.is_separator {
                return Row::new(vec![
                    Cell::from(""),
                    Cell::from(""),
                    Cell::from(Line::from(vec![
                        ratatui::text::Span::styled("     ", Style::default()),
                        ratatui::text::Span::styled("│", Style::default().fg(TEXT_DIM)),
                        ratatui::text::Span::styled(
                            format!("─── {} ───", parsed.display_name),
                            Style::default().fg(TEXT_DIM).add_modifier(Modifier::DIM),
                        ),
                    ])),
                    Cell::from(""),
                ]);
            }

            let mut spans = vec![];

            // Channel Number Logic
            let ch_num_str = if let Some(ref num) = s.num {
                let num_val = num.to_string_value().unwrap_or_default();
                if num_val == "0" || num_val.is_empty() {
                    crate::api::get_id_str(&s.stream_id)
                } else {
                    num_val
                }
            } else if let Some(ref cp) = parsed.channel_prefix {
                cp.clone()
            } else {
                crate::api::get_id_str(&s.stream_id)
            };

            let ch_display = if ch_num_str.len() > 5 {
                format!("..{}", &ch_num_str[ch_num_str.len().saturating_sub(4)..])
            } else {
                ch_num_str.clone()
            };

            // Status Icon logic
            let now = Utc::now();
            let is_ended = parsed.stop_time.map(|t| now > t).unwrap_or(false);
            let is_live = parsed.start_time.map(|st| now >= st).unwrap_or(false) && !is_ended;
            let is_blink_on = (app.session.loading_tick / 4).is_multiple_of(2);

            let status_span = if is_live {
                let live_color = if is_blink_on {
                    Color::Rgb(255, 100, 100)
                } else {
                    TEXT_DIM
                };
                ratatui::text::Span::styled(" ● ", Style::default().fg(live_color))
            } else {
                ratatui::text::Span::styled("   ", Style::default())
            };

            let name_color = if s.stream_type == "live" {
                MATRIX_GREEN
            } else {
                TEXT_PRIMARY
            };

            let (mut styled_name, _) = stylize_channel_name(
                &parsed.display_name,
                false,
                is_ended,
                parsed.quality,
                None,
                parsed.sports_event.as_ref(),
                Style::default().fg(name_color),
                false, // show_quality: Disable in Live Streams with QUAL column
            );

            if let Some(y) = &parsed.year {
                if !y.is_empty() {
                    styled_name.push(ratatui::text::Span::styled(
                        format!(" [{}]", y),
                        Style::default().fg(TEXT_SECONDARY),
                    ));
                }
            }

            spans.extend(styled_name);

            if s.stream_type == "live" {
                if let Some(st) = parsed.start_time {
                    let time_str = format!(" {} ", format_relative_time(st, &user_tz));
                    spans.push(ratatui::text::Span::styled(
                        time_str,
                        Style::default().fg(TEXT_SECONDARY),
                    ));
                }
            }

            // Network Health
            let health = app
                .sports
                .stream_health_cache
                .get(&s_id)
                .copied()
                .or(s.latency_ms);
            let health_span = latency_to_bars(health);

            if let Some(account) = &s.account_name {
                spans.push(ratatui::text::Span::styled(
                    format!(" [{}]", account),
                    Style::default().fg(TEXT_DIM),
                ));
            }

            // Construct Row
            let is_selected = idx == selected;
            let row_style = if is_selected {
                Style::default().bg(HIGHLIGHT_BG)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(format!("{:>5}", ch_display)).style(Style::default().fg(TEXT_DIM)),
                Cell::from(status_span),
                Cell::from(Line::from(spans)),
                Cell::from(health_span),
            ])
            .style(row_style)
        })
        .collect();

    let title = if app.search_state.query.is_empty() {
        format!(
            "search ({}) — type to filter",
            app.global_search_results.len()
        )
    } else {
        format!(
            "search: \"{}\" ({}/{})",
            app.search_state.query,
            selected.saturating_add(1),
            app.global_search_results.len()
        )
    };
    let inner_area = crate::ui::common::render_matrix_box(f, area, &title, SOFT_GREEN);

    if app.global_search_results.is_empty() {
        return; // Empty state handled by default box
    }

    let constraints = [
        Constraint::Length(6), // Ch #
        Constraint::Length(3), // Status dot
        Constraint::Fill(1),   // Name / Info
        Constraint::Length(5), // Health bars
    ];

    let header_style = Style::default()
        .fg(TEXT_SECONDARY)
        .add_modifier(Modifier::BOLD);
    let header = Row::new(vec![
        Cell::from("  CH#").style(header_style),
        Cell::from("").style(header_style),
        Cell::from(" SEARCH RESULTS").style(header_style),
        Cell::from("HLTH").style(header_style),
    ])
    .height(1)
    .style(Style::default().bg(Color::Rgb(10, 20, 10)));

    let table = Table::new(items, constraints)
        .header(header)
        .column_spacing(1)
        .row_highlight_style(list_highlight_style())
        .highlight_symbol("▎");

    let mut table_state = ratatui::widgets::TableState::default();
    table_state.select(Some(selected - start));

    f.render_stateful_widget(table, inner_area, &mut table_state);
}

fn latency_to_bars(latency: Option<u64>) -> ratatui::text::Span<'static> {
    match latency {
        Some(l) if l < 200 => {
            ratatui::text::Span::styled(" ▰▰▰", Style::default().fg(MATRIX_GREEN))
        }
        Some(l) if l < 600 => {
            ratatui::text::Span::styled(" ▰▰░", Style::default().fg(Color::Rgb(255, 200, 80)))
        }
        Some(_) => {
            ratatui::text::Span::styled(" ▰░░", Style::default().fg(Color::Rgb(255, 100, 100)))
        }
        None => ratatui::text::Span::raw(""),
    }
}

/// JiraTUI-inspired detail panel for the selected stream.
/// Shows labeled fields in bordered groups, visible for all streams.
/// Uses debounce: during fast scrolling, shows minimal info to avoid lag.
pub fn render_channel_detail_panel(f: &mut Frame, app: &mut App, area: Rect, border_color: Color) {
    let selected_idx = app.selected_stream_index;
    let focused_stream = if app.current_screen == CurrentScreen::GlobalSearch {
        app.global_search_results.get(selected_idx)
    } else {
        app.streams.get(selected_idx)
    };

    let Some(s) = focused_stream else {
        // Empty detail panel
        let inner = crate::ui::common::render_matrix_box(f, area, "details", border_color);
        let msg = Paragraph::new("No channel selected")
            .style(Style::default().fg(TEXT_DIM))
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(msg, inner);
        return;
    };

    // Debounce removed: The `parsed_cache` guarantees O(1) allocation overhead here.

    let parsed = if let Some(ref cached) = s.cached_parsed {
        cached.as_ref().clone()
    } else {
        parse_stream(&s.name, app.session.provider_timezone.as_deref())
    };

    let inner = crate::ui::common::render_matrix_box(f, area, "details", border_color);

    let label_color = MATRIX_GREEN;
    let value_style = Style::default().fg(TEXT_PRIMARY);
    let dim_style = Style::default().fg(TEXT_DIM);
    let w = inner.width as usize;

    let mut lines: Vec<Line> = Vec::new();

    // ── Summary (JiraTUI top field) ──
    // Show stream_id in summary header for easy channel identification
    let ch_id_str = crate::api::get_id_str(&s.stream_id);
    let summary_label = format!("Ch {} · Summary", ch_id_str);
    lines.push(Line::from(vec![Span::styled(
        summary_label,
        Style::default()
            .fg(MATRIX_GREEN)
            .add_modifier(Modifier::BOLD),
    )]));
    // Truncate display name to fit panel width
    let display = if parsed.display_name.chars().count() > w.saturating_sub(1) {
        let s: String = parsed
            .display_name
            .chars()
            .take(w.saturating_sub(2))
            .collect();
        format!("{}…", s)
    } else {
        parsed.display_name.clone()
    };
    lines.push(Line::from(Span::styled(
        display,
        value_style.add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled("─".repeat(w), dim_style)));

    // ── Status Row (LIVE NOW / Starts at / Ended) ──
    {
        let user_tz_str = app.config.get_user_timezone();
        let user_tz: Tz = user_tz_str.parse().unwrap_or(chrono_tz::UTC);
        let now = Utc::now();
        let is_blink_on = (app.session.loading_tick / 4).is_multiple_of(2);

        // Check live score data first (most authoritative for sports)
        let score_data = app.get_score_for_stream(&parsed.display_name);
        let (status_label, status_spans) = if let Some(score) = score_data {
            if score.status_state == "in" {
                let clock = if !score.display_clock.is_empty() && score.display_clock != "00:00" {
                    format!(" — {}", score.display_clock)
                } else {
                    String::new()
                };
                let live_color = if is_blink_on {
                    Color::Rgb(255, 100, 100)
                } else {
                    TEXT_DIM
                };
                (
                    "Status",
                    vec![
                        Span::styled(
                            if is_blink_on { "● " } else { "  " },
                            Style::default().fg(live_color),
                        ),
                        Span::styled(
                            "LIVE NOW",
                            Style::default().fg(live_color).add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(clock, Style::default().fg(TEXT_SECONDARY)),
                    ],
                )
            } else if score.status_state == "post" {
                (
                    "Status",
                    vec![Span::styled("✓ Ended", Style::default().fg(TEXT_DIM))],
                )
            } else {
                // Pre-game — show start time if available
                if let Some(st) = parsed.start_time {
                    let time_str = format_relative_time(st, &user_tz);
                    (
                        "Status",
                        vec![Span::styled(
                            format!("Starts {}", time_str),
                            Style::default().fg(Color::Rgb(255, 200, 80)),
                        )],
                    )
                } else {
                    (
                        "Status",
                        vec![Span::styled(
                            &score.status_detail,
                            Style::default().fg(TEXT_SECONDARY),
                        )],
                    )
                }
            }
        } else {
            // No score data — use parsed timing
            let has_start = parsed.start_time.is_some();
            let is_ended = parsed.stop_time.map(|t| now > t).unwrap_or(false);
            let is_live = parsed.start_time.map(|st| now >= st).unwrap_or(false) && !is_ended;

            if is_live {
                let live_color = if is_blink_on {
                    Color::Rgb(255, 100, 100)
                } else {
                    TEXT_DIM
                };
                (
                    "Status",
                    vec![
                        Span::styled(
                            if is_blink_on { "● " } else { "  " },
                            Style::default().fg(live_color),
                        ),
                        Span::styled(
                            "LIVE NOW",
                            Style::default().fg(live_color).add_modifier(Modifier::BOLD),
                        ),
                    ],
                )
            } else if is_ended {
                (
                    "Status",
                    vec![Span::styled("✓ Ended", Style::default().fg(TEXT_DIM))],
                )
            } else if has_start {
                let st = parsed.start_time.unwrap();
                let time_str = format_relative_time(st, &user_tz);
                (
                    "Status",
                    vec![Span::styled(
                        format!("Starts {}", time_str),
                        Style::default().fg(Color::Rgb(255, 200, 80)),
                    )],
                )
            } else {
                ("Status", vec![Span::styled("—", dim_style)])
            }
        };

        lines.push(Line::from(Span::styled(
            status_label,
            Style::default().fg(label_color),
        )));
        lines.push(Line::from(status_spans));

        // ── Schedule Row (start → end time + duration) ──
        let schedule_data: Option<(String, String, String, String)> = if parsed.start_time.is_some()
            || parsed.stop_time.is_some()
        {
            let start_str = parsed
                .start_time
                .map(|st| {
                    let local = st.with_timezone(&user_tz);
                    local.format("%I:%M %p").to_string()
                })
                .unwrap_or_else(|| "—".to_string());

            let end_str = parsed
                .stop_time
                .map(|et| {
                    let local = et.with_timezone(&user_tz);
                    local.format("%I:%M %p").to_string()
                })
                .unwrap_or_else(|| "—".to_string());

            let duration_str = if let (Some(st), Some(et)) = (parsed.start_time, parsed.stop_time) {
                let dur = et.signed_duration_since(st);
                let hours = dur.num_hours();
                let mins = dur.num_minutes() % 60;
                if hours > 0 {
                    format!(" ({}h {}m)", hours, mins)
                } else {
                    format!(" ({}m)", mins)
                }
            } else {
                String::new()
            };

            let tz_abbr = now.with_timezone(&user_tz).format("%Z").to_string();
            Some((start_str, end_str, duration_str, tz_abbr))
        } else {
            None
        };

        if let Some((start_str, end_str, duration_str, tz_abbr)) = schedule_data {
            lines.push(Line::from(Span::styled(
                "Schedule",
                Style::default().fg(label_color),
            )));
            lines.push(Line::from(vec![
                Span::styled(start_str, value_style),
                Span::styled(" → ", Style::default().fg(TEXT_DIM)),
                Span::styled(end_str, value_style),
                Span::styled(format!(" {}", tz_abbr), Style::default().fg(TEXT_SECONDARY)),
                Span::styled(duration_str, Style::default().fg(TEXT_SECONDARY)),
            ]));
        }

        lines.push(Line::from(Span::styled("─".repeat(w), dim_style)));
    }

    // ── Field Grid (JiraTUI 2-column fields) ──
    let has_quality = parsed.quality.is_some();
    let has_region = parsed.country.is_some();
    let half = w / 2;

    if has_quality || has_region {
        let region_str = parsed
            .country
            .as_ref()
            .map(|c| {
                let flag = country_flag(c);
                if !flag.is_empty() {
                    format!("{} {}", flag, c)
                } else {
                    c.clone()
                }
            })
            .unwrap_or_else(|| "—".to_string());

        lines.push(Line::from(vec![
            Span::styled(
                format!("{:<half$}", "Quality"),
                Style::default().fg(label_color),
            ),
            Span::styled("Region", Style::default().fg(label_color)),
        ]));
        let quality_span = if let Some(q) = &parsed.quality {
            Span::styled(
                format!("{:<half$}", format!(" {} ", q.badge())),
                Style::default()
                    .fg(Color::Black)
                    .bg(q.color())
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled(format!("{:<half$}", "—"), dim_style)
        };
        lines.push(Line::from(vec![
            quality_span,
            Span::styled(region_str.clone(), value_style),
        ]));
        lines.push(Line::from(Span::styled("─".repeat(w), dim_style)));
    }

    // Category
    let cat_display = s
        .category_id
        .as_ref()
        .map(|cat_id| {
            app.categories
                .iter()
                .find(|c| &c.category_id == cat_id)
                .map(|c| c.category_name.as_str())
                .or_else(|| {
                    app.all_categories
                        .iter()
                        .find(|c| &c.category_id == cat_id)
                        .map(|c| c.category_name.as_str())
                })
                .unwrap_or(cat_id.as_str())
                .to_string()
        })
        .unwrap_or_default();

    if !cat_display.is_empty() && cat_display.to_lowercase() != "null" {
        lines.push(Line::from(Span::styled(
            "Category",
            Style::default().fg(label_color),
        )));
        let cat_trunc = if cat_display.chars().count() > w {
            let s: String = cat_display.chars().take(w.saturating_sub(1)).collect();
            format!("{}…", s)
        } else {
            cat_display
        };
        lines.push(Line::from(Span::styled(cat_trunc, value_style)));
        lines.push(Line::from(Span::styled("─".repeat(w), dim_style)));
    }

    // EPG Now Playing (own row)
    let s_id_for_epg = crate::api::get_id_str(&s.stream_id);
    let now_playing = app
        .epg_cache
        .get(&s_id_for_epg)
        .cloned()
        .filter(|s| !s.is_empty() && s.to_lowercase() != "null");

    if let Some(np) = now_playing {
        let epg_trunc = if np.chars().count() > w.saturating_sub(1) {
            let sc: String = np.chars().take(w.saturating_sub(2)).collect();
            format!("{}…", sc)
        } else {
            np
        };

        lines.push(Line::from(Span::styled(
            "Now Playing",
            Style::default().fg(label_color),
        )));
        lines.push(Line::from(Span::styled(epg_trunc, value_style)));
        lines.push(Line::from(Span::styled("─".repeat(w), dim_style)));
    }

    // Stream ID (own row)
    let sid = s_id_for_epg.clone();
    if !sid.is_empty() && sid.to_lowercase() != "null" {
        lines.push(Line::from(vec![Span::styled(
            "Stream ID",
            Style::default().fg(label_color),
        )]));
        lines.push(Line::from(Span::styled(&sid, dim_style)));
        lines.push(Line::from(Span::styled("─".repeat(w), dim_style)));
    }

    // Favorite + Rating row
    let s_id = crate::api::get_id_str(&s.stream_id);
    let is_fav = app.config.favorites.streams.contains(&s_id);
    let fav_str = if is_fav { "* Yes" } else { "- No" };
    let rating_str = s
        .rating
        .filter(|r| *r > 0.0)
        .map(|r| {
            let stars = "*".repeat((r / 2.0).ceil() as usize);
            let stars_trunc: String = stars.chars().take(5).collect();
            format!("{} {:.1}", stars_trunc, r)
        })
        .unwrap_or_else(|| "—".to_string());

    lines.push(Line::from(vec![
        Span::styled(
            format!("{:<half$}", "Favorite"),
            Style::default().fg(label_color),
        ),
        Span::styled("Rating", Style::default().fg(label_color)),
    ]));
    lines.push(Line::from(vec![
        if is_fav {
            Span::styled(
                format!("{:<half$}", fav_str),
                Style::default()
                    .fg(MATRIX_GREEN)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled(format!("{:<half$}", fav_str), dim_style)
        },
        Span::styled(&rating_str, Style::default().fg(Color::Rgb(255, 200, 80))),
    ]));
    lines.push(Line::from(Span::styled("─".repeat(w), dim_style)));

    // Added + Account row
    let added_str = s
        .added
        .as_ref()
        .filter(|a| !a.is_empty())
        .map(|a| {
            if let Ok(ts) = a.parse::<i64>() {
                chrono::DateTime::from_timestamp(ts, 0)
                    .map(|dt| dt.format("%Y-%m-%d").to_string())
                    .unwrap_or_else(|| a.clone())
            } else {
                a.clone()
            }
        })
        .unwrap_or_else(|| "—".to_string());
    let acct_str = s
        .account_name
        .as_ref()
        .filter(|a| !a.is_empty())
        .cloned()
        .unwrap_or_else(|| "—".to_string());

    lines.push(Line::from(vec![
        Span::styled(
            format!("{:<half$}", "Added"),
            Style::default().fg(label_color),
        ),
        Span::styled("Account", Style::default().fg(label_color)),
    ]));
    lines.push(Line::from(vec![
        Span::styled(format!("{:<half$}", added_str), value_style),
        Span::styled(&acct_str, value_style),
    ]));

    // Network health (if available)
    if let Some(lat) = s.latency_ms {
        lines.push(Line::from(Span::styled("─".repeat(w), dim_style)));
        let (label, color) = if lat < 200 {
            ("Excellent", MATRIX_GREEN)
        } else if lat < 600 {
            ("Good", Color::Rgb(255, 200, 80))
        } else {
            ("Poor", Color::Rgb(255, 100, 100))
        };
        lines.push(Line::from(Span::styled(
            "Network",
            Style::default().fg(label_color),
        )));
        lines.push(Line::from(Span::styled(
            format!("{} ({}ms)", label, lat),
            Style::default().fg(color),
        )));
    }

    // Truncate lines to fit area
    let max_lines = inner.height as usize;
    if lines.len() > max_lines {
        lines.truncate(max_lines);
    }

    f.render_widget(Paragraph::new(lines), inner);
}

pub fn render_stream_details_pane(f: &mut Frame, app: &mut App, area: Rect, border_color: Color) {
    let title = " match intelligence ";
    let inner_area = crate::ui::common::render_matrix_box(f, area, title, border_color);

    let selected_idx = app.selected_stream_index;
    let focused_stream = if app.current_screen == CurrentScreen::GlobalSearch {
        app.global_search_results.get(selected_idx)
    } else {
        app.streams.get(selected_idx)
    };

    let Some(s) = focused_stream else {
        let msg = Paragraph::new("No channel selected")
            .style(Style::default().fg(TEXT_DIM))
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(msg, inner_area);
        return;
    };

    let parsed = if let Some(ref cached) = s.cached_parsed {
        cached.as_ref().clone()
    } else {
        parse_stream(&s.name, app.session.provider_timezone.as_deref())
    };

    let score_data = app.get_score_for_stream(&parsed.display_name);
    let event = parsed.sports_event.clone();

    // No match data — show compact channel info instead of hiding the panel
    if event.is_none() && score_data.is_none() {
        let user_tz_str = app.config.get_user_timezone();
        let user_tz: Tz = user_tz_str.parse().unwrap_or(chrono_tz::UTC);
        let mut lines = vec![Line::from(Span::styled(
            &parsed.display_name,
            Style::default()
                .fg(TEXT_PRIMARY)
                .add_modifier(Modifier::BOLD),
        ))];
        if let Some(st) = parsed.start_time {
            lines.push(Line::from(vec![
                Span::styled("⏰ ", Style::default().fg(MATRIX_GREEN)),
                Span::styled(
                    format_relative_time(st, &user_tz),
                    Style::default().fg(MATRIX_GREEN),
                ),
            ]));
        } else {
            lines.push(Line::from(Span::styled(
                "No live match data",
                Style::default().fg(TEXT_DIM),
            )));
        }
        lines.push(Line::from(vec![
            Span::styled(
                "enter",
                Style::default()
                    .fg(MATRIX_GREEN)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" to watch", Style::default().fg(TEXT_SECONDARY)),
        ]));
        f.render_widget(Paragraph::new(lines), inner_area);
        return;
    }

    let (team1, team2) = if let Some(ref ev) = event {
        (ev.team1.clone(), ev.team2.clone())
    } else if let Some(sd) = score_data {
        (sd.home_team.clone(), sd.away_team.clone())
    } else {
        return;
    };

    let sub_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(inner_area);

    // --- LEFT SIDE ---
    let mut left = Vec::new();

    let (h_score, a_score, _h_abbr, _a_abbr) = if let Some(score) = score_data {
        (
            score.home_score.as_str(),
            score.away_score.as_str(),
            score.home_abbr.as_str(),
            score.away_abbr.as_str(),
        )
    } else {
        ("-", "-", "", "")
    };

    let h_num: i32 = h_score.parse().unwrap_or(0);
    let a_num: i32 = a_score.parse().unwrap_or(0);

    let is_game_active = score_data
        .map(|s| s.status_state == "in" || s.status_state == "post")
        .unwrap_or(false);
    let has_scoring = h_num > 0 || a_num > 0;

    if let Some(score) = score_data {
        if score.status_state == "in" {
            let display_clock = if !score.display_clock.is_empty() && score.display_clock != "00:00"
            {
                format!("{} — {}", score.display_clock, score.status_detail)
            } else {
                score.status_detail.clone()
            };

            left.push(Line::from(vec![
                Span::styled("⏱ ", Style::default().fg(Color::Rgb(255, 100, 100))),
                Span::styled(
                    display_clock,
                    Style::default()
                        .fg(Color::Rgb(255, 150, 150))
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
        } else if score.status_state == "post" {
            left.push(Line::from(vec![Span::styled(
                "✓ final",
                Style::default().fg(TEXT_SECONDARY),
            )]));
        }
    }

    let mut team1_spans = vec![Span::styled(
        &team1,
        Style::default()
            .fg(crate::sports::get_team_color_with_fallback(&team1, true))
            .add_modifier(Modifier::BOLD),
    )];
    if has_scoring || is_game_active {
        team1_spans.push(Span::styled(
            format!("  {} ", h_score),
            Style::default()
                .fg(Color::Rgb(255, 200, 80))
                .add_modifier(Modifier::BOLD),
        ));
    }
    left.push(Line::from(team1_spans));

    let mut team2_spans = vec![Span::styled(
        &team2,
        Style::default()
            .fg(crate::sports::get_team_color_with_fallback(&team2, false))
            .add_modifier(Modifier::BOLD),
    )];
    if has_scoring || is_game_active {
        team2_spans.push(Span::styled(
            format!("  {} ", a_score),
            Style::default()
                .fg(Color::Rgb(255, 200, 80))
                .add_modifier(Modifier::BOLD),
        ));
    }
    left.push(Line::from(team2_spans));

    f.render_widget(Paragraph::new(left), sub_chunks[0]);

    // --- RIGHT SIDE ---
    let mut right = Vec::new();

    if let Some(score) = score_data {
        if let (Some(hwp), Some(awp)) = (score.home_win_pct, score.away_win_pct) {
            right.push(Line::from(vec![
                Span::styled("odds ", Style::default().fg(TEXT_SECONDARY)),
                Span::styled(
                    format!("{} ", score.home_abbr),
                    Style::default().fg(TEXT_PRIMARY),
                ),
                Span::styled(
                    format!("{:.0}%", hwp * 100.0),
                    Style::default().fg(if hwp > 0.5 {
                        MATRIX_GREEN
                    } else {
                        TEXT_SECONDARY
                    }),
                ),
                Span::styled(" · ", Style::default().fg(TEXT_DIM)),
                Span::styled(
                    format!("{} ", score.away_abbr),
                    Style::default().fg(TEXT_PRIMARY),
                ),
                Span::styled(
                    format!("{:.0}%", awp * 100.0),
                    Style::default().fg(if awp > 0.5 {
                        MATRIX_GREEN
                    } else {
                        TEXT_SECONDARY
                    }),
                ),
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
                let truncated = if hl.len() > 45 {
                    format!("{}...", &hl[..42])
                } else {
                    hl.clone()
                };
                right.push(Line::from(vec![
                    Span::styled("recap ", Style::default().fg(TEXT_SECONDARY)),
                    Span::styled(truncated, Style::default().fg(TEXT_PRIMARY)),
                ]));
            }
        }

        if score.status_state == "in" {
            if let Some(lp) = &score.last_play {
                let truncated = if lp.len() > 35 {
                    format!("{}...", &lp[..32])
                } else {
                    lp.clone()
                };
                right.push(Line::from(vec![
                    Span::styled("▶ ", Style::default().fg(MATRIX_GREEN)),
                    Span::styled(truncated, Style::default().fg(TEXT_PRIMARY)),
                ]));
            }
        }

        if !score.broadcasts.is_empty() {
            let channels = score
                .broadcasts
                .iter()
                .take(2)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ");
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
        Span::styled(
            "enter",
            Style::default()
                .fg(MATRIX_GREEN)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" to watch", Style::default().fg(TEXT_SECONDARY)),
    ]));
    f.render_widget(Paragraph::new(right), sub_chunks[1]);
}
