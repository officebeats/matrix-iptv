use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Color},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use chrono::{Utc, TimeZone};
use chrono_tz::Tz;
use std::str::FromStr;
use crate::app::{App, CurrentScreen};
use crate::ui::colors::{MATRIX_GREEN, DARK_GREEN, BRIGHT_GREEN};

fn clean_val(v: &serde_json::Value) -> String {
    let s = match v {
        serde_json::Value::String(s) => s.clone(),
        _ => v.to_string(),
    };
    s.replace('"', "")
}

pub fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),      // Tabs and Mode
            Constraint::Length(60),  // Stats (Dynamic space for time/exp)
        ])
        .split(area);

    if app.search_mode {
        let search_text = format!(" // SEARCH_PROTOCOLS: {}_", app.search_query);
        let p = Paragraph::new(search_text)
            .style(Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" SYSTEM_SEARCH_OVERRIDE ")
                    .border_style(Style::default().fg(MATRIX_GREEN)),
            );
        f.render_widget(p, area);
        return;
    }

    let header_title = if app.current_screen == CurrentScreen::SportsDashboard {
        " // SPORTS_UPLINK"
    } else {
        " // SYSTEM_NETWORK"
    };

    let mut left_spans = vec![Span::styled(
        header_title,
        Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD),
    )];
    
    // Color-coded MODE indicator
    if !app.config.processing_modes.is_empty() {
        left_spans.push(Span::styled(" [", Style::default().fg(DARK_GREEN)));
        
        let mut first = true;
        for mode in &app.config.processing_modes {
            if !first {
                left_spans.push(Span::styled("+", Style::default().fg(Color::DarkGray)));
            }
            first = false;
            
            match mode {
                crate::config::ProcessingMode::Merica => {
                    left_spans.push(Span::styled("'", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
                    left_spans.push(Span::styled("m", Style::default().fg(Color::Rgb(255, 50, 50)).add_modifier(Modifier::BOLD)));
                    left_spans.push(Span::styled("e", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)));
                    left_spans.push(Span::styled("r", Style::default().fg(Color::LightBlue).add_modifier(Modifier::BOLD)));
                    left_spans.push(Span::styled("i", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
                    left_spans.push(Span::styled("c", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
                    left_spans.push(Span::styled("a", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
                }
                crate::config::ProcessingMode::Sports => {
                    left_spans.push(Span::styled("SPORTS", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
                }
                crate::config::ProcessingMode::AllEnglish => {
                    left_spans.push(Span::styled("EN", Style::default().fg(Color::LightBlue).add_modifier(Modifier::BOLD)));
                }
            }
        }
        
        left_spans.push(Span::styled("] ", Style::default().fg(DARK_GREEN)));
    }

    left_spans.push(Span::styled(" ", Style::default()));

    let tabs = Paragraph::new(Line::from(left_spans)).block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(DARK_GREEN)),
    );
    f.render_widget(tabs, chunks[0]);

    if let Some(_) = &app.current_client {
        let name = app.config.accounts.get(app.selected_account_index).map(|a| a.name.clone()).unwrap_or_else(|| "Unknown".to_string());
        let tz_str = app.config.get_user_timezone();
        let user_tz: Tz = Tz::from_str(&tz_str).unwrap_or(chrono_tz::Europe::London);
        let now = Utc::now().with_timezone(&user_tz);
        let time = now.format("%I:%M%p").to_string(); // More compact time
        
        let (active, total, exp) = if let Some(info) = &app.account_info {
            let a = info.active_cons.as_ref().map(clean_val).unwrap_or_else(|| "0".to_string());
            let t = info.max_connections.as_ref().map(clean_val).unwrap_or_else(|| "1".to_string());
            let e = info.exp_date.as_ref().map(clean_val).unwrap_or_else(|| "N/A".to_string());
            (a, t, e)
        } else {
            ("?".to_string(), "?".to_string(), "N/A".to_string())
        };

        let exp_formatted = if let Ok(ts) = exp.parse::<i64>() {
             Utc.timestamp_opt(ts, 0).single().map(|dt| dt.format("%b %d").to_string()).unwrap_or(exp) // Compacter exp date
        } else {
            exp
        };

        let mut right_spans = Vec::new();
        right_spans.push(Span::styled(name, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));
        right_spans.push(Span::styled(" │ ", Style::default().fg(Color::DarkGray)));
        right_spans.push(Span::styled(time, Style::default().fg(Color::White)));
        right_spans.push(Span::styled(" │ ", Style::default().fg(Color::DarkGray)));
        right_spans.push(Span::styled(format!("Exp: {}", exp_formatted), Style::default().fg(Color::Yellow)));
        right_spans.push(Span::styled(" │ ", Style::default().fg(Color::DarkGray)));
        right_spans.push(Span::styled("\u{1f464} ", Style::default().fg(Color::White)));
        right_spans.push(Span::styled(format!("{}/{}", active, total), Style::default().fg(BRIGHT_GREEN).add_modifier(Modifier::BOLD)));

        let stats = Paragraph::new(Line::from(right_spans))
            .alignment(Alignment::Right)
            .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(DARK_GREEN)));
        f.render_widget(stats, chunks[1]);
    }
}
