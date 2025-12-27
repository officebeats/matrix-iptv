use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
    Frame,
};
use crate::app::App;
use crate::ui::colors::{MATRIX_GREEN, DARK_GREEN, BRIGHT_GREEN};

pub fn render_home(f: &mut Frame, app: &mut App, area: Rect) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    let logo_text = vec![
        "███╗   ███╗ █████╗ ████████╗██████╗ ██╗██╗  ██╗    ██╗██████╗ ████████╗██╗   ██╗     ██████╗██╗     ██╗",
        "████╗ ████║██╔══██╗╚══██╔══╝██╔══██╗██║╚██╗██╔╝    ██║██╔══██╗╚══██╔══╝██║   ██║    ██╔════╝██║     ██║",
        "██╔████╔██║███████║   ██║   ██████╔╝██║ ╚███╔╝     ██║██████╔╝   ██║   ██║   ██║    ██║     ██║     ██║",
        "██║╚██╔╝██║██╔══██║   ██║   ██╔══██╗██║ ██╔██╗     ██║██╔═══╝    ██║   ╚██╗ ██╔╝    ██║     ██║     ██║",
        "██║ ╚═╝ ██║██║  ██║   ██║   ██║  ██║██║██╔╝ ██╗    ██║██║        ██║    ╚████╔╝     ╚██████╗███████╗██║",
        "╚═╝     ╚═╝╚═╝  ╚═╝   ╚═╝   ╚═╝  ╚═╝╚═╝╚═╝  ╚═╝    ╚═╝╚═╝        ╚═╝     ╚═══╝       ╚═════╝╚══════╝╚═╝",
    ];

    f.render_widget(Paragraph::new(logo_text.join("\n")).style(Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)).alignment(Alignment::Center), main_layout[0]);

    let content_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(40),
            Constraint::Min(0),
        ])
        .split(main_layout[1]);

    let accounts: Vec<ListItem> = app.config.accounts.iter().map(|acc| {
        ListItem::new(Line::from(vec![
            Span::styled(" [NODE] ", Style::default().fg(Color::LightBlue)),
            Span::styled(acc.name.to_uppercase(), Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
        ]))
    }).collect();

    app.area_accounts = content_layout[0];
    f.render_stateful_widget(List::new(accounts).block(Block::default().title(" // PLAYLIST_NODES ").borders(Borders::ALL).border_type(BorderType::Double).border_style(Style::default().fg(DARK_GREEN))).highlight_style(Style::default().bg(MATRIX_GREEN).fg(Color::Black).add_modifier(Modifier::BOLD)).highlight_symbol(" » "), content_layout[0], &mut app.account_list_state);

    let main_zone_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(12),
            Constraint::Min(0),
        ])
        .split(content_layout[1]);

    let mut guides_text = Vec::new();
    if app.config.accounts.is_empty() {
        guides_text.extend(vec![
            Line::from(vec![Span::styled(" // SYSTEM_GUIDES:", Style::default().fg(BRIGHT_GREEN).add_modifier(Modifier::BOLD))]),
            Line::from(""),
            Line::from(vec![Span::styled(" [1] ", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)), Span::styled("Why CLI for IPTV?", Style::default().fg(MATRIX_GREEN))]),
            Line::from(vec![Span::styled(" [2] ", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)), Span::styled("Where do I get playlists?", Style::default().fg(MATRIX_GREEN))]),
            Line::from(vec![Span::styled(" [3] ", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)), Span::styled("What is IPTV?", Style::default().fg(MATRIX_GREEN))]),
        ]);
    } else {
        guides_text.extend(vec![
            Line::from(vec![Span::styled(" // SYSTEM_READY:", Style::default().fg(BRIGHT_GREEN).add_modifier(Modifier::BOLD))]),
            Line::from(""),
            Line::from(vec![Span::styled(" > Press [Enter] to Load Playlist", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::SLOW_BLINK))]),
        ]);
    }

    guides_text.extend(vec![
        Line::from(""),
        Line::from(vec![Span::styled(" ⚠ ", Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD)), Span::styled(" DISCLAIMER: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)), Span::styled("Matrix IPTV CLI is a client only.", Style::default().fg(Color::Yellow))]),
    ]);

    f.render_widget(Paragraph::new(guides_text).block(Block::default().borders(Borders::ALL).border_type(BorderType::Thick).border_style(Style::default().fg(DARK_GREEN)).padding(ratatui::widgets::Padding::new(2, 2, 1, 1))), main_zone_chunks[0]);

    let footer_info = Paragraph::new(vec![Line::from(vec![
        Span::styled(" Esc ", Style::default().fg(ratatui::style::Color::Cyan).add_modifier(Modifier::BOLD)), Span::styled("Back  ", Style::default().fg(Color::White)),
        Span::styled(" x ", Style::default().fg(ratatui::style::Color::Cyan).add_modifier(Modifier::BOLD)), Span::styled("Settings  ", Style::default().fg(Color::White)),
        Span::styled(" Enter ", Style::default().fg(ratatui::style::Color::Cyan).add_modifier(Modifier::BOLD)), Span::styled("Play", Style::default().fg(Color::White)),
    ])]).alignment(Alignment::Center);
    f.render_widget(footer_info, main_layout[2]);
}
