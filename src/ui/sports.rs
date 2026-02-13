use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph, Wrap},
    Frame,
};
use crate::app::{App, Pane};
use crate::ui::colors::{MATRIX_GREEN, SOFT_GREEN, DARK_GREEN, HIGHLIGHT_BG, TEXT_PRIMARY, TEXT_SECONDARY, TEXT_DIM};
use crate::ui::common::render_matrix_box;
use crate::sports::get_team_color;

pub fn render_sports_view(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(25),
            Constraint::Min(40),
            Constraint::Length(45),
        ])
        .split(area);

    render_sports_categories_pane(f, app, chunks[0]);
    render_sports_matches_pane(f, app, chunks[1]);
    render_sports_details_pane(f, app, chunks[2]);
}

fn render_sports_categories_pane(f: &mut Frame, app: &mut App, area: Rect) {
    let is_active = app.active_pane == Pane::Categories;
    let border_color = if is_active { SOFT_GREEN } else { DARK_GREEN };

    let items: Vec<ListItem> = app.sports_categories.iter().map(|cat| {
        let display = cat.to_uppercase().replace("-", " ");
        ListItem::new(Line::from(vec![
            Span::styled("◆ ", Style::default().fg(SOFT_GREEN)),
            Span::styled(display, Style::default().fg(MATRIX_GREEN)),
        ]))
    }).collect();

    let inner_area = render_matrix_box(f, area, "sports", border_color);

    let list = List::new(items)
        .highlight_style(Style::default().bg(HIGHLIGHT_BG).fg(MATRIX_GREEN).add_modifier(Modifier::BOLD))
        .highlight_symbol(" ▎");

    f.render_stateful_widget(list, inner_area, &mut app.sports_category_list_state);
}

fn render_sports_matches_pane(f: &mut Frame, app: &mut App, area: Rect) {
    let category = &app.sports_categories[app.selected_sports_category_index].to_uppercase().replace("-", " ");
    let title = format!("{} ({})", category.to_lowercase(), app.sports_matches.len());
    let is_active = app.active_pane == Pane::Streams;
    let border_color = if is_active { SOFT_GREEN } else { DARK_GREEN };

    let items: Vec<ListItem> = app.sports_matches.iter().map(|m| {
        let mut spans = Vec::new();
        
        let icon = match m.category.as_str() {
            "football" => "⚽ ",
            "basketball" => "🏀 ",
            "baseball" => "⚾ ",
            "hockey" => "🏒 ",
            "ufc" | "mma" => "🥊 ",
            "f1" | "racing" => "🏎️  ",
            "tennis" => "🎾 ",
            _ => "◆ ",
        };
        spans.push(Span::styled(icon, Style::default()));

        if let Some(teams) = &m.teams {
            if let Some(home) = &teams.home {
                spans.push(Span::styled(&home.name, Style::default().fg(get_team_color(&home.name))));
                spans.push(Span::styled(" vs ", Style::default().fg(TEXT_DIM)));
            }
            if let Some(away) = &teams.away {
                spans.push(Span::styled(&away.name, Style::default().fg(get_team_color(&away.name))));
            }
        } else {
            spans.push(Span::styled(&m.title, Style::default().fg(TEXT_PRIMARY)));
        }

        if m.popular {
            spans.push(Span::styled(" ●", Style::default().fg(Color::Rgb(255, 100, 100))));
        }

        ListItem::new(Line::from(spans))
    }).collect();

    let inner_area = render_matrix_box(f, area, &title, border_color);

    let list = List::new(items)
        .highlight_style(Style::default().bg(HIGHLIGHT_BG).fg(MATRIX_GREEN).add_modifier(Modifier::BOLD))
        .highlight_symbol(" ▎");

    f.render_stateful_widget(list, inner_area, &mut app.sports_list_state);
}

fn render_sports_details_pane(f: &mut Frame, app: &mut App, area: Rect) {
    let inner_area = render_matrix_box(f, area, "match details", DARK_GREEN);

    let selected_idx = app.sports_list_state.selected().unwrap_or(0);
    if let Some(m) = app.sports_matches.get(selected_idx) {
        let mut details = Vec::new();

        details.push(Line::from(vec![
            Span::styled(&m.title, Style::default().fg(TEXT_PRIMARY).add_modifier(Modifier::BOLD)),
        ]));
        details.push(Line::from(""));

        let time_str = chrono::DateTime::from_timestamp(m.date / 1000, 0)
            .map(|dt| dt.format("%b %d, %I:%M %p").to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        details.push(Line::from(vec![
            Span::styled("status  ", Style::default().fg(TEXT_SECONDARY)),
            if m.popular {
                Span::styled("live · trending", Style::default().fg(Color::Rgb(255, 100, 100)).add_modifier(Modifier::BOLD))
            } else {
                Span::styled("scheduled", Style::default().fg(TEXT_SECONDARY))
            }
        ]));
        details.push(Line::from(vec![
            Span::styled("time    ", Style::default().fg(TEXT_SECONDARY)),
            Span::styled(time_str, Style::default().fg(MATRIX_GREEN)),
        ]));
        
        details.push(Line::from(""));

        if let Some(teams) = &m.teams {
            if let Some(home) = &teams.home {
                details.push(Line::from(vec![
                    Span::styled("home    ", Style::default().fg(TEXT_SECONDARY)),
                    Span::styled(&home.name, Style::default().fg(get_team_color(&home.name)).add_modifier(Modifier::BOLD)),
                ]));
            }
            if let Some(away) = &teams.away {
                details.push(Line::from(vec![
                    Span::styled("away    ", Style::default().fg(TEXT_SECONDARY)),
                    Span::styled(&away.name, Style::default().fg(get_team_color(&away.name)).add_modifier(Modifier::BOLD)),
                ]));
            }
            details.push(Line::from(""));
        }

        details.push(Line::from(vec![
            Span::styled("channels", Style::default().fg(TEXT_SECONDARY)),
        ]));

        if app.sports_details_loading {
            details.push(Line::from(vec![
                Span::styled("  scanning...", Style::default().fg(TEXT_DIM)),
            ]));
        } else if app.current_sports_streams.is_empty() {
             details.push(Line::from(vec![
                Span::styled("  no channels found", Style::default().fg(TEXT_DIM)),
            ]));
        } else {
            for stream in &app.current_sports_streams {
                let quality = if stream.hd { "HD" } else { "SD" };
                details.push(Line::from(vec![
                    Span::styled("  ▸ ", Style::default().fg(SOFT_GREEN)),
                    Span::styled(format!("Ch {} ", stream.stream_no), Style::default().fg(TEXT_PRIMARY)),
                    Span::styled(format!("({}) ", stream.language), Style::default().fg(TEXT_SECONDARY)),
                    Span::styled(quality, Style::default().fg(MATRIX_GREEN)),
                ]));
            }
        }

        let p = Paragraph::new(details).wrap(Wrap { trim: true });
        f.render_widget(p, inner_area);
    } else {
        let p = Paragraph::new("Select a match to view details...")
            .style(Style::default().fg(TEXT_DIM));
        f.render_widget(p, inner_area);
    }
}
