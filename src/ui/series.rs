use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem},
    Frame,
};
use crate::app::{App, Pane};
use crate::ui::colors::{MATRIX_GREEN, DARK_GREEN, BRIGHT_GREEN};
use crate::ui::utils::calculate_three_column_split;

pub fn render_series_view(f: &mut Frame, app: &mut App, area: Rect) {
    let (cat_width, series_width, episode_width) = calculate_three_column_split(
        &app.series_categories,
        &app.series_streams,
        &app.series_episodes,
        area.width,
    );
    
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            ratatui::layout::Constraint::Length(cat_width),
            ratatui::layout::Constraint::Length(series_width),
            ratatui::layout::Constraint::Min(episode_width),
        ])
        .split(area);

    app.area_categories = chunks[0];
    app.area_streams = chunks[1];

    let border_color = MATRIX_GREEN;

    render_series_categories_pane(f, app, chunks[0], border_color);
    render_series_streams_pane(f, app, chunks[1], border_color);
    render_series_episodes_pane(f, app, chunks[2], border_color);
}

fn render_series_categories_pane(f: &mut Frame, app: &mut App, area: Rect, border_color: Color) {
    let title = format!(" // CATEGORIES ({}) ", app.series_categories.len());
    let is_active = app.active_pane == Pane::Categories;
    let block_border = if is_active { BRIGHT_GREEN } else { border_color };

    let items: Vec<ListItem> = app.series_categories.iter().map(|c| {
        let base_color = if is_active { MATRIX_GREEN } else { DARK_GREEN };
        ListItem::new(Line::from(vec![
            Span::styled("📁 ", Style::default().fg(base_color)),
            Span::styled(c.category_name.clone(), Style::default().fg(base_color)),
        ]))
    }).collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(block_border)).border_type(BorderType::Double).title(Span::styled(title, Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD))))
        .highlight_style(Style::default().bg(MATRIX_GREEN).fg(Color::Black).add_modifier(Modifier::BOLD))
        .highlight_symbol(" » ");

    f.render_stateful_widget(list, area, &mut app.series_category_list_state);
}

fn render_series_streams_pane(f: &mut Frame, app: &mut App, area: Rect, border_color: Color) {
    let title = format!(" // SERIES ({}) ", app.series_streams.len());
    let is_active = app.active_pane == Pane::Streams;
    let block_border = if is_active { BRIGHT_GREEN } else { border_color };

    let items: Vec<ListItem> = app.series_streams.iter().map(|s| {
        let base_color = if is_active { MATRIX_GREEN } else { DARK_GREEN };
        let mut spans = vec![Span::styled("📺 ", Style::default().fg(base_color))];
        spans.push(Span::styled(s.name.clone(), Style::default().fg(base_color).add_modifier(Modifier::BOLD)));
        ListItem::new(Line::from(spans))
    }).collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(block_border)).border_type(BorderType::Double).title(Span::styled(title, Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD))))
        .highlight_style(Style::default().bg(MATRIX_GREEN).fg(Color::Black).add_modifier(Modifier::BOLD))
        .highlight_symbol(" » ");

    f.render_stateful_widget(list, area, &mut app.series_stream_list_state);
}

fn render_series_episodes_pane(f: &mut Frame, app: &mut App, area: Rect, border_color: Color) {
    let title = format!(" // EPISODES ({}) ", app.series_episodes.len());
    let is_active = app.active_pane == Pane::Episodes;
    let block_border = if is_active { BRIGHT_GREEN } else { border_color };

    let items: Vec<ListItem> = app.series_episodes.iter().map(|ep| {
        let base_color = if is_active { MATRIX_GREEN } else { DARK_GREEN };
        let season_color = match ep.season % 5 {
            0 => Color::Cyan, 1 => Color::LightBlue, 2 => Color::LightMagenta, 3 => Color::LightYellow, _ => Color::LightGreen,
        };
        let title = ep.title.as_deref().unwrap_or("Untitled");
        let spans = vec![
            Span::styled("▶ ", Style::default().fg(base_color)),
            Span::styled(format!("S{:02}E{:02}", ep.season, ep.episode_num), Style::default().fg(season_color).add_modifier(Modifier::BOLD)),
            Span::styled(" │ ", Style::default().fg(DARK_GREEN)),
            Span::styled(title.to_string(), Style::default().fg(base_color)),
        ];
        ListItem::new(Line::from(spans))
    }).collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(block_border)).border_type(BorderType::Double).title(Span::styled(title, Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD))))
        .highlight_style(Style::default().bg(MATRIX_GREEN).fg(Color::Black).add_modifier(Modifier::BOLD))
        .highlight_symbol(" » ");

    f.render_stateful_widget(list, area, &mut app.series_episode_list_state);
}
