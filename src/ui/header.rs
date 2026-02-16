use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use chrono::{Utc, TimeZone};
use chrono_tz::Tz;
use std::str::FromStr;
use crate::app::{App, CurrentScreen};
use crate::ui::colors::{MATRIX_GREEN, TEXT_PRIMARY, TEXT_SECONDARY, TEXT_DIM};

fn clean_val(v: &crate::flex_id::FlexId) -> String {
    v.to_string_value().unwrap_or_default()
}

pub fn render_header(f: &mut Frame, app: &App, area: Rect) {
    use ratatui::widgets::{Block, Borders};
    use ratatui::symbols::border;

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(60),
        ])
        .split(area);

    // Common border style
    let border_style = Style::default().fg(MATRIX_GREEN);
    
    // 1. Search Mode (Prominent Input Box)
    if app.search_mode {
        let search_block = Block::default()
            .borders(Borders::ALL)
            .border_set(border::ROUNDED)
            .border_style(border_style)
            .title(Span::styled(" Search Query ", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)));
        
        let inner = search_block.inner(area);
        f.render_widget(search_block, area);

        let search_text = format!(" >_ {}", app.search_state.query);
        let p = Paragraph::new(search_text)
            .style(Style::default().fg(TEXT_PRIMARY).add_modifier(Modifier::BOLD));
        f.render_widget(p, inner);
        return;
    }

    // 2. Standard Header
    // Left: Breadcrumbs / Mode (bordered)
    let left_block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(border_style)
        .title(" Views "); // JiraTUI style label
    let left_inner = left_block.inner(chunks[0]);
    f.render_widget(left_block, chunks[0]);

    // Right: System Status (bordered)
    let right_block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(border_style)
        .title(" System ");
    let right_inner = right_block.inner(chunks[1]);
    f.render_widget(right_block, chunks[1]);

    // Breadcrumb Content
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

    // Mode keys (Filter-like look)
    if !app.config.processing_modes.is_empty() {
        left_spans.push(Span::styled("  │  ", Style::default().fg(TEXT_DIM))); // Separator like a filter bar
        
        let mut first = true;
        for mode in &app.config.processing_modes {
            if !first {
                left_spans.push(Span::styled(" ", Style::default()));
            }
            first = false;

            match mode {
                crate::config::ProcessingMode::Merica => {
                    left_spans.push(Span::styled("'MERICA", Style::default().fg(Color::Black).bg(Color::Rgb(255, 200, 80)).add_modifier(Modifier::BOLD)));
                }
                crate::config::ProcessingMode::Sports => {
                    left_spans.push(Span::styled("SPORTS", Style::default().fg(Color::Black).bg(MATRIX_GREEN).add_modifier(Modifier::BOLD)));
                }
                crate::config::ProcessingMode::AllEnglish => {
                    left_spans.push(Span::styled("EN", Style::default().fg(Color::Black).bg(TEXT_PRIMARY).add_modifier(Modifier::BOLD)));
                }
            }
        }
    }

    // Background refresh
    if app.background_refresh_active {
        left_spans.push(Span::styled("  ⟳ syncing...", Style::default().fg(Color::Rgb(80, 160, 80))));
    }

    let tabs = Paragraph::new(Line::from(left_spans));
    f.render_widget(tabs, left_inner);

    // System/Account Info Context
    if let Some(_) = &app.current_client {
        let name = app.config.accounts.get(app.selected_account_index).map(|a| a.name.clone()).unwrap_or_else(|| "Unknown".to_string());
        let tz_str = app.config.get_user_timezone();
        let user_tz: Tz = Tz::from_str(&tz_str).unwrap_or(chrono_tz::Europe::London);
        let now = Utc::now().with_timezone(&user_tz);
        let time = now.format("%H:%M").to_string();

        let (_active, _total, exp) = if let Some(info) = &app.account_info {
            let a = info.active_cons.as_ref().map(clean_val).unwrap_or_else(|| "0".to_string());
            let t = info.max_connections.as_ref().map(clean_val).unwrap_or_else(|| "1".to_string());
            let e = info.exp_date.as_ref().map(clean_val).unwrap_or_else(|| "N/A".to_string());
            (a, t, e)
        } else {
            ("?".to_string(), "?".to_string(), "N/A".to_string())
        };

        let exp_formatted = if let Ok(ts) = exp.parse::<i64>() {
             Utc.timestamp_opt(ts, 0).single().map(|dt| dt.format("%Y-%m-%d").to_string()).unwrap_or(exp)
        } else {
            exp
        };

        let mut right_spans = Vec::new();
        // Account Badge
        right_spans.push(Span::styled(format!(" {} ", name), Style::default().fg(Color::Black).bg(MATRIX_GREEN).add_modifier(Modifier::BOLD)));
        right_spans.push(Span::styled(" ", Style::default()));
        
        // Time
        right_spans.push(Span::styled(&time, Style::default().fg(TEXT_PRIMARY).add_modifier(Modifier::BOLD)));
        right_spans.push(Span::styled(" ", Style::default()));
        
        // Exp Date
        right_spans.push(Span::styled(format!("Exp: {}", exp_formatted), Style::default().fg(TEXT_SECONDARY)));

        let stats = Paragraph::new(Line::from(right_spans))
            .alignment(Alignment::Right);
        f.render_widget(stats, right_inner);
    }
}
