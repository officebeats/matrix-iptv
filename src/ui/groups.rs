use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem, Paragraph},
    Frame,
};
use crate::app::App;
use crate::ui::colors::{MATRIX_GREEN, SOFT_GREEN, HIGHLIGHT_BG, TEXT_PRIMARY, TEXT_SECONDARY, TEXT_DIM};

/// Render the Group Management screen
pub fn render_group_management(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(10),
            Constraint::Length(2),
        ])
        .split(area);

    let title = Paragraph::new(Line::from(vec![
        Span::styled("  groups", Style::default().fg(TEXT_PRIMARY).add_modifier(Modifier::BOLD)),
        Span::styled(" · custom channel groups", Style::default().fg(TEXT_SECONDARY)),
    ]));
    f.render_widget(title, chunks[0]);

    let groups = &app.config.favorites.groups;
    let items: Vec<ListItem> = if groups.is_empty() {
        vec![ListItem::new(Line::from(vec![
            Span::styled("  No groups yet. Press ", Style::default().fg(TEXT_SECONDARY)),
            Span::styled("n", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
            Span::styled(" to create one.", Style::default().fg(TEXT_SECONDARY)),
        ]))]
    } else {
        groups.iter().map(|g| {
            let icon = g.icon.as_deref().unwrap_or("");
            let count = g.stream_ids.len();
            let prefix = if icon.is_empty() { "  ".to_string() } else { format!("  {} ", icon) };
            ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(SOFT_GREEN)),
                Span::styled(&g.name, Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
                Span::styled(format!("  {} channels", count), Style::default().fg(TEXT_DIM)),
            ]))
        }).collect()
    };

    let inner_area = crate::ui::common::render_matrix_box(f, chunks[1], &format!("groups ({})", groups.len()), SOFT_GREEN);

    let list = List::new(items)
        .highlight_style(Style::default().bg(HIGHLIGHT_BG).fg(MATRIX_GREEN).add_modifier(Modifier::BOLD))
        .highlight_symbol(" ▎");

    f.render_stateful_widget(list, inner_area, &mut app.group_list_state);

    let key_style = Style::default().fg(MATRIX_GREEN);
    let label_style = Style::default().fg(TEXT_SECONDARY);
    let help = Paragraph::new(Line::from(vec![
        Span::styled("n", key_style), Span::styled(" new  ", label_style),
        Span::styled("d", key_style), Span::styled(" delete  ", label_style),
        Span::styled("enter", key_style), Span::styled(" view  ", label_style),
        Span::styled("esc", key_style), Span::styled(" back", label_style),
    ])).alignment(Alignment::Center);
    f.render_widget(help, chunks[2]);
}

/// Render the Group Picker popup
pub fn render_group_picker(f: &mut Frame, app: &mut App, area: Rect) {
    let popup_width = 40.min(area.width.saturating_sub(4));
    let popup_height = (app.config.favorites.groups.len() as u16 + 5)
        .min(15)
        .min(area.height.saturating_sub(2));
    
    let popup_x = area.x + (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = area.y + (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    let clear = Block::default().style(Style::default().bg(Color::Black));
    f.render_widget(clear, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .split(popup_area);

    let title = Paragraph::new(Line::from(vec![
        Span::styled("add to group", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
    ]))
    .alignment(Alignment::Center)
    ;
    f.render_widget(title, chunks[0]);

    let groups = &app.config.favorites.groups;
    let mut items: Vec<ListItem> = groups.iter().map(|g| {
        let icon = g.icon.as_deref().unwrap_or("");
        let prefix = if icon.is_empty() { "  ".to_string() } else { format!(" {} ", icon) };
        ListItem::new(Line::from(vec![
            Span::styled(prefix, Style::default().fg(SOFT_GREEN)),
            Span::styled(&g.name, Style::default().fg(MATRIX_GREEN)),
        ]))
    }).collect();

    items.push(ListItem::new(Line::from(vec![
        Span::styled(" + ", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
        Span::styled("Create New Group...", Style::default().fg(TEXT_PRIMARY)),
    ])));

    let list = List::new(items)
        .highlight_style(Style::default().bg(HIGHLIGHT_BG).fg(MATRIX_GREEN).add_modifier(Modifier::BOLD))
        .highlight_symbol(" ▎");

    f.render_stateful_widget(list, chunks[1], &mut app.group_list_state);

    let help = Paragraph::new(Line::from(vec![
        Span::styled("enter", Style::default().fg(MATRIX_GREEN)),
        Span::styled(" add · ", Style::default().fg(TEXT_SECONDARY)),
        Span::styled("esc", Style::default().fg(MATRIX_GREEN)),
        Span::styled(" cancel", Style::default().fg(TEXT_SECONDARY)),
    ])).alignment(Alignment::Center);
    f.render_widget(help, chunks[2]);
}
