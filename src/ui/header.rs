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
            Constraint::Length(38), // Tabs
            Constraint::Min(0),     // Stats
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

    let current_tab = match app.current_screen {
        CurrentScreen::VodCategories | CurrentScreen::VodStreams => 1,
        CurrentScreen::Settings | CurrentScreen::TimezoneSettings => 2,
        CurrentScreen::SeriesCategories | CurrentScreen::SeriesStreams => 3,
        _ => 0,
    };

    let style_active = Style::default().bg(MATRIX_GREEN).fg(Color::Black).add_modifier(Modifier::BOLD);
    let separator = Span::styled(" / ", Style::default().fg(Color::LightBlue));

    let mut spans = vec![Span::styled(
        " // SYSTEM_NETWORK",
        Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD),
    )];
    
    if app.config.american_mode {
        spans.push(Span::styled(" \u{1f1fa}\u{1f1f8} ", Style::default().add_modifier(Modifier::BOLD)));
    }

    spans.push(separator.clone());
    spans.push(if current_tab == 0 { Span::styled(" [LIVE_UPLINK] ", style_active) } else { Span::styled(" LIVE_UPLINK ", Style::default().fg(MATRIX_GREEN)) });

    spans.push(separator.clone());
    spans.push(if current_tab == 1 { Span::styled(" [MOVIE_ACCESS] ", style_active) } else { Span::styled(" MOVIE_ACCESS ", Style::default().fg(MATRIX_GREEN)) });

    spans.push(separator.clone());
    spans.push(if current_tab == 3 { Span::styled(" [SERIAL_LOGS] ", style_active) } else { Span::styled(" SERIAL_LOGS ", Style::default().fg(MATRIX_GREEN)) });

    spans.push(separator.clone());
    spans.push(if current_tab == 2 { Span::styled(" [CORE_CONFIG] ", style_active) } else { Span::styled(" CORE_CONFIG ", Style::default().fg(MATRIX_GREEN)) });

    let tabs = Paragraph::new(Line::from(spans)).block(
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
        let time = now.format("%I:%M:%S %p %Z").to_string();
        
        let (active, total, exp) = if let Some(info) = &app.account_info {
            let a = info.active_cons.as_ref().map(clean_val).unwrap_or_else(|| "0".to_string());
            let t = info.max_connections.as_ref().map(clean_val).unwrap_or_else(|| "1".to_string());
            let e = info.exp_date.as_ref().map(clean_val).unwrap_or_else(|| "N/A".to_string());
            (a, t, e)
        } else {
            ("?".to_string(), "?".to_string(), "N/A".to_string())
        };

        let exp_formatted = if let Ok(ts) = exp.parse::<i64>() {
             Utc.timestamp_opt(ts, 0).single().map(|dt| dt.format("%m/%d/%Y").to_string()).unwrap_or(exp)
        } else {
            exp
        };

        let stats_text = format!("{} | {} | Exp: {} | \u{1f464} {}/{}", name, time, exp_formatted, active, total);
        let stats = Paragraph::new(stats_text)
            .alignment(Alignment::Right)
            .style(Style::default().fg(BRIGHT_GREEN).add_modifier(Modifier::BOLD))
            .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(DARK_GREEN)));
        f.render_widget(stats, chunks[1]);
    }
}
