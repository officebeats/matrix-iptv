use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
    Frame,
};
use crate::app::App;
use crate::ui::colors::{MATRIX_GREEN, DARK_GREEN, BRIGHT_GREEN};

/// Render the Group Management screen (accessible via 'G' from categories)
pub fn render_group_management(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(10),    // Group list
            Constraint::Length(3),  // Help
        ])
        .split(area);

    // Title
    let title = Paragraph::new(Line::from(vec![
        Span::styled(" // ", Style::default().fg(DARK_GREEN)),
        Span::styled("GROUP_MANAGEMENT", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled(" - Custom Channel Groups", Style::default().fg(MATRIX_GREEN)),
    ]))
    .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(DARK_GREEN)));
    f.render_widget(title, chunks[0]);

    // Group List
    let groups = &app.config.favorites.groups;
    let items: Vec<ListItem> = if groups.is_empty() {
        vec![ListItem::new(Line::from(vec![
            Span::styled("  No groups yet. Press ", Style::default().fg(Color::DarkGray)),
            Span::styled("n", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
            Span::styled(" to create one.", Style::default().fg(Color::DarkGray)),
        ]))]
    } else {
        groups.iter().enumerate().map(|(i, g)| {
            let icon = g.icon.as_deref().unwrap_or("📁");
            let count = g.stream_ids.len();
            let mut spans = vec![
                Span::styled(format!(" {} ", icon), Style::default().fg(Color::White)),
                Span::styled(&g.name, Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
                Span::styled(format!(" ({} channels)", count), Style::default().fg(Color::DarkGray)),
            ];
            if i == app.selected_group_index {
                spans.insert(0, Span::styled(" » ", Style::default().fg(BRIGHT_GREEN)));
            } else {
                spans.insert(0, Span::raw("   "));
            }
            ListItem::new(Line::from(spans))
        }).collect()
    };

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(DARK_GREEN))
            .title(Span::styled(format!(" Groups ({}) ", groups.len()), Style::default().fg(Color::White))))
        .highlight_style(Style::default().bg(MATRIX_GREEN).fg(Color::Black).add_modifier(Modifier::BOLD));

    f.render_stateful_widget(list, chunks[1], &mut app.group_list_state);

    // Help bar
    let help = Paragraph::new(Line::from(vec![
        Span::styled("n", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
        Span::styled(":New ", Style::default().fg(Color::White)),
        Span::styled("│ ", Style::default().fg(Color::DarkGray)),
        Span::styled("d", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
        Span::styled(":Delete ", Style::default().fg(Color::White)),
        Span::styled("│ ", Style::default().fg(Color::DarkGray)),
        Span::styled("Enter", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
        Span::styled(":View ", Style::default().fg(Color::White)),
        Span::styled("│ ", Style::default().fg(Color::DarkGray)),
        Span::styled("Esc", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
        Span::styled(":Back", Style::default().fg(Color::White)),
    ])).alignment(Alignment::Center);
    f.render_widget(help, chunks[2]);
}

/// Render the Group Picker popup (when pressing 'g' on a stream)
pub fn render_group_picker(f: &mut Frame, app: &mut App, area: Rect) {
    // Center the popup
    let popup_width = 40.min(area.width.saturating_sub(4));
    let popup_height = (app.config.favorites.groups.len() as u16 + 5).min(15);
    
    let popup_x = (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = (area.height.saturating_sub(popup_height)) / 2;
    
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    // Clear background
    let clear = Block::default().style(Style::default().bg(Color::Black));
    f.render_widget(clear, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // Title
            Constraint::Min(3),     // Group list
            Constraint::Length(1),  // Help
        ])
        .split(popup_area);

    // Title
    let title = Paragraph::new(Line::from(vec![
        Span::styled("Add to Group", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
    ]))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(DARK_GREEN)));
    f.render_widget(title, chunks[0]);

    // Group options
    let groups = &app.config.favorites.groups;
    let mut items: Vec<ListItem> = groups.iter().map(|g| {
        let icon = g.icon.as_deref().unwrap_or("📁");
        ListItem::new(Line::from(vec![
            Span::styled(format!(" {} ", icon), Style::default().fg(Color::White)),
            Span::styled(&g.name, Style::default().fg(MATRIX_GREEN)),
        ]))
    }).collect();

    // Add "Create New Group" option at the end
    items.push(ListItem::new(Line::from(vec![
        Span::styled(" + ", Style::default().fg(BRIGHT_GREEN).add_modifier(Modifier::BOLD)),
        Span::styled("Create New Group...", Style::default().fg(BRIGHT_GREEN)),
    ])));

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(MATRIX_GREEN)))
        .highlight_style(Style::default().bg(MATRIX_GREEN).fg(Color::Black).add_modifier(Modifier::BOLD))
        .highlight_symbol(" » ");

    f.render_stateful_widget(list, chunks[1], &mut app.group_list_state);

    // Help
    let help = Paragraph::new(Line::from(vec![
        Span::styled("Enter", Style::default().fg(MATRIX_GREEN)),
        Span::styled(":Add │ ", Style::default().fg(Color::DarkGray)),
        Span::styled("Esc", Style::default().fg(MATRIX_GREEN)),
        Span::styled(":Cancel", Style::default().fg(Color::DarkGray)),
    ])).alignment(Alignment::Center);
    f.render_widget(help, chunks[2]);
}
