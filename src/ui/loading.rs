use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use crate::app::App;
use crate::ui::colors::{MATRIX_GREEN, SOFT_GREEN, TEXT_PRIMARY, TEXT_SECONDARY, TEXT_DIM};

pub fn render_loading(f: &mut Frame, app: &App, area: Rect) {
    if !app.session.state_loading {
        return;
    }

    // Subtle overlay across the background
    let row = "░".repeat(area.width as usize);
    let lines = vec![Line::from(row.as_str()); area.height as usize];
    let dim_paragraph = Paragraph::new(lines)
        .style(Style::default().fg(Color::DarkGray).bg(Color::Rgb(0, 0, 0)));
    f.render_widget(dim_paragraph, area);

    // Perfectly fitted fixed-height modal popup
    let popup_area = centered_rect(80, 9, area);
    f.render_widget(Clear, popup_area);

    // Dynamic title: show % when progress is available
    let title = if let Some(ref progress) = app.session.loading_progress {
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
        .margin(1)
        .constraints([
            Constraint::Length(1), // Hero: spinner + current message
            Constraint::Length(1), // Gap
            Constraint::Length(1), // Progress bar (or empty)
            Constraint::Min(0),    // Spacer
            Constraint::Length(1), // Footer
        ])
        .split(popup_area);

    let tick = app.session.loading_tick;

    // Matrix style Katakana spinner and decoding effect
    let katakana = ['ｦ', 'ｧ', 'ｨ', 'ｩ', 'ｪ', 'ｫ', 'ｬ', 'ｭ', 'ｮ', 'ｯ', 'ｰ', 'ｱ', 'ｲ', 'ｳ', 'ｴ', 'ｵ', 'ｶ', 'ｷ', 'ｸ', 'ｹ', 'ｺ', 'ｻ', 'ｼ', 'ｽ', 'ｾ', 'ｿ', 'ﾀ', 'ﾁ', 'ﾂ', 'ﾃ', 'ﾄ', 'ﾅ', 'ﾆ', 'ﾇ', 'ﾈ', 'ﾉ', 'ﾊ', 'ﾋ', 'ﾌ', 'ﾍ', 'ﾎ', 'ﾏ', 'ﾐ', 'ﾑ', 'ﾒ', 'ﾓ', 'ﾔ', 'ﾕ', 'ﾖ', 'ﾗ', 'ﾘ', 'ﾙ', 'ﾚ', 'ﾛ', 'ﾜ', 'ﾝ'];
    let spinner = katakana[(tick as usize) % katakana.len()];

    let glitch_len = 8;
    let mut glitch_str = String::with_capacity(glitch_len);
    for i in 0..glitch_len {
        let char_idx = (tick.wrapping_add((i * 13) as u64) as usize) % katakana.len();
        glitch_str.push(katakana[char_idx]);
    }

    let current_msg = app.session.loading_message.as_deref().unwrap_or("Initializing system...");

    // ── Hero line ─────────────────────────────────────────────
    // Matrix style: Katakana spinner + Message + Glitch decoding tail
    let hero = Paragraph::new(Line::from(vec![
        Span::styled(format!("{} ", spinner), Style::default().fg(Color::Rgb(200, 255, 200)).add_modifier(Modifier::BOLD)),
        Span::styled(current_msg, Style::default().fg(TEXT_PRIMARY).add_modifier(Modifier::BOLD)),
        Span::styled(format!(" [{}]", glitch_str), Style::default().fg(MATRIX_GREEN)),
    ]));
    f.render_widget(hero, chunks[0]);

    // ── Progress bar ──────────────────────────────────────────
    if let Some(ref progress) = app.session.loading_progress {
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

    // ── Footer ────────────────────────────────────────────────
    let footer = Paragraph::new(Line::from(Span::styled(
        "esc to cancel",
        Style::default().fg(TEXT_DIM),
    )))
    .alignment(Alignment::Center);
    f.render_widget(footer, chunks[4]);
}

fn centered_rect(percent_x: u16, height: u16, r: Rect) -> Rect {
    let vertical_margin = r.height.saturating_sub(height) / 2;
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(vertical_margin),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(r);

    let horizontal_margin = (100_u16.saturating_sub(percent_x)) / 2;
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(horizontal_margin),
            Constraint::Percentage(percent_x),
            Constraint::Min(0),
        ])
        .split(popup_layout[1])[1]
}
