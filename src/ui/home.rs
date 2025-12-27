use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
    Frame,
};
use crate::app::App;
use crate::ui::colors::{MATRIX_GREEN, DARK_GREEN};

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

    let logo_lines: Vec<Line> = logo_text.iter().map(|line| {
        let spans: Vec<Span> = line.chars().map(|c| {
            if c == '█' {
                Span::styled(c.to_string(), Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD))
            } else if c == ' ' {
                Span::raw(" ")
            } else {
                Span::styled(c.to_string(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
            }
        }).collect();
        Line::from(spans)
    }).collect();

    f.render_widget(Paragraph::new(logo_lines).alignment(Alignment::Center), main_layout[0]);

    let content_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(42),
            Constraint::Min(0),
        ])
        .split(main_layout[1]);

    let accounts: Vec<ListItem> = app.config.accounts.iter().map(|acc| {
        ListItem::new(Line::from(vec![
            Span::styled(" [NODE] ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(acc.name.to_uppercase(), Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
        ]))
    }).collect();

    app.area_accounts = content_layout[0];
    f.render_stateful_widget(
        List::new(accounts)
            .block(Block::default()
                .title(Line::from(vec![
                    Span::styled(" // ", Style::default().fg(DARK_GREEN)),
                    Span::styled("PLAYLIST_NODES ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                ]))
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(Style::default().fg(DARK_GREEN)))
            .highlight_style(Style::default().bg(MATRIX_GREEN).fg(Color::Black).add_modifier(Modifier::BOLD))
            .highlight_symbol(" » "), 
        content_layout[0], 
        &mut app.account_list_state
    );

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
            Line::from(vec![
                Span::styled(" // ", Style::default().fg(DARK_GREEN)),
                Span::styled("SYSTEM_GUIDES:", Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(" [", Style::default().fg(Color::White)),
                Span::styled("1", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
                Span::styled("] ", Style::default().fg(Color::White)),
                Span::styled("Why CLI for IPTV?", Style::default().fg(MATRIX_GREEN))
            ]),
            Line::from(vec![
                Span::styled(" [", Style::default().fg(Color::White)),
                Span::styled("2", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
                Span::styled("] ", Style::default().fg(Color::White)),
                Span::styled("Where do I get playlists?", Style::default().fg(MATRIX_GREEN))
            ]),
            Line::from(vec![
                Span::styled(" [", Style::default().fg(Color::White)),
                Span::styled("3", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
                Span::styled("] ", Style::default().fg(Color::White)),
                Span::styled("What is IPTV?", Style::default().fg(MATRIX_GREEN))
            ]),
        ]);
    } else {
        guides_text.extend(vec![
            Line::from(vec![
                Span::styled(" // ", Style::default().fg(DARK_GREEN)),
                Span::styled("SYSTEM_READY:", Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(" > ", Style::default().fg(Color::White)),
                Span::styled("Press [Enter] to Load Playlist", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::SLOW_BLINK))
            ]),
        ]);
    }

    guides_text.extend(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(" ⚠ ", Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD)), 
            Span::styled(" DISCLAIMER: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)), 
            Span::styled("Matrix IPTV CLI is a client only.", Style::default().fg(Color::White))
        ]),
    ]);

    f.render_widget(
        Paragraph::new(guides_text)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Double) // Changed to Double for consistency
                .border_style(Style::default().fg(DARK_GREEN))
                .padding(ratatui::widgets::Padding::new(2, 2, 1, 1))), 
        main_zone_chunks[0]
    );

    crate::ui::footer::render_footer(f, app, main_layout[2]);
}
