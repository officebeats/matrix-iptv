use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, BorderType, Borders, List, ListItem},
    Frame,
};
use crate::app::{App, Pane};
use crate::ui::colors::{MATRIX_GREEN, DARK_GREEN, BRIGHT_GREEN};
use crate::ui::utils::calculate_three_column_split;
use crate::parser::{parse_category, parse_stream};
use crate::ui::common::stylize_channel_name;

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
    
    // Save areas for mouse interaction if needed
    app.area_categories = chunks[0];
    app.area_streams = chunks[1];

    let border_color = MATRIX_GREEN;

    render_series_categories_pane(f, app, chunks[0], border_color);
    render_series_streams_pane(f, app, chunks[1], border_color);
    render_series_episodes_pane(f, app, chunks[2], border_color);
}

fn render_series_categories_pane(f: &mut Frame, app: &mut App, area: Rect, border_color: Color) {
    let visible_height = area.height.saturating_sub(2) as usize;
    let total = app.series_categories.len();
    let selected = app.selected_series_category_index;

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

    let items: Vec<ListItem> = app.series_categories.iter().enumerate()
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

    let title = format!(" // CATEGORIES ({}) ", app.series_categories.len());
    let is_active = app.active_pane == Pane::Categories;
    let block_border = if is_active { BRIGHT_GREEN } else { border_color };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(block_border)).border_type(BorderType::Double).title(ratatui::text::Span::styled(title, Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD))))
        .highlight_style(Style::default().bg(MATRIX_GREEN).fg(Color::Black).add_modifier(Modifier::BOLD))
        .highlight_symbol(" » ");

    let mut adjusted_state = app.series_category_list_state.clone();
    if adjusted_start > 0 {
        adjusted_state.select(Some(selected - adjusted_start));
    }
    f.render_stateful_widget(list, area, &mut adjusted_state);
}

fn render_series_streams_pane(f: &mut Frame, app: &mut App, area: Rect, border_color: Color) {
    let visible_height = area.height.saturating_sub(2) as usize;
    let total = app.series_streams.len();
    let selected = app.selected_series_stream_index;

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

    let items: Vec<ListItem> = app.series_streams.iter().enumerate()
        .skip(adjusted_start)
        .take(end - adjusted_start)
        .map(|(_, s)| {
            let parsed = parse_stream(&s.name, app.provider_timezone.as_deref());
            let mut spans = vec![
                ratatui::text::Span::styled("📺 ", Style::default().fg(MATRIX_GREEN)),
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

    let title = format!(" // SERIES ({}) ", app.series_streams.len());
    let is_active = app.active_pane == Pane::Streams;
    let block_border = if is_active { BRIGHT_GREEN } else { border_color };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(block_border)).border_type(BorderType::Double).title(ratatui::text::Span::styled(title, Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD))))
        .highlight_style(Style::default().bg(MATRIX_GREEN).fg(Color::Black).add_modifier(Modifier::BOLD))
        .highlight_symbol(" » ");

    let mut adjusted_state = app.series_stream_list_state.clone();
    if adjusted_start > 0 {
        adjusted_state.select(Some(selected - adjusted_start));
    }
    f.render_stateful_widget(list, area, &mut adjusted_state);
}

fn render_series_episodes_pane(f: &mut Frame, app: &mut App, area: Rect, border_color: Color) {
    let visible_height = area.height.saturating_sub(2) as usize;
    let total = app.series_episodes.len();
    let selected = app.selected_series_episode_index;

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

    let items: Vec<ListItem> = app.series_episodes.iter().enumerate()
        .skip(adjusted_start)
        .take(end - adjusted_start)
        .map(|(_, ep)| {
            let season_color = match ep.season % 5 {
                0 => Color::Cyan, 1 => Color::LightBlue, 2 => Color::LightMagenta, 3 => Color::LightYellow, _ => Color::LightGreen,
            };
            let title = ep.title.as_deref().unwrap_or("Untitled");
            let spans = vec![
                ratatui::text::Span::styled("▶ ", Style::default().fg(MATRIX_GREEN)),
                ratatui::text::Span::styled(format!("S{:02}E{:02}", ep.season, ep.episode_num), Style::default().fg(season_color).add_modifier(Modifier::BOLD)),
                ratatui::text::Span::styled(" │ ", Style::default().fg(DARK_GREEN)),
                ratatui::text::Span::styled(title.to_string(), Style::default().fg(MATRIX_GREEN)),
            ];
            ListItem::new(Line::from(spans))
        }).collect();

    let title = format!(" // EPISODES ({}) ", app.series_episodes.len());
    let is_active = app.active_pane == Pane::Episodes;
    let block_border = if is_active { BRIGHT_GREEN } else { border_color };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(block_border)).border_type(BorderType::Double).title(ratatui::text::Span::styled(title, Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD))))
        .highlight_style(Style::default().bg(MATRIX_GREEN).fg(Color::Black).add_modifier(Modifier::BOLD))
        .highlight_symbol(" » ");

    let mut adjusted_state = app.series_episode_list_state.clone();
    if adjusted_start > 0 {
        adjusted_state.select(Some(selected - adjusted_start));
    }
    f.render_stateful_widget(list, area, &mut adjusted_state);
}
