use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, BorderType, Borders, List, ListItem},
    Frame,
};
use crate::app::{App, Pane};
use crate::ui::colors::{MATRIX_GREEN, BRIGHT_GREEN};
use crate::parser::{parse_category, parse_stream};
use crate::ui::common::stylize_channel_name;
use crate::ui::utils::calculate_vod_three_column_split;
use crate::ui::colors::DARK_GREEN;

pub fn render_vod_view(f: &mut Frame, app: &mut App, area: Rect) {
    let (cat_width, stream_width, details_width) = calculate_vod_three_column_split(
        &app.vod_categories,
        &app.vod_streams,
        area.width,
    );

    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            ratatui::layout::Constraint::Length(cat_width),
            ratatui::layout::Constraint::Length(stream_width),
            ratatui::layout::Constraint::Min(details_width),
        ])
        .split(area);

    app.area_categories = chunks[0];
    app.area_streams = chunks[1];

    let border_color = MATRIX_GREEN;

    render_vod_categories_pane(f, app, chunks[0], border_color);
    render_vod_streams_pane(f, app, chunks[1], border_color);
    render_vod_details_pane(f, app, chunks[2], border_color);
}

pub fn render_vod_categories_pane(f: &mut Frame, app: &mut App, area: Rect, border_color: Color) {
    let visible_height = area.height.saturating_sub(2) as usize;
    let total = app.vod_categories.len();
    let selected = app.selected_vod_category_index;

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

    let items: Vec<ListItem> = app.vod_categories.iter().enumerate()
        .skip(adjusted_start)
        .take(end - adjusted_start)
        .map(|(_, c)| {
            let parsed = parse_category(&c.category_name);
            let mut spans = vec![
                ratatui::text::Span::styled("📁 ", Style::default().fg(Color::White)),
            ];
            
            let (styled_name, _) = stylize_channel_name(
                &parsed.display_name,
                parsed.is_vip,
                false,
                parsed.quality,
                parsed.content_type,
                None,
                Style::default().fg(MATRIX_GREEN),
            );
            spans.extend(styled_name);
            
            ListItem::new(Line::from(spans))
        }).collect();

    let title = format!(" // MOVIES ({}) ", app.vod_categories.len());
    let is_active = app.active_pane == Pane::Categories;
    let block_border = if is_active { BRIGHT_GREEN } else { border_color };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(block_border)).border_type(BorderType::Double).title(title))
        .highlight_style(Style::default().bg(MATRIX_GREEN).fg(Color::Black).add_modifier(Modifier::BOLD))
        .highlight_symbol(" » ");

    let mut adjusted_state = app.vod_category_list_state.clone();
    if adjusted_start > 0 {
        adjusted_state.select(Some(selected - adjusted_start));
    }
    f.render_stateful_widget(list, area, &mut adjusted_state);
}

pub fn render_vod_streams_pane(f: &mut Frame, app: &mut App, area: Rect, border_color: Color) {
    let visible_height = area.height.saturating_sub(2) as usize;
    let total = app.vod_streams.len();
    let selected = app.selected_vod_stream_index;

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

    let items: Vec<ListItem> = app.vod_streams.iter().enumerate()
        .skip(adjusted_start)
        .take(end - adjusted_start)
        .map(|(_, s)| {
            let parsed = parse_stream(&s.name, app.provider_timezone.as_deref());
            let mut spans = vec![
                ratatui::text::Span::styled("🎬 ", Style::default().fg(MATRIX_GREEN)),
            ];
            
            let mut name = parsed.display_name.clone();
            // Force Year Restoration from Raw Source (ensure (YYYY) format)
            let re_year = regex::Regex::new(r"[\(\[](19|20)\d{2}[\)\]]").unwrap();
            if let Some(mat) = re_year.find(&s.name) {
                let year_clean = mat.as_str().replace('[', "(").replace(']', ")");
                if !name.contains(&year_clean) {
                    name.push_str(" ");
                    name.push_str(&year_clean);
                }
            }

            let (styled_name, _) = stylize_channel_name(
                &name,
                false,
                false,
                parsed.quality,
                None,
                None,
                Style::default().fg(Color::White),
            );
            spans.extend(styled_name);

            // Add rating in brackets (color-coded, 1 decimal)
            if let Some(rating_val) = &s.rating {
                let rating_str = match rating_val {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Number(n) => n.to_string(),
                    _ => String::new(),
                };
                let rating_f = rating_str.parse::<f32>().unwrap_or(0.0);
                if rating_f > 0.0 {
                    let rating_color = crate::ui::utils::get_rating_color(&rating_str);
                    spans.push(ratatui::text::Span::styled(
                        format!(" [{:.1}]", rating_f),
                        Style::default().fg(rating_color).add_modifier(Modifier::BOLD),
                    ));
                }
            }

            ListItem::new(Line::from(spans))
        }).collect();

    let title = format!(" // RESULTS ({}) ", app.vod_streams.len());
    let is_active = app.active_pane == Pane::Streams;
    let block_border = if is_active { BRIGHT_GREEN } else { border_color };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(block_border)).border_type(BorderType::Double).title(title))
        .highlight_style(Style::default().bg(MATRIX_GREEN).fg(Color::Black).add_modifier(Modifier::BOLD))
        .highlight_symbol(" » ");

    let mut adjusted_state = app.vod_stream_list_state.clone();
    if adjusted_start > 0 {
        adjusted_state.select(Some(selected - adjusted_start));
    }

    f.render_stateful_widget(list, area, &mut adjusted_state);
}

pub fn render_vod_details_pane(f: &mut Frame, app: &mut App, area: Rect, border_color: Color) {
    use ratatui::widgets::Paragraph;
    use ratatui::widgets::Wrap;

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .border_type(BorderType::Double)
        .title(" // MOVIE DETAILS ");

    let mut details_text = Vec::new();

    if let Some(vod_info) = &app.current_vod_info {
        if let Some(info) = &vod_info.info {
            // Title (Restore year from stream if missing)
            let mut title = info.get("name").and_then(|v| v.as_str()).map(|s| s.to_string())
                .or_else(|| vod_info.movie_data.as_ref().and_then(|m| m.name.clone()))
                .unwrap_or_default();
            
            if !title.contains('(') && !title.contains('[') {
                if let Some(stream) = app.vod_streams.get(app.selected_vod_stream_index) {
                    let re_year = regex::Regex::new(r"[\(\[](19|20)\d{2}[\)\]]").unwrap();
                    if let Some(mat) = re_year.find(&stream.name) {
                        title.push_str(" ");
                        title.push_str(mat.as_str());
                    }
                }
            }

            if !title.is_empty() {
                details_text.push(Line::from(vec![
                    ratatui::text::Span::styled("TITLE: ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                    ratatui::text::Span::styled(title, Style::default().fg(Color::Cyan)),
                ]));
                details_text.push(Line::from(""));
            }

            // Rating
            if let Some(rating) = info.get("rating").and_then(|v| {
                match v {
                    serde_json::Value::String(s) => Some(s.clone()),
                    serde_json::Value::Number(n) => Some(n.to_string()),
                    _ => None,
                }
            }) {
                details_text.push(Line::from(vec![
                    ratatui::text::Span::styled("RATING: ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                    ratatui::text::Span::styled(format!("⭐ {} / 10", rating), Style::default().fg(Color::Cyan)),
                ]));
            }

            // Runtime
            if let Some(runtime) = info.get("runtime").and_then(|v| {
                match v {
                    serde_json::Value::String(s) => Some(s.clone()),
                    serde_json::Value::Number(n) => Some(n.to_string()),
                    _ => None,
                }
            }) {
                details_text.push(Line::from(vec![
                    ratatui::text::Span::styled("RUNTIME: ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                    ratatui::text::Span::styled(format!("{} min", runtime), Style::default().fg(Color::White)),
                ]));
            }

            // Release Date
            if let Some(releasedate) = info.get("releasedate").and_then(|v| v.as_str()) {
                details_text.push(Line::from(vec![
                    ratatui::text::Span::styled("RELEASED: ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                    ratatui::text::Span::styled(releasedate, Style::default().fg(Color::LightYellow)),
                ]));
            }

            // Genres
            if let Some(genre) = info.get("genre").and_then(|v| v.as_str()) {
                details_text.push(Line::from(vec![
                    ratatui::text::Span::styled("GENRE: ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                    ratatui::text::Span::styled(genre, Style::default().fg(MATRIX_GREEN)),
                ]));
            }

            details_text.push(Line::from(""));

            // Description / Synopsis
            if let Some(plot) = info.get("plot").and_then(|v| v.as_str()).or_else(|| info.get("description").and_then(|v| v.as_str())) {
                details_text.push(Line::from(ratatui::text::Span::styled("SYNOPSIS:", Style::default().fg(Color::White).add_modifier(Modifier::BOLD))));
                details_text.push(Line::from(ratatui::text::Span::styled(plot, Style::default().fg(MATRIX_GREEN))));
                details_text.push(Line::from(""));
            }

            // Cast
            if let Some(cast) = info.get("cast").and_then(|v| v.as_str()).or_else(|| info.get("actors").and_then(|v| v.as_str())) {
                details_text.push(Line::from(ratatui::text::Span::styled("CAST:", Style::default().fg(Color::White).add_modifier(Modifier::BOLD))));
                details_text.push(Line::from(ratatui::text::Span::styled(cast, Style::default().fg(MATRIX_GREEN))));
            }
        }
    } else {
        details_text.push(Line::from(ratatui::text::Span::styled(
            "Select a movie to view details...",
            Style::default().fg(DARK_GREEN).add_modifier(Modifier::ITALIC),
        )));
    }

    let paragraph = Paragraph::new(details_text)
        .block(block)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}
