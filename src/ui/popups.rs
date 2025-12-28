use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};
use crate::app::{App, CurrentScreen, Guide, Pane};
use crate::ui::colors::{MATRIX_GREEN, DARK_GREEN, BRIGHT_GREEN};
use crate::ui::utils::centered_rect;

pub fn render_help_popup(f: &mut Frame, area: Rect) {
    let area = centered_rect(60, 60, area);
    f.render_widget(Clear, area);
    let inner_area = crate::ui::common::render_composite_block(f, area, Some(" // COMMAND_LEGEND "));

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(8),
            Constraint::Min(0),
        ])
        .split(inner_area);

    let shortcuts = vec![
        "Keyboard Shortcuts:",
        "",
        "  /       - Toggle Search",
        "  Tab     - Switch Panes (Categories/Streams)",
        "  m       - Switch Mode (Live TV / VOD)",
        "  f       - Toggle Favorite",
        "  Enter   - Select / Play",
        "  j / k   - Navigate Down / Up",
        "  q       - Quit",
    ];
    let shortcuts_p = Paragraph::new(shortcuts.join("\n")).style(Style::default().fg(ratatui::style::Color::White));
    f.render_widget(shortcuts_p, chunks[0]);
}

pub fn render_guide_popup(f: &mut Frame, app: &App, area: Rect) {
    if let Some(guide) = app.show_guide {
        let content = match guide {
            Guide::WhatIsApp => include_str!("../content/what_is_this_app.md"),
            Guide::HowToGetPlaylists => include_str!("../content/how_to_get_playlists.md"),
            Guide::WhatIsIptv => include_str!("../content/what_is_iptv.md"),
        };

        let lines: Vec<Line> = content
            .lines()
            .map(|l| {
                if l.is_empty() {
                    return Line::from("");
                }

                if l.starts_with("---") {
                    return Line::from(Span::styled(
                        "─".repeat(area.width as usize),
                        Style::default().fg(DARK_GREEN)
                    ));
                }

                if l.starts_with("# ") {
                    return Line::from(Span::styled(
                        l.trim_start_matches("# ").to_uppercase(),
                        Style::default().fg(BRIGHT_GREEN).add_modifier(Modifier::BOLD)
                    ));
                }
                
                if l.starts_with("## ") {
                    return Line::from(Span::styled(
                        l.trim_start_matches("## ").to_uppercase(),
                        Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)
                    ));
                }

                if l.starts_with("### ") {
                    return Line::from(Span::styled(
                        l.trim_start_matches("### ").to_uppercase(),
                        Style::default().fg(ratatui::style::Color::White).add_modifier(Modifier::BOLD).add_modifier(Modifier::UNDERLINED)
                    ));
                }

                // Robust state-machine based character parser (UTF-8 safe)
                let mut spans = Vec::new();
                let mut chars = l.chars().peekable();
                let mut current_text = String::new();

                while let Some(c) = chars.next() {
                    match c {
                        // Check for bold ** or italic *
                        '*' => {
                            if chars.peek() == Some(&'*') {
                                // Bold detected
                                chars.next(); // Consume second '*'
                                if !current_text.is_empty() {
                                    spans.push(Span::raw(current_text.clone()));
                                    current_text.clear();
                                }
                                let mut bold_content = String::new();
                                while let Some(&nc) = chars.peek() {
                                    if nc == '*' {
                                        chars.next();
                                        if chars.peek() == Some(&'*') {
                                            chars.next();
                                            break;
                                        } else {
                                            bold_content.push('*');
                                        }
                                    } else {
                                        bold_content.push(chars.next().unwrap());
                                    }
                                }
                                spans.push(Span::styled(
                                    bold_content,
                                    Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)
                                ));
                            } else {
                                // Italic detected (single *)
                                if !current_text.is_empty() {
                                    spans.push(Span::raw(current_text.clone()));
                                    current_text.clear();
                                }
                                let mut italic_content = String::new();
                                while let Some(&nc) = chars.peek() {
                                    if nc == '*' {
                                        chars.next();
                                        break;
                                    } else {
                                        italic_content.push(chars.next().unwrap());
                                    }
                                }
                                spans.push(Span::styled(
                                    italic_content,
                                    Style::default().fg(ratatui::style::Color::DarkGray).add_modifier(Modifier::ITALIC)
                                ));
                            }
                        }
                        // Check for inline code `
                        '`' => {
                            if !current_text.is_empty() {
                                spans.push(Span::raw(current_text.clone()));
                                current_text.clear();
                            }
                            let mut code_content = String::new();
                            while let Some(&nc) = chars.peek() {
                                if nc == '`' {
                                    chars.next();
                                    break;
                                } else {
                                    code_content.push(chars.next().unwrap());
                                }
                            }
                            spans.push(Span::styled(
                                code_content,
                                Style::default().fg(ratatui::style::Color::Cyan).bg(ratatui::style::Color::Rgb(20, 20, 20))
                            ));
                        }
                        _ => {
                            current_text.push(c);
                        }
                    }
                }
                
                if !current_text.is_empty() {
                    spans.push(Span::raw(current_text));
                }

                Line::from(spans)
            })
            .collect();

        let area = centered_rect(80, 80, area);
        f.render_widget(Clear, area);
        let inner_area = crate::ui::common::render_composite_block(f, area, Some(" // SYSTEM_PROTOCOLS "));

        let p = Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .scroll((app.guide_scroll, 0));

        f.render_widget(p, inner_area);
    }
}

pub fn render_content_type_selection(f: &mut Frame, app: &mut App, area: Rect) {
    let area = centered_rect(70, 50, area);
    f.render_widget(Clear, area);
    let inner = crate::ui::common::render_composite_block(f, area, Some(" // CHOOSE_PATH "));
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(10),
            Constraint::Min(4),
        ])
        .margin(1)
        .split(inner);

    let title = Paragraph::new("Select Content Type:")
        .alignment(Alignment::Center)
        .style(Style::default().fg(BRIGHT_GREEN).add_modifier(Modifier::BOLD));
    f.render_widget(title, layout[0]);

    let selected = app.selected_content_type_index;
    let items: Vec<ListItem> = vec![
        (0, "(=====)", "LIVE CHANNELS", "[Red Pill]", ratatui::style::Color::Red),
        (1, "(=====)", "MOVIES (VOD)", "[Blue Pill]", ratatui::style::Color::Cyan),
        (2, "(=====)", "SERIES (VOD)", "[White Rabbit]", ratatui::style::Color::White),
    ]
    .into_iter()
    .map(|(i, icon, label, sub, color)| {
        let is_selected = i == selected;
        let icon_style = Style::default().fg(color).add_modifier(Modifier::BOLD);
        let text_style = if is_selected { Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD) } else { Style::default().fg(ratatui::style::Color::White) };
        ListItem::new(Line::from(vec![
            Span::styled(format!("  {} ", icon), icon_style),
            Span::styled(label, text_style),
            Span::styled(format!(" {}", sub), Style::default().fg(color)),
        ]))
    })
    .collect();

    let mut list_state = ratatui::widgets::ListState::default();
    list_state.select(Some(selected));
    f.render_stateful_widget(List::new(items).highlight_symbol(">> "), layout[1], &mut list_state);

    let (quote, color) = match selected {
        0 => ("\"You take the red pill... you stay in Wonderland,\nand I show you how deep the rabbit hole goes.\"", ratatui::style::Color::Red),
        1 => ("\"You take the blue pill... the story ends,\nyou wake up in your bed and believe whatever you want to believe.\"", ratatui::style::Color::Cyan),
        _ => ("\"Follow the white rabbit.\"", ratatui::style::Color::White),
    };

    f.render_widget(Paragraph::new(quote).alignment(Alignment::Center).wrap(Wrap { trim: true }).style(Style::default().fg(color).add_modifier(Modifier::ITALIC)), layout[2]);
}
pub fn render_error_popup(f: &mut Frame, area: Rect, error: &str) {
    let block = Block::default()
        .title(Span::styled(" // SYSTEM_ERROR_OVERRIDE ", Style::default().fg(ratatui::style::Color::Red).add_modifier(Modifier::BOLD)))
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(ratatui::style::Color::Red));

    let area = centered_rect(60, 30, area);
    f.render_widget(Clear, area);
    f.render_widget(block, area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    let error_text = Paragraph::new(error)
        .style(Style::default().fg(ratatui::style::Color::White))
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Center);

    let dismiss_text = Paragraph::new("Press [Esc] to Acknowledge")
        .style(Style::default().fg(DARK_GREEN))
        .alignment(Alignment::Center);

    f.render_widget(error_text, layout[0]);
    f.render_widget(dismiss_text, layout[1]);
}

pub fn render_play_details_popup(f: &mut Frame, app: &App, area: Rect) {
    let area = centered_rect(75, 80, area);
    f.render_widget(Clear, area);
    let inner = crate::ui::common::render_composite_block(f, area, Some(" // CONTENT_CONFIRMATION "));
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1), // Title
            Constraint::Min(0),    // Details
            Constraint::Length(3), // Controls
        ])
        .split(inner);

    if let Some(title) = &app.pending_play_title {
        let display_title = if app.current_screen == CurrentScreen::SeriesStreams && app.active_pane == Pane::Episodes {
            if let Some(stream) = app.series_streams.get(app.selected_series_stream_index) {
                format!("{} - {}", stream.name, title)
            } else {
                title.clone()
            }
        } else {
            title.clone()
        };

        f.render_widget(
            Paragraph::new(Span::styled(display_title.to_uppercase(), Style::default().fg(ratatui::style::Color::Cyan).add_modifier(Modifier::BOLD)))
                .alignment(Alignment::Center),
            chunks[0]
        );
    }

    let mut details = Vec::new();

    // 1. Try episode-specific info if in series/episodes view
    let mut metadata_found = false;
    if app.current_screen == CurrentScreen::SeriesStreams && app.active_pane == Pane::Episodes {
        if !app.series_episodes.is_empty() {
            let ep = &app.series_episodes[app.selected_series_episode_index.min(app.series_episodes.len() - 1)];
            if let Some(info) = &ep.info {
                if let Some(map) = info.as_object() {
                    add_metadata_lines(&mut details, map);
                    metadata_found = true;
                }
            }
        }
    }

    // 2. Fallback/Complement with VOD or Series level info
    if let Some(vod_info) = &app.current_vod_info {
        if let Some(info) = &vod_info.info {
            if let Some(map) = info.as_object() {
                if !metadata_found {
                    add_metadata_lines(&mut details, map);
                    metadata_found = true;
                }
            }
        }
    } else if let Some(series_info) = &app.current_series_info {
        if let Some(info) = &series_info.info {
            if let Some(map) = info.as_object() {
                // For episodes, we might want to append series cast/rating if not already found in episode info
                if !metadata_found {
                    add_metadata_lines(&mut details, map);
                    metadata_found = true;
                } else {
                    // Just add cast if missing from episode info
                    if !details.iter().any(|l| l.spans.iter().any(|s| s.content.contains("CAST:"))) {
                        if let Some(cast) = map.get("cast").and_then(|v| v.as_str()).or_else(|| map.get("actors").and_then(|v| v.as_str())) {
                            details.push(Line::from(vec![
                                Span::styled("CAST:    ", Style::default().fg(ratatui::style::Color::White).add_modifier(Modifier::BOLD)),
                                Span::styled(cast, Style::default().fg(MATRIX_GREEN)),
                            ]));
                        }
                    }
                }
            }
        }
    }

    if !metadata_found {
        details.push(Line::from(vec![
            Span::styled("No additional metadata available.", Style::default().fg(DARK_GREEN).add_modifier(Modifier::ITALIC))
        ]));
    }

    f.render_widget(
        Paragraph::new(details)
            .wrap(Wrap { trim: true })
            .block(Block::default().borders(Borders::NONE)),
        chunks[1]
    );

    let controls = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(" [Enter] ", Style::default().fg(ratatui::style::Color::Black).bg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
            Span::styled(" PLAY NOW   ", Style::default().fg(ratatui::style::Color::White)),
            Span::styled(" [Esc] ", Style::default().fg(ratatui::style::Color::Black).bg(ratatui::style::Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(" ABORT MISSION ", Style::default().fg(ratatui::style::Color::White)),
        ])
    ]).alignment(Alignment::Center);

    f.render_widget(controls, chunks[2]);
}

fn add_metadata_lines(details: &mut Vec<Line>, info: &serde_json::Map<String, serde_json::Value>) {
    // Rating
    if let Some(rating) = info.get("rating").and_then(|v| {
        match v {
            serde_json::Value::String(s) => Some(s.clone()),
            serde_json::Value::Number(n) => Some(n.to_string()),
            _ => None,
        }
    }) {
        details.push(Line::from(vec![
            Span::styled("RATING:  ", Style::default().fg(ratatui::style::Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(format!("⭐ {} / 10", rating), Style::default().fg(ratatui::style::Color::Cyan)),
        ]));
    }

    // Runtime
    if let Some(runtime) = info.get("runtime").and_then(|v| {
        match v {
            serde_json::Value::String(s) => Some(s.clone()),
            serde_json::Value::Number(n) => Some(n.to_string()),
            _ => None,
        }
    }) {
        details.push(Line::from(vec![
            Span::styled("RUNTIME: ", Style::default().fg(ratatui::style::Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{} min", runtime), Style::default().fg(ratatui::style::Color::White)),
        ]));
    }

    // Release Date
    if let Some(date) = info.get("releasedate").and_then(|v| v.as_str()) {
        details.push(Line::from(vec![
            Span::styled("RELEASE: ", Style::default().fg(ratatui::style::Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(date.to_string(), Style::default().fg(ratatui::style::Color::LightYellow)),
        ]));
    }

    // Cast
    if let Some(cast) = info.get("cast").and_then(|v| v.as_str()).or_else(|| info.get("actors").and_then(|v| v.as_str())) {
        details.push(Line::from(vec![
            Span::styled("CAST:    ", Style::default().fg(ratatui::style::Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(cast.to_string(), Style::default().fg(MATRIX_GREEN)),
        ]));
    }

    details.push(Line::from(""));

    // Synopsis
    if let Some(plot) = info.get("plot").and_then(|v| v.as_str()).or_else(|| info.get("description").and_then(|v| v.as_str())) {
        details.push(Line::from(vec![
            Span::styled("SYNOPSIS:", Style::default().fg(ratatui::style::Color::White).add_modifier(Modifier::BOLD)),
        ]));
        details.push(Line::from(vec![
            Span::styled(plot.to_string(), Style::default().fg(MATRIX_GREEN)),
        ]));
    }
}

pub fn render_update_prompt(f: &mut Frame, app: &App, area: Rect) {
    let area_rect = centered_rect(65, 45, area);
    f.render_widget(Clear, area_rect);
    let inner = crate::ui::common::render_composite_block(f, area_rect, Some(" // SYSTEM_UPDATE_DETECTED "));
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(inner);

    let new_version = app.new_version_available.as_deref().unwrap_or("Unknown");
    let current_version = env!("CARGO_PKG_VERSION");

    let text = vec![
        Line::from(vec![
            Span::styled("A newer version of Matrix IPTV is available!", Style::default().fg(ratatui::style::Color::White).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Current Version: ", Style::default().fg(DARK_GREEN)),
            Span::styled(current_version, Style::default().fg(ratatui::style::Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  New Version:     ", Style::default().fg(DARK_GREEN)),
            Span::styled(new_version, Style::default().fg(BRIGHT_GREEN).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("The update will be downloaded and installed automatically.", Style::default().fg(ratatui::style::Color::Gray)),
        ]),
        Line::from(vec![
            Span::styled("You will be returned to the app once the update is complete.", Style::default().fg(ratatui::style::Color::Gray)),
        ]),
    ];

    f.render_widget(Paragraph::new(text).alignment(Alignment::Center).wrap(Wrap { trim: true }), chunks[0]);

    let controls = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(" [Enter/U] ", Style::default().fg(ratatui::style::Color::Black).bg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
            Span::styled(" UPDATE NOW   ", Style::default().fg(ratatui::style::Color::White)),
            Span::styled(" [Esc/L] ", Style::default().fg(ratatui::style::Color::Black).bg(ratatui::style::Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(" LATER ", Style::default().fg(ratatui::style::Color::White)),
        ])
    ]).alignment(Alignment::Center);

    f.render_widget(controls, chunks[1]);
}
