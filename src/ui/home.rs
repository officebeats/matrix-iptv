use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph, Wrap},
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

    let now = chrono::Utc::now().timestamp();
    let accounts: Vec<ListItem> = app.config.accounts.iter().map(|acc| {
        let mut spans = vec![
            Span::styled(" [NODE] ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(acc.name.to_uppercase(), Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
        ];
        
        // Add sync status indicator with human-readable times
        if let Some(last) = acc.last_refreshed {
            let secs_ago = now - last;
            let hours_ago = secs_ago / 3600;
            let days_ago = hours_ago / 24;
            let weeks_ago = days_ago / 7;
            
            let (time_text, style) = if hours_ago < 1 {
                ("just now".to_string(), Style::default().fg(MATRIX_GREEN))
            } else if hours_ago < 2 {
                ("1 hour ago".to_string(), Style::default().fg(MATRIX_GREEN))
            } else if hours_ago < 24 {
                (format!("{} hours ago", hours_ago), Style::default().fg(Color::DarkGray))
            } else if days_ago < 2 {
                ("yesterday".to_string(), Style::default().fg(Color::DarkGray))
            } else if days_ago < 7 {
                (format!("{} days ago", days_ago), Style::default().fg(Color::DarkGray))
            } else if weeks_ago < 2 {
                ("1 week ago".to_string(), Style::default().fg(Color::Yellow))
            } else if weeks_ago < 5 {
                (format!("{} weeks ago", weeks_ago), Style::default().fg(Color::Yellow))
            } else {
                let months_ago = days_ago / 30;
                if months_ago < 2 {
                    ("1 month ago".to_string(), Style::default().fg(Color::Red))
                } else {
                    (format!("{} months ago", months_ago), Style::default().fg(Color::Red))
                }
            };
            
            spans.push(Span::styled(" 🔄", Style::default().fg(Color::White)));
            spans.push(Span::styled(format!("{}", time_text), style));
        } else {
            spans.push(Span::styled(" (NEW)", Style::default().fg(Color::Cyan)));
        }
        
        ListItem::new(Line::from(spans))
    }).collect();

    app.area_accounts = content_layout[0];
    let inner_list_area = crate::ui::common::render_composite_block(f, content_layout[0], Some(" // PLAYLIST_NODES "));
    
    f.render_stateful_widget(
        List::new(accounts)
            .highlight_style(Style::default().bg(MATRIX_GREEN).fg(Color::Black).add_modifier(Modifier::BOLD))
            .highlight_symbol(" » "), 
        inner_list_area, 
        &mut app.account_list_state
    );

    let main_zone_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),
            Constraint::Length(0),
        ])
        .split(content_layout[1]);

    let mut guides_text = Vec::new();
    if app.config.accounts.is_empty() {
        guides_text.extend(vec![
            Line::from(vec![
                Span::styled("// ", Style::default().fg(DARK_GREEN)),
                Span::styled("SYSTEM_GUIDES", Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  [", Style::default().fg(Color::DarkGray)),
                Span::styled("1", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
                Span::styled("] ", Style::default().fg(Color::DarkGray)),
                Span::styled("Why CLI for IPTV?", Style::default().fg(Color::White))
            ]),
            Line::from(vec![
                Span::styled("  [", Style::default().fg(Color::DarkGray)),
                Span::styled("2", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
                Span::styled("] ", Style::default().fg(Color::DarkGray)),
                Span::styled("Where do I get playlists?", Style::default().fg(Color::White))
            ]),
            Line::from(vec![
                Span::styled("  [", Style::default().fg(Color::DarkGray)),
                Span::styled("3", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
                Span::styled("] ", Style::default().fg(Color::DarkGray)),
                Span::styled("What is IPTV?", Style::default().fg(Color::White))
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Press ", Style::default().fg(Color::DarkGray)),
                Span::styled("n", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
                Span::styled(" to add your first playlist", Style::default().fg(Color::DarkGray))
            ]),
        ]);
    } else {
        guides_text.extend(vec![
            Line::from(vec![
                Span::styled("// ", Style::default().fg(DARK_GREEN)),
                Span::styled("SYSTEM_READY", Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  » ", Style::default().fg(MATRIX_GREEN)),
                Span::styled("Select a playlist and press ", Style::default().fg(Color::DarkGray)),
                Span::styled("Enter", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
                Span::styled(" to connect", Style::default().fg(Color::DarkGray))
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  [", Style::default().fg(Color::DarkGray)),
                Span::styled("1", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
                Span::styled("] ", Style::default().fg(Color::DarkGray)),
                Span::styled("Why CLI for IPTV?", Style::default().fg(Color::White))
            ]),
            Line::from(vec![
                Span::styled("  [", Style::default().fg(Color::DarkGray)),
                Span::styled("2", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
                Span::styled("] ", Style::default().fg(Color::DarkGray)),
                Span::styled("Where do I get playlists?", Style::default().fg(Color::White))
            ]),
            Line::from(vec![
                Span::styled("  [", Style::default().fg(Color::DarkGray)),
                Span::styled("3", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
                Span::styled("] ", Style::default().fg(Color::DarkGray)),
                Span::styled("What is IPTV?", Style::default().fg(Color::White))
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Playlists: ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{}", app.config.accounts.len()), Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
                Span::styled(" configured", Style::default().fg(Color::DarkGray))
            ]),
        ]);
    }

    guides_text.extend(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  ⚠ ", Style::default().fg(Color::Yellow)), 
            Span::styled("DISCLAIMER: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)), 
            Span::styled("Matrix IPTV is a client only.", Style::default().fg(Color::DarkGray))
        ]),
    ]);

    let inner_guides_area = crate::ui::common::render_composite_block(f, main_zone_chunks[0], None);

    f.render_widget(
        Paragraph::new(guides_text)
            .wrap(Wrap { trim: true }),
        inner_guides_area
    );

    crate::ui::footer::render_footer(f, app, main_layout[2]);
}
