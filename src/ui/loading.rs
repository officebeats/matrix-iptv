use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};
use crate::app::App;
use crate::ui::colors::{MATRIX_GREEN, DARK_GREEN, SOFT_GREEN, TEXT_PRIMARY};
use crate::ui::utils::centered_rect;

pub fn render_loading(f: &mut Frame, app: &App, area: Rect) {
    let popup_area = centered_rect(50, 20, area);
    f.render_widget(Clear, popup_area);

    // Outer frame — clean rounded border, soft green
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(SOFT_GREEN));
    f.render_widget(block, popup_area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),    // Rain area
            Constraint::Length(1), // Separator
            Constraint::Length(3), // Message area
        ])
        .margin(1)
        .split(popup_area);

    // Katakana rain — sparser, more cinematic
    let rain_chars = vec!["ｱ", "ｲ", "ｳ", "ｴ", "ｵ", "ｶ", "ｷ", "ｸ", "ｹ", "ｺ",
                          "ｻ", "ｼ", "ｽ", "ｾ", "ｿ", "ﾀ", "ﾁ", "ﾂ", "ﾃ", "ﾄ",
                          " ", " ", " ", " "]; // 20% space = sparser feel
    let tick = app.loading_tick;

    let mut rain_lines = Vec::new();
    for i in 0..layout[0].height {
        let mut spans = Vec::new();
        for j in 0..layout[0].width {
            let offset = (i as u64 + j as u64 * 3 + tick) % rain_chars.len() as u64;
            let ch = rain_chars[offset as usize];

            // Three intensity levels for depth effect
            let brightness = ((j as u64 + i as u64 * 2 + tick / 2) % 5) as u8;
            let color = match brightness {
                0 => MATRIX_GREEN,                    // Bright trail head
                1 => SOFT_GREEN,                      // Medium
                _ => DARK_GREEN,                      // Dim background
            };
            spans.push(Span::styled(ch, Style::default().fg(color)));
        }
        rain_lines.push(Line::from(spans));
    }

    f.render_widget(
        Paragraph::new(rain_lines).alignment(Alignment::Center),
        layout[0],
    );

    // Thin separator line
    let sep = "─".repeat(layout[1].width as usize);
    f.render_widget(
        Paragraph::new(sep).style(Style::default().fg(DARK_GREEN)),
        layout[1],
    );

    // Loading message — clean, Claude Code style
    let msg = if let Some(progress) = &app.loading_progress {
        progress.to_message()
    } else {
        app.loading_message.as_deref().unwrap_or("Connecting...").to_string()
    };

    // Animated dots
    let dots = ".".repeat(((tick / 8) % 4) as usize);
    let spinner_chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let spinner = spinner_chars[(tick % spinner_chars.len() as u64) as usize];

    let loading_text = Paragraph::new(Line::from(vec![
        Span::styled(format!(" {} ", spinner), Style::default().fg(MATRIX_GREEN)),
        Span::styled(msg, Style::default().fg(TEXT_PRIMARY).add_modifier(Modifier::BOLD)),
        Span::styled(dots, Style::default().fg(SOFT_GREEN)),
    ]))
    .alignment(Alignment::Center);

    f.render_widget(loading_text, layout[2]);
}
