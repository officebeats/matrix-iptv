use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph, Wrap},
    Frame,
};
use crate::app::App;
use crate::ui::colors::{MATRIX_GREEN, HIGHLIGHT_BG, TEXT_PRIMARY, TEXT_SECONDARY, TEXT_DIM};

pub fn render_home(f: &mut Frame, app: &mut App, area: Rect) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),   // Logo
            Constraint::Min(0),      // Content
            Constraint::Length(1),   // Footer
        ])
        .split(area);

    // Logo — clean, minimal
    let logo_lines: Vec<Line> = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  matrix", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
            Span::styled("-iptv", Style::default().fg(TEXT_PRIMARY).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  Terminal streaming client", Style::default().fg(TEXT_DIM)),
        ]),
    ];

    f.render_widget(Paragraph::new(logo_lines), main_layout[0]);

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
            Span::styled("  ", Style::default()),
            Span::styled(&acc.name, Style::default().fg(TEXT_PRIMARY).add_modifier(Modifier::BOLD)),
        ];

        // Sync status
        if let Some(last) = acc.last_refreshed {
            let secs_ago = now - last;
            let hours_ago = secs_ago / 3600;
            let days_ago = hours_ago / 24;
            let weeks_ago = days_ago / 7;

            let (time_text, style) = if hours_ago < 1 {
                ("just now".to_string(), Style::default().fg(MATRIX_GREEN))
            } else if hours_ago < 2 {
                ("1h ago".to_string(), Style::default().fg(MATRIX_GREEN))
            } else if hours_ago < 24 {
                (format!("{}h ago", hours_ago), Style::default().fg(TEXT_SECONDARY))
            } else if days_ago < 2 {
                ("yesterday".to_string(), Style::default().fg(TEXT_SECONDARY))
            } else if days_ago < 7 {
                (format!("{}d ago", days_ago), Style::default().fg(TEXT_SECONDARY))
            } else if weeks_ago < 2 {
                ("1w ago".to_string(), Style::default().fg(Color::Rgb(255, 200, 80)))
            } else if weeks_ago < 5 {
                (format!("{}w ago", weeks_ago), Style::default().fg(Color::Rgb(255, 200, 80)))
            } else {
                let months_ago = days_ago / 30;
                if months_ago < 2 {
                    ("1mo ago".to_string(), Style::default().fg(Color::Rgb(255, 100, 100)))
                } else {
                    (format!("{}mo ago", months_ago), Style::default().fg(Color::Rgb(255, 100, 100)))
                }
            };

            spans.push(Span::styled("  ", Style::default()));
            spans.push(Span::styled(time_text, style));
        } else {
            spans.push(Span::styled("  new", Style::default().fg(MATRIX_GREEN)));
        }

        ListItem::new(Line::from(spans))
    }).collect();

    app.area_accounts = content_layout[0];

    // Playlists title + list — no border box
    let playlist_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(0),
        ])
        .split(content_layout[0]);

    let playlist_title = Paragraph::new(Line::from(vec![
        Span::styled("  playlists", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
    ]));
    f.render_widget(playlist_title, playlist_chunks[0]);

    f.render_stateful_widget(
        List::new(accounts)
            .highlight_style(Style::default().bg(HIGHLIGHT_BG).fg(MATRIX_GREEN).add_modifier(Modifier::BOLD))
            .highlight_symbol(" ▎"),
        playlist_chunks[1],
        &mut app.account_list_state
    );

    // Right side — guides
    let mut guides_text = Vec::new();
    if app.config.accounts.is_empty() {
        guides_text.extend(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  getting started", Style::default().fg(TEXT_PRIMARY).add_modifier(Modifier::BOLD))
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled("1", Style::default().fg(MATRIX_GREEN)),
                Span::styled("  The TUI Edge: Why CLI?", Style::default().fg(TEXT_PRIMARY))
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled("2", Style::default().fg(MATRIX_GREEN)),
                Span::styled("  Acquiring Playlists safely", Style::default().fg(TEXT_PRIMARY))
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled("3", Style::default().fg(MATRIX_GREEN)),
                Span::styled("  Understanding the IPTV Protocol", Style::default().fg(TEXT_PRIMARY))
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Press ", Style::default().fg(TEXT_SECONDARY)),
                Span::styled("n", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
                Span::styled(" to add your first playlist", Style::default().fg(TEXT_SECONDARY))
            ]),
        ]);
    } else {
        guides_text.extend(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  Select a playlist and press ", Style::default().fg(TEXT_SECONDARY)),
                Span::styled("enter", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
                Span::styled(" to connect", Style::default().fg(TEXT_SECONDARY))
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled("1", Style::default().fg(MATRIX_GREEN)),
                Span::styled("  The TUI Edge: Why CLI?", Style::default().fg(TEXT_PRIMARY))
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled("2", Style::default().fg(MATRIX_GREEN)),
                Span::styled("  Acquiring Playlists safely", Style::default().fg(TEXT_PRIMARY))
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled("3", Style::default().fg(MATRIX_GREEN)),
                Span::styled("  Understanding the IPTV Protocol", Style::default().fg(TEXT_PRIMARY))
            ]),
        ]);
    }

    f.render_widget(
        Paragraph::new(guides_text)
            .wrap(Wrap { trim: true }),
        content_layout[1]
    );

    crate::ui::footer::render_footer(f, app, main_layout[2]);
}
