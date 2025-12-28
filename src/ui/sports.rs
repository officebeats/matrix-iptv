use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph, Wrap},
    Frame,
};
use crate::app::{App, Pane};
use crate::ui::colors::{MATRIX_GREEN, DARK_GREEN, BRIGHT_GREEN};
use crate::ui::common::render_matrix_box;
use crate::sports::get_team_color;

pub fn render_sports_view(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(25), // Categories
            Constraint::Min(40),    // Matches
            Constraint::Length(45), // Details/Links
        ])
        .split(area);

    render_sports_categories_pane(f, app, chunks[0]);
    render_sports_matches_pane(f, app, chunks[1]);
    render_sports_details_pane(f, app, chunks[2]);
}

fn render_sports_categories_pane(f: &mut Frame, app: &mut App, area: Rect) {
    let title = " // SPORTS_UPLINK ";
    let is_active = app.active_pane == Pane::Categories;
    let border_color = if is_active { BRIGHT_GREEN } else { MATRIX_GREEN };

    let items: Vec<ListItem> = app.sports_categories.iter().enumerate().map(|(_, cat)| {
        let display = cat.to_uppercase().replace("-", " ");
        ListItem::new(Line::from(vec![
            Span::styled("⚡ ", Style::default().fg(Color::Yellow)),
            Span::styled(display, Style::default().fg(MATRIX_GREEN)),
        ]))
    }).collect();

    let inner_area = render_matrix_box(f, area, title, border_color);

    let list = List::new(items)
        .highlight_style(Style::default().bg(MATRIX_GREEN).fg(Color::Black).add_modifier(Modifier::BOLD))
        .highlight_symbol(" » ");

    f.render_stateful_widget(list, inner_area, &mut app.sports_category_list_state);
}

fn render_sports_matches_pane(f: &mut Frame, app: &mut App, area: Rect) {
    let category = &app.sports_categories[app.selected_sports_category_index].to_uppercase().replace("-", " ");
    let title = format!(" // {}_PROTOCOLS ({}) ", category, app.sports_matches.len());
    let is_active = app.active_pane == Pane::Streams;
    let border_color = if is_active { BRIGHT_GREEN } else { MATRIX_GREEN };

    let items: Vec<ListItem> = app.sports_matches.iter().map(|m| {
        let mut spans = Vec::new();
        
        // Icon based on category
        let icon = match m.category.as_str() {
            "football" => "⚽ ",
            "basketball" => "🏀 ",
            "baseball" => "⚾ ",
            "hockey" => "🏒 ",
            "ufc" | "mma" => "🥊 ",
            "f1" | "racing" => "🏎️  ",
            "tennis" => "🎾 ",
            _ => "🎮 ",
        };
        spans.push(Span::styled(icon, Style::default()));

        // Process title to highlight teams if we have them
        if let Some(teams) = &m.teams {
            if let Some(home) = &teams.home {
                spans.push(Span::styled(&home.name, Style::default().fg(get_team_color(&home.name))));
                spans.push(Span::styled(" vs ", Style::default().fg(Color::Gray)));
            }
            if let Some(away) = &teams.away {
                spans.push(Span::styled(&away.name, Style::default().fg(get_team_color(&away.name))));
            }
        } else {
            spans.push(Span::styled(&m.title, Style::default().fg(Color::White)));
        }

        if m.popular {
            spans.push(Span::styled(" 🔥", Style::default().fg(Color::Rgb(255, 100, 0))));
        }

        ListItem::new(Line::from(spans))
    }).collect();

    let inner_area = render_matrix_box(f, area, &title, border_color);

    let list = List::new(items)
        .highlight_style(Style::default().bg(MATRIX_GREEN).fg(Color::Black).add_modifier(Modifier::BOLD))
        .highlight_symbol(" » ");

    f.render_stateful_widget(list, inner_area, &mut app.sports_list_state);
}

fn render_sports_details_pane(f: &mut Frame, app: &mut App, area: Rect) {
    let title = " // MATCH_INTELLIGENCE ";
    let inner_area = render_matrix_box(f, area, title, MATRIX_GREEN);

    let selected_idx = app.sports_list_state.selected().unwrap_or(0);
    if let Some(m) = app.sports_matches.get(selected_idx) {
        let mut details = Vec::new();

        // 1. Large Match Title
        details.push(Line::from(vec![
            Span::styled(m.title.to_uppercase(), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]));
        details.push(Line::from(""));

        // 2. Status & Time
        let time_str = chrono::DateTime::from_timestamp(m.date / 1000, 0)
            .map(|dt| dt.format("%b %d, %I:%M %p").to_string())
            .unwrap_or_else(|| "Unknown Time".to_string());

        details.push(Line::from(vec![
            Span::styled("STATUS: ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            if m.popular {
                Span::styled("LIVE / TRENDING", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            } else {
                Span::styled("SCHEDULED", Style::default().fg(Color::Gray))
            }
        ]));
        details.push(Line::from(vec![
            Span::styled("TIME:   ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(time_str, Style::default().fg(MATRIX_GREEN)),
        ]));
        
        details.push(Line::from(""));

        // 3. Teams Info
        if let Some(teams) = &m.teams {
            if let Some(home) = &teams.home {
                details.push(Line::from(vec![
                    Span::styled("HOME: ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                    Span::styled(&home.name, Style::default().fg(get_team_color(&home.name))),
                ]));
            }
            if let Some(away) = &teams.away {
                details.push(Line::from(vec![
                    Span::styled("AWAY: ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                    Span::styled(&away.name, Style::default().fg(get_team_color(&away.name))),
                ]));
            }
            details.push(Line::from(""));
        }

        // 4. Stream Links
        details.push(Line::from(vec![
            Span::styled("AVAILABLE CHANNELS:", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]));

        if app.sports_details_loading {
            details.push(Line::from(vec![
                Span::styled(" Linking with satellites...", Style::default().fg(DARK_GREEN).add_modifier(Modifier::ITALIC)),
            ]));
        } else if app.current_sports_streams.is_empty() {
             details.push(Line::from(vec![
                Span::styled(" No active uplinks found for this event.", Style::default().fg(Color::Red)),
            ]));
        } else {
            for stream in &app.current_sports_streams {
                let quality = if stream.hd { " [HD]" } else { " [SD]" };
                details.push(Line::from(vec![
                    Span::styled(" > ", Style::default().fg(MATRIX_GREEN)),
                    Span::styled(format!("Channel {} ", stream.stream_no), Style::default().fg(Color::White)),
                    Span::styled(format!("({})", stream.language), Style::default().fg(Color::Cyan)),
                    Span::styled(quality, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                ]));
            }
        }

        let p = Paragraph::new(details)
            .wrap(Wrap { trim: true });
        f.render_widget(p, inner_area);
    } else {
        let p = Paragraph::new("Select a protocol to scan for data...")
            .style(Style::default().fg(DARK_GREEN).add_modifier(Modifier::ITALIC));
        f.render_widget(p, inner_area);
    }
}
