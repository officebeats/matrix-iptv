use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use crate::app::App;
use crate::ui::colors::{MATRIX_GREEN, SOFT_GREEN, TEXT_PRIMARY, TEXT_SECONDARY, TEXT_DIM};

pub fn render_loading(f: &mut Frame, app: &App, area: Rect) {
    if !app.state_loading {
        return;
    }

    // Subtle overlay across the background
    let row = "░".repeat(area.width as usize);
    let lines = vec![Line::from(row.as_str()); area.height as usize];
    let dim_paragraph = Paragraph::new(lines)
        .style(Style::default().fg(Color::DarkGray).bg(Color::Rgb(0, 0, 0)));
    f.render_widget(dim_paragraph, area);

    let popup_area = centered_rect(62, 50, area);
    f.render_widget(Clear, popup_area);

    // Dynamic title: show % when progress is available
    let title = if let Some(ref progress) = app.loading_progress {
        let pct = if progress.total > 0 { (progress.current * 100) / progress.total } else { 0 };
        format!(" loading  {}% ", pct)
    } else {
        " loading ".to_string()
    };

    use ratatui::symbols::border;
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(MATRIX_GREEN))
        .title(Span::styled(title, Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)))
        .title_alignment(Alignment::Center)
        .style(Style::default().bg(Color::Rgb(0, 0, 0)));

    f.render_widget(block, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(1), // Hero: spinner + current message
            Constraint::Length(1), // Gap
            Constraint::Length(1), // Progress bar (or empty)
            Constraint::Length(1), // Gap
            Constraint::Length(1), // Separator
            Constraint::Min(4),    // Log stream
            Constraint::Length(1), // Footer
        ])
        .split(popup_area);

    let tick = app.loading_tick;

    // Braille spinner — same chars as Claude Code
    let spinner_chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let spinner = spinner_chars[(tick / 2 % spinner_chars.len() as u64) as usize];

    let current_msg = app.loading_message.as_deref().unwrap_or("Initializing...");

    // ── Hero line ─────────────────────────────────────────────
    // Claude Code style: spinner glyph directly before the current operation
    let hero = Paragraph::new(Line::from(vec![
        Span::styled(format!("{} ", spinner), Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
        Span::styled(current_msg, Style::default().fg(TEXT_PRIMARY).add_modifier(Modifier::BOLD)),
    ]));
    f.render_widget(hero, chunks[0]);

    // ── Progress bar ──────────────────────────────────────────
    if let Some(ref progress) = app.loading_progress {
        let pct = if progress.total > 0 { (progress.current * 100) / progress.total } else { 0 };
        // Dynamic bar width: pad to fit inside border margins
        let bar_width = (popup_area.width as usize).saturating_sub(22).max(10).min(40);
        let filled = (pct * bar_width) / 100;
        let empty = bar_width.saturating_sub(filled);
        let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));

        let eta_str = progress.eta.as_ref().map(|d| {
            let secs = d.as_secs();
            if secs >= 60 { format!("{}m {}s", secs / 60, secs % 60) } else { format!("{}s", secs) }
        }).unwrap_or_else(|| "…".to_string());

        let bar_line = Paragraph::new(Line::from(vec![
            Span::styled(format!("[{}]", bar), Style::default().fg(SOFT_GREEN)),
            Span::styled(format!("  {:>3}%", pct), Style::default().fg(TEXT_PRIMARY).add_modifier(Modifier::BOLD)),
            Span::styled(
                format!("  {}/{}", progress.current, progress.total),
                Style::default().fg(TEXT_SECONDARY),
            ),
            Span::styled(
                format!("  eta {}", eta_str),
                Style::default().fg(SOFT_GREEN),
            ),
        ]));
        f.render_widget(bar_line, chunks[2]);
    }

    // ── Separator ─────────────────────────────────────────────
    let sep = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray));
    f.render_widget(sep, chunks[4]);

    // ── Log stream ────────────────────────────────────────────
    // Most recent entry is brightest; older entries fade to TEXT_DIM.
    // Uses › chevron (Gemini CLI / Claude Code style)
    let log_entries: Vec<Line> = app
        .loading_log
        .iter()
        .rev()
        .take(8)
        .enumerate()
        .map(|(i, msg)| {
            let text_color = if i == 0 { TEXT_SECONDARY } else { TEXT_DIM };
            Line::from(vec![
                Span::styled(" › ", Style::default().fg(Color::DarkGray)),
                Span::styled(msg.as_str(), Style::default().fg(text_color)),
            ])
        })
        .collect();

    let log_p = Paragraph::new(log_entries)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    f.render_widget(log_p, chunks[5]);

    // ── Footer ────────────────────────────────────────────────
    let footer = Paragraph::new(Line::from(Span::styled(
        "esc to cancel",
        Style::default().fg(TEXT_DIM),
    )))
    .alignment(Alignment::Center);
    f.render_widget(footer, chunks[6]);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
