use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, BorderType, Borders, List, ListItem},
    Frame,
};
use crate::app::{App, Pane};
use crate::ui::colors::{MATRIX_GREEN, SOFT_GREEN, DARK_GREEN, HIGHLIGHT_BG, TEXT_PRIMARY, TEXT_SECONDARY, TEXT_DIM};
use crate::parser::{parse_category, parse_stream};
use crate::ui::common::stylize_channel_name;
use crate::ui::utils::calculate_vod_three_column_split;

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

    render_vod_categories_pane(f, app, chunks[0]);
    render_vod_streams_pane(f, app, chunks[1]);
    render_vod_details_pane(f, app, chunks[2]);
}

pub fn render_vod_categories_pane(f: &mut Frame, app: &mut App, area: Rect) {
    let visible_height = area.height.saturating_sub(2) as usize;
    let total = app.vod_categories.len();
    let selected = app.selected_vod_category_index;

    let half_window = visible_height / 2;
    let start = if selected > half_window { selected - half_window } else { 0 };
    let end = (start + visible_height + half_window).min(total);
    let adjusted_start = if end == total && end > visible_height + half_window {
        end.saturating_sub(visible_height + half_window)
    } else { start };

    let items: Vec<ListItem> = app.vod_categories.iter().enumerate()
        .skip(adjusted_start)
        .take(end - adjusted_start)
        .map(|(_, c)| {
            let parsed = parse_category(&c.category_name);
            let mut spans = vec![
                ratatui::text::Span::styled("◆ ", Style::default().fg(SOFT_GREEN)),
            ];
            
            let (styled_name, _) = stylize_channel_name(
                &parsed.display_name, parsed.is_vip, false,
                parsed.quality, parsed.content_type, None,
                Style::default().fg(MATRIX_GREEN),
            );
            spans.extend(styled_name);
            ListItem::new(Line::from(spans))
        }).collect();

    let title = format!(" movies ({}) ", app.vod_categories.len());
    let is_active = app.active_pane == Pane::Categories;
    let border_color = if is_active { SOFT_GREEN } else { DARK_GREEN };

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .border_type(BorderType::Rounded)
            .title(ratatui::text::Span::styled(&title, Style::default().fg(border_color).add_modifier(Modifier::BOLD))))
        .highlight_style(Style::default().bg(HIGHLIGHT_BG).fg(MATRIX_GREEN).add_modifier(Modifier::BOLD))
        .highlight_symbol(" ▎");

    let mut adjusted_state = app.vod_category_list_state.clone();
    if adjusted_start > 0 { adjusted_state.select(Some(selected - adjusted_start)); }
    f.render_stateful_widget(list, area, &mut adjusted_state);
}

pub fn render_vod_streams_pane(f: &mut Frame, app: &mut App, area: Rect) {
    let visible_height = area.height.saturating_sub(2) as usize;
    let total = app.vod_streams.len();
    let selected = app.selected_vod_stream_index;

    let half_window = visible_height / 2;
    let start = if selected > half_window { selected - half_window } else { 0 };
    let end = (start + visible_height + half_window).min(total);
    let adjusted_start = if end == total && end > visible_height + half_window {
        end.saturating_sub(visible_height + half_window)
    } else { start };

    let items: Vec<ListItem> = app.vod_streams.iter().enumerate()
        .skip(adjusted_start)
        .take(end - adjusted_start)
        .map(|(_, s)| {
            let parsed = parse_stream(&s.name, app.provider_timezone.as_deref());
            let mut spans = vec![
                ratatui::text::Span::styled("◆ ", Style::default().fg(SOFT_GREEN)),
            ];
            
            let mut name = parsed.display_name.clone();
            let re_year = regex::Regex::new(r"[\(\[](19|20)\d{2}[\)\]]").unwrap();
            if let Some(mat) = re_year.find(&s.name) {
                let year_clean = mat.as_str().replace('[', "(").replace(']', ")");
                if !name.contains(&year_clean) {
                    name.push_str(" ");
                    name.push_str(&year_clean);
                }
            }

            let (styled_name, _) = stylize_channel_name(
                &name, false, false, parsed.quality, None, None,
                Style::default().fg(TEXT_PRIMARY),
            );
            spans.extend(styled_name);

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
                        format!(" {:.1}", rating_f),
                        Style::default().fg(rating_color),
                    ));
                }
            }

            ListItem::new(Line::from(spans))
        }).collect();

    let title = format!(" results ({}) ", app.vod_streams.len());
    let is_active = app.active_pane == Pane::Streams;
    let border_color = if is_active { SOFT_GREEN } else { DARK_GREEN };

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .border_type(BorderType::Rounded)
            .title(ratatui::text::Span::styled(&title, Style::default().fg(border_color).add_modifier(Modifier::BOLD))))
        .highlight_style(Style::default().bg(HIGHLIGHT_BG).fg(MATRIX_GREEN).add_modifier(Modifier::BOLD))
        .highlight_symbol(" ▎");

    let mut adjusted_state = app.vod_stream_list_state.clone();
    if adjusted_start > 0 { adjusted_state.select(Some(selected - adjusted_start)); }
    f.render_stateful_widget(list, area, &mut adjusted_state);
}

pub fn render_vod_details_pane(f: &mut Frame, app: &mut App, area: Rect) {
    use ratatui::widgets::Paragraph;
    use ratatui::widgets::Wrap;

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DARK_GREEN))
        .border_type(BorderType::Rounded)
        .title(ratatui::text::Span::styled(" details ", Style::default().fg(SOFT_GREEN).add_modifier(Modifier::BOLD)));

    let mut details_text = Vec::new();

    if let Some(vod_info) = &app.current_vod_info {
        if let Some(info) = &vod_info.info {
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
                    ratatui::text::Span::styled(title.clone(), Style::default().fg(TEXT_PRIMARY).add_modifier(Modifier::BOLD)),
                ]));
                details_text.push(Line::from(""));
            }

            if let Some(rating) = info.get("rating").and_then(|v| {
                match v {
                    serde_json::Value::String(s) => Some(s.clone()),
                    serde_json::Value::Number(n) => Some(n.to_string()),
                    _ => None,
                }
            }) {
                details_text.push(Line::from(vec![
                    ratatui::text::Span::styled("rating  ", Style::default().fg(TEXT_SECONDARY)),
                    ratatui::text::Span::styled(format!("{} / 10", rating), Style::default().fg(MATRIX_GREEN)),
                ]));
            }

            if let Some(runtime) = info.get("runtime").and_then(|v| {
                match v {
                    serde_json::Value::String(s) => Some(s.clone()),
                    serde_json::Value::Number(n) => Some(n.to_string()),
                    _ => None,
                }
            }) {
                details_text.push(Line::from(vec![
                    ratatui::text::Span::styled("runtime ", Style::default().fg(TEXT_SECONDARY)),
                    ratatui::text::Span::styled(format!("{} min", runtime), Style::default().fg(TEXT_PRIMARY)),
                ]));
            }

            if let Some(releasedate) = info.get("releasedate").and_then(|v| v.as_str()) {
                details_text.push(Line::from(vec![
                    ratatui::text::Span::styled("released ", Style::default().fg(TEXT_SECONDARY)),
                    ratatui::text::Span::styled(releasedate, Style::default().fg(TEXT_PRIMARY)),
                ]));
            }

            if let Some(genre) = info.get("genre").and_then(|v| v.as_str()) {
                details_text.push(Line::from(vec![
                    ratatui::text::Span::styled("genre   ", Style::default().fg(TEXT_SECONDARY)),
                    ratatui::text::Span::styled(genre, Style::default().fg(MATRIX_GREEN)),
                ]));
            }

            details_text.push(Line::from(""));

            if let Some(plot) = info.get("plot").and_then(|v| v.as_str()).or_else(|| info.get("description").and_then(|v| v.as_str())) {
                details_text.push(Line::from(ratatui::text::Span::styled(plot, Style::default().fg(TEXT_SECONDARY))));
                details_text.push(Line::from(""));
            }

            if let Some(cast) = info.get("cast").and_then(|v| v.as_str()).or_else(|| info.get("actors").and_then(|v| v.as_str())) {
                details_text.push(Line::from(vec![
                    ratatui::text::Span::styled("cast ", Style::default().fg(TEXT_SECONDARY)),
                    ratatui::text::Span::styled(cast, Style::default().fg(TEXT_PRIMARY)),
                ]));
            }
        }
    } else {
        details_text.push(Line::from(ratatui::text::Span::styled(
            "Select a movie to view details...",
            Style::default().fg(TEXT_DIM),
        )));
    }

    let paragraph = Paragraph::new(details_text)
        .block(block)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}
