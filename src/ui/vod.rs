use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, List, ListItem},
    Frame,
};
use crate::app::{App, Pane};
use crate::ui::colors::{MATRIX_GREEN, BRIGHT_GREEN};

pub fn render_vod_categories_pane(f: &mut Frame, app: &mut App, area: Rect, border_color: Color) {
    let title = format!(" // MOVIES ({}) ", app.vod_categories.len());
    let is_active = app.active_pane == Pane::Categories;
    let block_border = if is_active { BRIGHT_GREEN } else { border_color };

    let items: Vec<ListItem> = app.vod_categories.iter().map(|c| {
        ListItem::new(format!("  📁 {}", c.category_name))
    }).collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(block_border)).border_type(BorderType::Double).title(title))
        .highlight_style(Style::default().bg(MATRIX_GREEN).fg(Color::Black).add_modifier(Modifier::BOLD))
        .highlight_symbol(" » ");

    f.render_stateful_widget(list, area, &mut app.vod_category_list_state);
}

pub fn render_vod_streams_pane(f: &mut Frame, app: &mut App, area: Rect, border_color: Color) {
    let title = format!(" // RESULTS ({}) ", app.vod_streams.len());
    let is_active = app.active_pane == Pane::Streams;
    let block_border = if is_active { BRIGHT_GREEN } else { border_color };

    let items: Vec<ListItem> = app.vod_streams.iter().map(|s| {
        ListItem::new(format!("  🎬 {}", s.name))
    }).collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(block_border)).border_type(BorderType::Double).title(title))
        .highlight_style(Style::default().bg(MATRIX_GREEN).fg(Color::Black).add_modifier(Modifier::BOLD))
        .highlight_symbol(" » ");

    f.render_stateful_widget(list, area, &mut app.vod_stream_list_state);
}
