use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};
use crate::app::App;
use crate::ui::colors::{MATRIX_GREEN, DARK_GREEN, BRIGHT_GREEN};
use crate::ui::utils::centered_rect;

pub fn render_loading(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD));

    let popup_area = centered_rect(50, 15, area);
    f.render_widget(Clear, popup_area);
    f.render_widget(block, popup_area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(1),
            Constraint::Length(3),
        ])
        .margin(1)
        .split(popup_area);

    let rain_chars = vec!["ｱ", "ｲ", "ｳ", "ｴ", "ｵ", "ｶ", "ｷ", "ｸ", "ｹ", "ｺ", "ｻ", "ｼ", "ｽ", "ｾ", "ｿ", "ﾀ", "ﾁ", "ﾂ", "ﾃ", "ﾄ"];
    let tick = app.loading_tick;

    let mut rain_lines = Vec::new();
    for i in 0..layout[0].height {
        let mut spans = Vec::new();
        for j in 0..layout[0].width {
            let offset = (i as u64 + j as u64 + tick) % rain_chars.len() as u64;
            let char = rain_chars[offset as usize];
            let opacity = if (j as u64 + tick / 2) % 3 == 0 { MATRIX_GREEN } else { DARK_GREEN };
            spans.push(Span::styled(char, Style::default().fg(opacity)));
        }
        rain_lines.push(Line::from(spans));
    }

    f.render_widget(Paragraph::new(rain_lines).alignment(Alignment::Center), layout[0]);
    f.render_widget(Paragraph::new("─".repeat(layout[1].width as usize)).style(Style::default().fg(DARK_GREEN)), layout[1]);

    // Show detailed progress if available, else fallback to loading message
    let msg = if let Some(progress) = &app.loading_progress {
        progress.to_message()
    } else {
        app.loading_message.as_deref().unwrap_or("SECURE_UPLINK_INITIALIZING...").to_string()
    };
    
    let loading_text = Paragraph::new(format!(" > {} < ", msg.to_uppercase()))
        .style(Style::default().fg(BRIGHT_GREEN).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);

    // Bottom status bar inside popup
    let status_footer = Paragraph::new(" [SYSTEM_STATUS: ACTIVE_LINK_ESTABLISHED] ")
        .style(Style::default().fg(DARK_GREEN).add_modifier(Modifier::ITALIC))
        .alignment(Alignment::Right);

    f.render_widget(loading_text, layout[2]);
    
    // Add a small footer at the very bottom of the popup - with robust bounds safety
    if popup_area.height >= 3 && popup_area.width >= 4 {
        let footer_rect = Rect::new(
            popup_area.x.saturating_add(1), 
            popup_area.y.saturating_add(popup_area.height.saturating_sub(2)), 
            popup_area.width.saturating_sub(2), 
            1
        );
        f.render_widget(status_footer, footer_rect);
    }
}
