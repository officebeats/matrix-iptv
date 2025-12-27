use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};
use crate::app::{App, Guide};
use crate::ui::colors::{MATRIX_GREEN, DARK_GREEN, BRIGHT_GREEN};
use crate::ui::utils::centered_rect;

pub fn render_help_popup(f: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" // COMMAND_LEGEND ")
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(DARK_GREEN));

    let area = centered_rect(60, 60, area);
    f.render_widget(Clear, area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(8),
            Constraint::Min(0),
        ])
        .split(area);

    let shortcuts = vec![
        "Keyboard Shortcuts:",
        "",
        "  /       - Toggle Search",
        "  Tab     - Switch Panes (Categories/Streams)",
        "  m       - Switch Mode (Live TV / VOD)",
        "  f       - Toggle Favorite",
        "  Enter   - Select / Play",
        "  j / k   - Navigate Down / Up",
        "  q       - Quit",
    ];
    let shortcuts_p = Paragraph::new(shortcuts.join("\n")).style(Style::default().fg(ratatui::style::Color::White));
    f.render_widget(shortcuts_p, chunks[0]);
}

pub fn render_guide_popup(f: &mut Frame, app: &App, area: Rect) {
    if let Some(guide) = app.show_guide {
        let content = match guide {
            Guide::WhatIsApp => include_str!("../content/what_is_this_app.md"),
            Guide::HowToGetPlaylists => include_str!("../content/how_to_get_playlists.md"),
            Guide::WhatIsIptv => include_str!("../content/what_is_iptv.md"),
        };

        let lines: Vec<Line> = content
            .lines()
            .map(|l| {
                if l.starts_with("# ") {
                    Line::from(Span::styled(l.trim_start_matches("# ").to_uppercase(), Style::default().fg(BRIGHT_GREEN).add_modifier(Modifier::BOLD)))
                } else {
                    Line::from(l.to_string())
                }
            })
            .collect();

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(Style::default().fg(DARK_GREEN))
            .title(Span::styled(" // SYSTEM_PROTOCOLS ", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)));

        let area = centered_rect(80, 80, area);
        f.render_widget(Clear, area);

        let p = Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: true })
            .scroll((app.guide_scroll, 0));

        f.render_widget(p, area);
    }
}

pub fn render_content_type_selection(f: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .title(" // CHOOSE_PATH ")
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(MATRIX_GREEN));

    let area = centered_rect(70, 50, area);
    f.render_widget(Clear, area);
    f.render_widget(block.clone(), area);

    let inner = block.inner(area);
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(10),
            Constraint::Min(4),
        ])
        .margin(1)
        .split(inner);

    let title = Paragraph::new("Select Content Type:")
        .alignment(Alignment::Center)
        .style(Style::default().fg(BRIGHT_GREEN).add_modifier(Modifier::BOLD));
    f.render_widget(title, layout[0]);

    let selected = app.selected_content_type_index;
    let items: Vec<ListItem> = vec![
        (0, "(=====)", "LIVE CHANNELS", "[Red Pill]", ratatui::style::Color::Red),
        (1, "(=====)", "MOVIES (VOD)", "[Blue Pill]", ratatui::style::Color::Cyan),
        (2, "(=====)", "SERIES (VOD)", "[White Rabbit]", ratatui::style::Color::White),
    ]
    .into_iter()
    .map(|(i, icon, label, sub, color)| {
        let is_selected = i == selected;
        let icon_style = Style::default().fg(color).add_modifier(Modifier::BOLD);
        let text_style = if is_selected { Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD) } else { Style::default().fg(ratatui::style::Color::White) };
        ListItem::new(Line::from(vec![
            Span::styled(format!("  {} ", icon), icon_style),
            Span::styled(label, text_style),
            Span::styled(format!(" {}", sub), Style::default().fg(color)),
        ]))
    })
    .collect();

    let mut list_state = ratatui::widgets::ListState::default();
    list_state.select(Some(selected));
    f.render_stateful_widget(List::new(items).highlight_symbol(">> "), layout[1], &mut list_state);

    let (quote, color) = match selected {
        0 => ("\"You take the red pill... you stay in Wonderland,\nand I show you how deep the rabbit hole goes.\"", ratatui::style::Color::Red),
        1 => ("\"You take the blue pill... the story ends,\nyou wake up in your bed and believe whatever you want to believe.\"", ratatui::style::Color::Cyan),
        _ => ("\"Follow the white rabbit.\"", ratatui::style::Color::White),
    };

    f.render_widget(Paragraph::new(quote).alignment(Alignment::Center).wrap(Wrap { trim: true }).style(Style::default().fg(color).add_modifier(Modifier::ITALIC)), layout[2]);
}
pub fn render_error_popup(f: &mut Frame, area: Rect, error: &str) {
    let block = Block::default()
        .title(Span::styled(" // SYSTEM_ERROR_OVERRIDE ", Style::default().fg(ratatui::style::Color::Red).add_modifier(Modifier::BOLD)))
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(ratatui::style::Color::Red));

    let area = centered_rect(60, 30, area);
    f.render_widget(Clear, area);
    f.render_widget(block, area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    let error_text = Paragraph::new(error)
        .style(Style::default().fg(ratatui::style::Color::White))
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Center);

    let dismiss_text = Paragraph::new("Press [Esc] to Acknowledge")
        .style(Style::default().fg(DARK_GREEN))
        .alignment(Alignment::Center);

    f.render_widget(error_text, layout[0]);
    f.render_widget(dismiss_text, layout[1]);
}
