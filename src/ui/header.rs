use crate::app::{App, CurrentScreen};
use crate::ui::colors::{MATRIX_GREEN, TEXT_DIM, TEXT_PRIMARY, TEXT_SECONDARY};
use chrono::{TimeZone, Utc};
use chrono_tz::Tz;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use std::str::FromStr;

fn clean_val(v: &crate::flex_id::FlexId) -> String {
    v.to_string_value().unwrap_or_default()
}

pub fn render_header(f: &mut Frame, app: &App, area: Rect) {
    use ratatui::symbols::border;
    use ratatui::widgets::{Block, Borders};

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(60)])
        .split(area);

    // Common border style
    let border_style = Style::default().fg(MATRIX_GREEN);

    // 1. Search Mode (Prominent Input Box)
    if app.search_mode {
        let search_block = Block::default()
            .borders(Borders::ALL)
            .border_set(border::ROUNDED)
            .border_style(border_style)
            .title(Span::styled(
                " Search Query ",
                Style::default()
                    .fg(MATRIX_GREEN)
                    .add_modifier(Modifier::BOLD),
            ));

        let inner = search_block.inner(area);
        f.render_widget(search_block, area);

        let search_text = format!(" >_ {}", app.search_state.query);
        let p = Paragraph::new(search_text).style(
            Style::default()
                .fg(TEXT_PRIMARY)
                .add_modifier(Modifier::BOLD),
        );
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

    let mut left_spans: Vec<Span> = Vec::new();

    // Breadcrumb Content setup
    let mut add_breadcrumb = |text: &str, is_active: bool| {
        if !left_spans.is_empty() {
            left_spans.push(Span::styled(" › ", Style::default().fg(TEXT_DIM)));
        }
        if is_active {
            left_spans.push(Span::styled(
                text.to_string(),
                Style::default()
                    .fg(MATRIX_GREEN)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            left_spans.push(Span::styled(
                text.to_string(),
                Style::default().fg(TEXT_SECONDARY),
            ));
        }
    };

    match app.current_screen {
        CurrentScreen::Home => add_breadcrumb("home", true),
        CurrentScreen::Login => {
            add_breadcrumb("home", false);
            add_breadcrumb("add playlist", true);
        }
        CurrentScreen::GroupManagement => {
            add_breadcrumb("home", false);
            add_breadcrumb("groups", true);
        }
        CurrentScreen::Categories => {
            add_breadcrumb("home", false);
            add_breadcrumb("tv", true);
        }
        CurrentScreen::Streams => {
            add_breadcrumb("home", false);
            add_breadcrumb("tv", false);
            add_breadcrumb("streams", false);
            if let Some(cat) = app.categories.get(app.selected_category_index) {
                add_breadcrumb(&cat.category_name, true);
            }
        }
        CurrentScreen::VodCategories => {
            add_breadcrumb("home", false);
            add_breadcrumb("movies", true);
        }
        CurrentScreen::VodStreams => {
            add_breadcrumb("home", false);
            add_breadcrumb("movies", false);
            add_breadcrumb("browse", false);
            if let Some(cat) = app.vod_categories.get(app.selected_vod_category_index) {
                add_breadcrumb(&cat.category_name, true);
            }
        }
        CurrentScreen::SeriesCategories => {
            add_breadcrumb("home", false);
            add_breadcrumb("series", true);
        }
        CurrentScreen::SeriesStreams => {
            add_breadcrumb("home", false);
            add_breadcrumb("series", false);
            add_breadcrumb("browse", false);
            if let Some(cat) = app
                .series_categories
                .get(app.selected_series_category_index)
            {
                add_breadcrumb(&cat.category_name, true);
            }
        }
        CurrentScreen::Settings => {
            add_breadcrumb("home", false);
            add_breadcrumb("settings", true);
        }
        CurrentScreen::SportsDashboard => {
            add_breadcrumb("home", false);
            add_breadcrumb("sports", true);
        }
        CurrentScreen::GlobalSearch => {
            add_breadcrumb("home", false);
            add_breadcrumb("search", true);
        }
        _ => add_breadcrumb("matrix-iptv", true),
    }

    // Background refresh
    if app.session.background_refresh_active {
        left_spans.push(Span::styled(
            "  ⟳ syncing...",
            Style::default().fg(Color::Rgb(80, 160, 80)),
        ));
    }

    let tabs = Paragraph::new(Line::from(left_spans));
    f.render_widget(tabs, left_inner);

    // System/Account Info Context
    if app.session.current_client.is_some() {
        let name = app
            .config
            .accounts
            .get(app.session.selected_account_index)
            .map(|a| a.name.clone())
            .unwrap_or_else(|| "Unknown".to_string());
        let tz_str = app.config.get_user_timezone();
        let user_tz: Tz = Tz::from_str(&tz_str).unwrap_or(chrono_tz::Europe::London);
        let now = Utc::now().with_timezone(&user_tz);
        let time = now.format("%I:%M %p").to_string();

        let (_active, _total, exp) = if let Some(info) = &app.session.account_info {
            let a = info
                .active_cons
                .as_ref()
                .map(clean_val)
                .unwrap_or_else(|| "0".to_string());
            let t = info
                .max_connections
                .as_ref()
                .map(clean_val)
                .unwrap_or_else(|| "1".to_string());
            let e = info
                .exp_date
                .as_ref()
                .map(clean_val)
                .unwrap_or_else(|| "N/A".to_string());
            (a, t, e)
        } else {
            ("?".to_string(), "?".to_string(), "N/A".to_string())
        };

        let exp_formatted = if let Ok(ts) = exp.parse::<i64>() {
            Utc.timestamp_opt(ts, 0)
                .single()
                .map(|dt| dt.format("%Y-%m-%d").to_string())
                .unwrap_or(exp)
        } else {
            exp
        };

        let mut right_spans = Vec::new();

        // Mode keys (Filter-like look) - Moved to right side
        if !app.config.processing_modes.is_empty() {
            let mut first = true;
            for mode in &app.config.processing_modes {
                if !first {
                    right_spans.push(Span::styled(" ", Style::default()));
                }
                first = false;

                match mode {
                    crate::config::ProcessingMode::Merica => {
                        right_spans.push(Span::styled(
                            "'MERICA",
                            Style::default()
                                .fg(Color::Rgb(0, 0, 0))
                                .bg(Color::Rgb(255, 200, 80))
                                .add_modifier(Modifier::BOLD),
                        ));
                    }
                    crate::config::ProcessingMode::Sports => {
                        right_spans.push(Span::styled(
                            "SPORTS",
                            Style::default()
                                .fg(Color::Rgb(0, 0, 0))
                                .bg(MATRIX_GREEN)
                                .add_modifier(Modifier::BOLD),
                        ));
                    }
                    crate::config::ProcessingMode::AllEnglish => {
                        right_spans.push(Span::styled(
                            "EN",
                            Style::default()
                                .fg(Color::Rgb(0, 0, 0))
                                .bg(TEXT_PRIMARY)
                                .add_modifier(Modifier::BOLD),
                        ));
                    }
                }
            }
            right_spans.push(Span::styled("  │  ", Style::default().fg(TEXT_DIM)));
        }

        // Account Badge
        right_spans.push(Span::styled(
            format!(" {} ", name),
            Style::default()
                .fg(Color::Rgb(0, 0, 0))
                .bg(MATRIX_GREEN)
                .add_modifier(Modifier::BOLD),
        ));
        right_spans.push(Span::styled(" ", Style::default()));

        // Time
        right_spans.push(Span::styled(
            &time,
            Style::default()
                .fg(TEXT_PRIMARY)
                .add_modifier(Modifier::BOLD),
        ));
        right_spans.push(Span::styled(" ", Style::default()));

        // Exp Date
        right_spans.push(Span::styled(
            format!("Exp: {}", exp_formatted),
            Style::default().fg(TEXT_SECONDARY),
        ));

        let stats = Paragraph::new(Line::from(right_spans)).alignment(Alignment::Right);
        f.render_widget(stats, right_inner);
    }
}
