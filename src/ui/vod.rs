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
            
            let (styled_name, _) = stylize_channel_name(
                &parsed.display_name,
                false,
                false,
                parsed.quality,
                None,
                None,
                Style::default().fg(MATRIX_GREEN),
            );
            spans.extend(styled_name);

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
