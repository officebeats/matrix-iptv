use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, Paragraph, Wrap},
    Frame,
};
use crate::app::App;
use crate::ui::colors::{MATRIX_GREEN, SOFT_GREEN, HIGHLIGHT_BG, TEXT_PRIMARY, TEXT_DIM};

pub fn render_loading(f: &mut Frame, app: &App, area: Rect) {
    if !app.state_loading {
        return;
    }

    // Center the popup
    let popup_area = centered_rect(60, 45, area);
    f.render_widget(Clear, popup_area);

    // Build dynamic title with progress percentage
    let title = if let Some(ref progress) = app.loading_progress {
        let pct = if progress.total > 0 { (progress.current * 100) / progress.total } else { 0 };
        format!(" SYSTEM LINK ESTABLISHED · {}% ", pct)
    } else {
        " SYSTEM LINK ESTABLISHED ".to_string()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(MATRIX_GREEN))
        .title(title)
        .title_alignment(Alignment::Center)
        .style(Style::default().bg(Color::Black));
    
    f.render_widget(block.clone(), popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(4), // Header / Status (expanded for progress)
            Constraint::Length(1), // Separator
            Constraint::Min(5),    // Log
            Constraint::Length(1), // Separator
            Constraint::Length(1), // Footer
        ])
        .split(popup_area);

    let tick = app.loading_tick;
    // Animated spinner
    let spinner_chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let spinner = spinner_chars[(tick / 2 % spinner_chars.len() as u64) as usize];
    
    let current_msg = app.loading_message.as_deref().unwrap_or("Initializing...");
    
    // Status Header with progress info
    let mut status_lines = vec![
        Line::from(vec![
            Span::styled(format!(" {} ", spinner), Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
            Span::styled("EXECUTING PROTOCOL: ", Style::default().fg(TEXT_DIM)),
            Span::styled("DATA_SYNC_V4", Style::default().fg(SOFT_GREEN).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled(" CURRENT OPERATION: ", Style::default().fg(TEXT_DIM)),
            Span::styled(current_msg, Style::default().fg(TEXT_PRIMARY)),
        ]),
    ];

    // Add progress bar + ETA line when available
    if let Some(ref progress) = app.loading_progress {
        let pct = if progress.total > 0 { (progress.current * 100) / progress.total } else { 0 };
        let bar_width = 20usize;
        let filled = (pct * bar_width) / 100;
        let empty = bar_width - filled;
        let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));
        
        let eta_str = progress.eta.as_ref().map(|d| {
            let secs = d.as_secs();
            if secs >= 60 {
                format!("{}m {}s", secs / 60, secs % 60)
            } else {
                format!("{}s", secs)
            }
        }).unwrap_or_else(|| "calculating...".to_string());

        status_lines.push(Line::from(vec![
            Span::styled(format!(" [{}] ", bar), Style::default().fg(MATRIX_GREEN)),
            Span::styled(format!("{}/{}  ", progress.current, progress.total), Style::default().fg(TEXT_PRIMARY)),
            Span::styled(format!("ETA: {}", eta_str), Style::default().fg(SOFT_GREEN).add_modifier(Modifier::BOLD)),
        ]));
    }

    let status_p = Paragraph::new(status_lines)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    f.render_widget(status_p, chunks[0]);

    // Separator
    let sep = Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::DarkGray));
    f.render_widget(sep, chunks[1]);

    // Verbose Log (Matrix Stream)
    let log_entries: Vec<Line> = app.loading_log.iter().rev().take(12).map(|msg| {
        Line::from(vec![
            Span::styled(" > ", Style::default().fg(Color::DarkGray)),
            Span::styled(msg, Style::default().fg(SOFT_GREEN)),
        ])
    }).collect();
    
    let log_p = Paragraph::new(log_entries)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    f.render_widget(log_p, chunks[2]);

    // Footer with "Encryption" effect
    let footer_text = if (tick / 10) % 2 == 0 {
        "ENCRYPTED CONNECTION :: SECURE"
    } else {
        "ENCRYPTED CONNECTION :: ACTIVE"
    };
    let footer = Paragraph::new(Line::from(Span::styled(footer_text, Style::default().fg(TEXT_DIM))))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[4]);
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
