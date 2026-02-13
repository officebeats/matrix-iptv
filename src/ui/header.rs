use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use chrono::{Utc, TimeZone};
use chrono_tz::Tz;
use std::str::FromStr;
use crate::app::{App, CurrentScreen};
use crate::ui::colors::{MATRIX_GREEN, DARK_GREEN, SOFT_GREEN, TEXT_PRIMARY, TEXT_SECONDARY, TEXT_DIM};

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
            Constraint::Min(0),
            Constraint::Length(60),
        ])
        .split(area);

    if app.search_mode {
        let search_text = format!(" >_ {}", app.search_state.query);
        let p = Paragraph::new(search_text)
            .style(Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD))
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .title(Span::styled(" search ", Style::default().fg(SOFT_GREEN)))
                    .border_style(Style::default().fg(DARK_GREEN)),
            );
        f.render_widget(p, area);
        return;
    }

    // Breadcrumb — clean, minimal
    let breadcrumb_parts: Vec<(&str, bool)> = match app.current_screen {
        CurrentScreen::Home => vec![("home", true)],
        CurrentScreen::Categories => vec![("home", false), ("tv", true)],
        CurrentScreen::Streams => {
            vec![("home", false), ("tv", false), ("streams", true)]
        },
        CurrentScreen::VodCategories => vec![("home", false), ("movies", true)],
        CurrentScreen::VodStreams => vec![("home", false), ("movies", false), ("browse", true)],
        CurrentScreen::SeriesCategories => vec![("home", false), ("series", true)],
        CurrentScreen::SeriesStreams => vec![("home", false), ("series", false), ("browse", true)],
        CurrentScreen::Settings => vec![("home", false), ("settings", true)],
        CurrentScreen::SportsDashboard => vec![("home", false), ("sports", true)],
        CurrentScreen::GlobalSearch => vec![("home", false), ("search", true)],
        _ => vec![("matrix-iptv", true)],
    };

    let mut left_spans: Vec<Span> = Vec::new();
    left_spans.push(Span::styled(" ", Style::default()));
    
    for (i, (part, is_active)) in breadcrumb_parts.iter().enumerate() {
        if i > 0 {
            left_spans.push(Span::styled(" › ", Style::default().fg(TEXT_DIM)));
        }
        if *is_active {
            left_spans.push(Span::styled(*part, Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)));
        } else {
            left_spans.push(Span::styled(*part, Style::default().fg(TEXT_SECONDARY)));
        }
    }

    // Mode indicator (compact)
    if !app.config.processing_modes.is_empty() {
        left_spans.push(Span::styled("  ", Style::default()));
        left_spans.push(Span::styled("[", Style::default().fg(TEXT_DIM)));
        
        let mut first = true;
        for mode in &app.config.processing_modes {
            if !first {
                left_spans.push(Span::styled("+", Style::default().fg(TEXT_DIM)));
            }
            first = false;
            
            match mode {
                crate::config::ProcessingMode::Merica => {
                    left_spans.push(Span::styled("'merica", Style::default().fg(Color::Rgb(255, 200, 80)).add_modifier(Modifier::BOLD)));
                }
                crate::config::ProcessingMode::Sports => {
                    left_spans.push(Span::styled("sports", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)));
                }
                crate::config::ProcessingMode::AllEnglish => {
                    left_spans.push(Span::styled("en", Style::default().fg(TEXT_PRIMARY).add_modifier(Modifier::BOLD)));
                }
            }
        }
        
        left_spans.push(Span::styled("]", Style::default().fg(TEXT_DIM)));
    }

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
        let time = now.format("%I:%M%p").to_string();
        
        let (active, total, exp) = if let Some(info) = &app.account_info {
            let a = info.active_cons.as_ref().map(clean_val).unwrap_or_else(|| "0".to_string());
            let t = info.max_connections.as_ref().map(clean_val).unwrap_or_else(|| "1".to_string());
            let e = info.exp_date.as_ref().map(clean_val).unwrap_or_else(|| "N/A".to_string());
            (a, t, e)
        } else {
            ("?".to_string(), "?".to_string(), "N/A".to_string())
        };

        let exp_formatted = if let Ok(ts) = exp.parse::<i64>() {
             Utc.timestamp_opt(ts, 0).single().map(|dt| dt.format("%b %d").to_string()).unwrap_or(exp)
        } else {
            exp
        };

        let mut right_spans = Vec::new();
        right_spans.push(Span::styled(&name, Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)));
        right_spans.push(Span::styled(" · ", Style::default().fg(TEXT_DIM)));
        right_spans.push(Span::styled(&time, Style::default().fg(TEXT_SECONDARY)));
        right_spans.push(Span::styled(" · ", Style::default().fg(TEXT_DIM)));
        right_spans.push(Span::styled(format!("exp {}", exp_formatted), Style::default().fg(TEXT_SECONDARY)));
        right_spans.push(Span::styled(" · ", Style::default().fg(TEXT_DIM)));
        right_spans.push(Span::styled(format!("{}/{}", active, total), Style::default().fg(MATRIX_GREEN)));

        let stats = Paragraph::new(Line::from(right_spans))
            .alignment(Alignment::Right)
            .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(DARK_GREEN)));
        f.render_widget(stats, chunks[1]);
    }
}
