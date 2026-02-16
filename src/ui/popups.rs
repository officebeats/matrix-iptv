use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};
use crate::app::{App, CurrentScreen, Guide, Pane};
use crate::ui::colors::{MATRIX_GREEN, SOFT_GREEN, HIGHLIGHT_BG, TEXT_PRIMARY, TEXT_SECONDARY, TEXT_DIM};
use crate::ui::utils::centered_rect;

pub fn render_help_popup(f: &mut Frame, area: Rect) {
    let area = centered_rect(60, 60, area);
    f.render_widget(Clear, area);

    let inner_area = crate::ui::common::render_composite_block(f, area, Some("keyboard shortcuts"));

    let help_text = vec![
        Line::from(vec![
            Span::styled("  navigation", Style::default().fg(TEXT_PRIMARY).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ↑↓ / j/k   ", Style::default().fg(MATRIX_GREEN)),
            Span::styled("navigate items", Style::default().fg(TEXT_SECONDARY)),
        ]),
        Line::from(vec![
            Span::styled("  ←→ / h/l   ", Style::default().fg(MATRIX_GREEN)),
            Span::styled("switch panes", Style::default().fg(TEXT_SECONDARY)),
        ]),
        Line::from(vec![
            Span::styled("  enter       ", Style::default().fg(MATRIX_GREEN)),
            Span::styled("select / confirm", Style::default().fg(TEXT_SECONDARY)),
        ]),
        Line::from(vec![
            Span::styled("  esc         ", Style::default().fg(MATRIX_GREEN)),
            Span::styled("back / cancel", Style::default().fg(TEXT_SECONDARY)),
        ]),
        Line::from(vec![
            Span::styled("  q           ", Style::default().fg(MATRIX_GREEN)),
            Span::styled("quit", Style::default().fg(TEXT_SECONDARY)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  features", Style::default().fg(TEXT_PRIMARY).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ctrl+space  ", Style::default().fg(MATRIX_GREEN)),
            Span::styled("search current view", Style::default().fg(TEXT_SECONDARY)),
        ]),
        Line::from(vec![
            Span::styled("  f           ", Style::default().fg(MATRIX_GREEN)),
            Span::styled("toggle filter active/all", Style::default().fg(TEXT_SECONDARY)),
        ]),
        Line::from(vec![
            Span::styled("  v           ", Style::default().fg(MATRIX_GREEN)),
            Span::styled("toggle favorite", Style::default().fg(TEXT_SECONDARY)),
        ]),
        Line::from(vec![
            Span::styled("  m           ", Style::default().fg(MATRIX_GREEN)),
            Span::styled("change content mode", Style::default().fg(TEXT_SECONDARY)),
        ]),
        Line::from(vec![
            Span::styled("  R           ", Style::default().fg(MATRIX_GREEN)),
            Span::styled("force refresh data", Style::default().fg(TEXT_SECONDARY)),
        ]),
        Line::from(vec![
            Span::styled("  x           ", Style::default().fg(MATRIX_GREEN)),
            Span::styled("settings", Style::default().fg(TEXT_SECONDARY)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  press esc to close", Style::default().fg(TEXT_DIM)),
        ]),
    ];

    f.render_widget(
        Paragraph::new(help_text).wrap(Wrap { trim: true }),
        inner_area,
    );
}

pub fn render_guide_popup(f: &mut Frame, app: &App, area: Rect) {
    if let Some(guide) = &app.show_guide {
        let content = match guide {
            Guide::WhatIsApp => include_str!("../content/what_is_this_app.md"),
            Guide::HowToGetPlaylists => include_str!("../content/how_to_get_playlists.md"),
            Guide::WhatIsIptv => include_str!("../content/what_is_iptv.md"),
        };

        let lines: Vec<Line> = content
            .lines()
            .map(|line| {
                if line.starts_with("# ") {
                    Line::from(Span::styled(
                        line.trim_start_matches("# ").trim(),
                        Style::default().fg(TEXT_PRIMARY).add_modifier(Modifier::BOLD),
                    ))
                } else if line.starts_with("## ") {
                    Line::from(Span::styled(
                        line.trim_start_matches("## ").trim(),
                        Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD),
                    ))
                } else if line.starts_with("### ") {
                    Line::from(Span::styled(
                        line.trim_start_matches("### ").trim(),
                        Style::default().fg(SOFT_GREEN).add_modifier(Modifier::BOLD),
                    ))
                } else if line.starts_with("- ") || line.starts_with("* ") {
                    let content = line.trim_start_matches("- ").trim_start_matches("* ");
                    Line::from(vec![
                        Span::styled("  ▸ ", Style::default().fg(SOFT_GREEN)),
                        Span::raw(content.to_string()),
                    ])
                } else if line.trim().is_empty() {
                    Line::from("")
                } else {
                    // Parse inline markdown
                    let mut spans = Vec::new();
                    let mut current_text = String::new();
                    let mut chars = line.chars().peekable();

                    while let Some(c) = chars.next() {
                        match c {
                            '*' => {
                                if !current_text.is_empty() {
                                    spans.push(Span::styled(current_text.clone(), Style::default().fg(TEXT_SECONDARY)));
                                    current_text.clear();
                                }
                                if chars.peek() == Some(&'*') {
                                    chars.next();
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
                                        Style::default().fg(TEXT_DIM).add_modifier(Modifier::ITALIC)
                                    ));
                                }
                            }
                            '`' => {
                                if !current_text.is_empty() {
                                    spans.push(Span::styled(current_text.clone(), Style::default().fg(TEXT_SECONDARY)));
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
                                    Style::default().fg(MATRIX_GREEN).bg(ratatui::style::Color::Rgb(20, 20, 20))
                                ));
                            }
                            _ => {
                                current_text.push(c);
                            }
                        }
                    }
                    
                    if !current_text.is_empty() {
                        spans.push(Span::styled(current_text, Style::default().fg(TEXT_SECONDARY)));
                    }

                    Line::from(spans)
                }
            })
            .collect();

        let area = centered_rect(80, 80, area);
        f.render_widget(Clear, area);
        let inner_area = crate::ui::common::render_composite_block(f, area, Some("guide"));

        let p = Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .scroll((app.guide_scroll, 0));

        f.render_widget(p, inner_area);
    }
}

pub fn render_content_type_selection(f: &mut Frame, app: &mut App, area: Rect) {
    let area = centered_rect(70, 50, area);
    f.render_widget(Clear, area);
    let inner = crate::ui::common::render_composite_block(f, area, Some("select library"));
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(10),
            Constraint::Min(4),
        ])
        .margin(1)
        .split(inner);

    let title = Paragraph::new("Select content type:")
        .alignment(Alignment::Center)
        .style(Style::default().fg(TEXT_PRIMARY).add_modifier(Modifier::BOLD));
    f.render_widget(title, layout[0]);

    let selected = app.selected_content_type_index;
    let items: Vec<ListItem> = vec![
        (0, "", "Live Channels", "Real-time broadcasts"),
        (1, "", "Movies (VOD)", "On-demand library"),
        (2, "", "TV Series", "Episodic content"),
    ]
    .into_iter()
    .map(|(i, _icon, label, sub)| {
        let is_selected = i == selected;
        let text_style = if is_selected { Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD) } else { Style::default().fg(TEXT_PRIMARY) };
        ListItem::new(Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(label, text_style),
            Span::styled(format!("  · {}", sub), Style::default().fg(TEXT_SECONDARY)),
        ]))
    })
    .collect();

    let mut list_state = ratatui::widgets::ListState::default();
    list_state.select(Some(selected));
    f.render_stateful_widget(
        List::new(items)
            .highlight_style(Style::default().bg(HIGHLIGHT_BG).fg(MATRIX_GREEN).add_modifier(Modifier::BOLD))
            .highlight_symbol(" ▎"),
        layout[1],
        &mut list_state
    );

    let quote = match selected {
        0 => "Access global live TV streams via IPTV protocol.",
        1 => "Browse and watch movies from your provider's VOD library.",
        _ => "Watch TV series and binge-watch seasonal content.",
    };

    f.render_widget(Paragraph::new(quote).alignment(Alignment::Center).wrap(Wrap { trim: true }).style(Style::default().fg(TEXT_SECONDARY)), layout[2]);
}

pub fn render_error_popup(f: &mut Frame, area: Rect, error: &str) {
    let block = Block::default()
        .title(Span::styled(" error ", Style::default().fg(ratatui::style::Color::Rgb(255, 100, 100)).add_modifier(Modifier::BOLD)))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(ratatui::style::Color::Rgb(255, 100, 100)));

    let area = centered_rect(60, 30, area);
    f.render_widget(Clear, area);
    f.render_widget(block, area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    let error_text = Paragraph::new(error)
        .style(Style::default().fg(TEXT_PRIMARY))
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Center);

    let dismiss_text = Paragraph::new("press esc to dismiss")
        .style(Style::default().fg(TEXT_DIM))
        .alignment(Alignment::Center);

    f.render_widget(error_text, layout[0]);
    f.render_widget(dismiss_text, layout[1]);
}

pub fn render_play_details_popup(f: &mut Frame, app: &App, area: Rect) {
    let area = centered_rect(75, 80, area);
    f.render_widget(Clear, area);
    let inner = crate::ui::common::render_composite_block(f, area, Some("confirm"));
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(3),
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
            Paragraph::new(Span::styled(&display_title, Style::default().fg(TEXT_PRIMARY).add_modifier(Modifier::BOLD)))
                .alignment(Alignment::Center),
            chunks[0]
        );
    }

    let mut details = Vec::new();
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
                if !metadata_found {
                    add_metadata_lines(&mut details, map);
                    metadata_found = true;
                } else {
                    if !details.iter().any(|l| l.spans.iter().any(|s| s.content.contains("cast"))) {
                        if let Some(cast) = map.get("cast").and_then(|v| v.as_str()).or_else(|| map.get("actors").and_then(|v| v.as_str())) {
                            details.push(Line::from(vec![
                                Span::styled("cast     ", Style::default().fg(TEXT_SECONDARY)),
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
            Span::styled("No additional metadata available.", Style::default().fg(TEXT_DIM))
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
            Span::styled("enter", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
            Span::styled(" play   ", Style::default().fg(TEXT_PRIMARY)),
            Span::styled("esc", Style::default().fg(ratatui::style::Color::Rgb(255, 100, 100)).add_modifier(Modifier::BOLD)),
            Span::styled(" cancel", Style::default().fg(TEXT_PRIMARY)),
        ])
    ]).alignment(Alignment::Center);

    f.render_widget(controls, chunks[2]);
}

fn add_metadata_lines(details: &mut Vec<Line>, info: &serde_json::Map<String, serde_json::Value>) {
    if let Some(rating) = info.get("rating").and_then(|v| {
        match v {
            serde_json::Value::String(s) => Some(s.clone()),
            serde_json::Value::Number(n) => Some(n.to_string()),
            _ => None,
        }
    }) {
        details.push(Line::from(vec![
            Span::styled("rating   ", Style::default().fg(TEXT_SECONDARY)),
            Span::styled(format!("{} / 10", rating), Style::default().fg(MATRIX_GREEN)),
        ]));
    }

    if let Some(runtime) = info.get("runtime").and_then(|v| {
        match v {
            serde_json::Value::String(s) => Some(s.clone()),
            serde_json::Value::Number(n) => Some(n.to_string()),
            _ => None,
        }
    }) {
        details.push(Line::from(vec![
            Span::styled("runtime  ", Style::default().fg(TEXT_SECONDARY)),
            Span::styled(format!("{} min", runtime), Style::default().fg(TEXT_PRIMARY)),
        ]));
    }

    if let Some(date) = info.get("releasedate").and_then(|v| v.as_str()) {
        details.push(Line::from(vec![
            Span::styled("released ", Style::default().fg(TEXT_SECONDARY)),
            Span::styled(date.to_string(), Style::default().fg(TEXT_PRIMARY)),
        ]));
    }

    if let Some(cast) = info.get("cast").and_then(|v| v.as_str()).or_else(|| info.get("actors").and_then(|v| v.as_str())) {
        details.push(Line::from(vec![
            Span::styled("cast     ", Style::default().fg(TEXT_SECONDARY)),
            Span::styled(cast.to_string(), Style::default().fg(MATRIX_GREEN)),
        ]));
    }

    details.push(Line::from(""));

    if let Some(plot) = info.get("plot").and_then(|v| v.as_str()).or_else(|| info.get("description").and_then(|v| v.as_str())) {
        details.push(Line::from(vec![
            Span::styled(plot.to_string(), Style::default().fg(TEXT_SECONDARY)),
        ]));
    }
}

pub fn render_update_prompt(f: &mut Frame, app: &App, area: Rect) {
    let area_rect = centered_rect(65, 45, area);
    f.render_widget(Clear, area_rect);
    let inner = crate::ui::common::render_composite_block(f, area_rect, Some("update available"));
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(inner);

    let new_version = app.new_version_available.as_deref().unwrap_or("Unknown");
    let current_version = env!("CARGO_PKG_VERSION");

    let text = vec![
        Line::from(vec![
            Span::styled("A newer version of Matrix IPTV is available!", Style::default().fg(TEXT_PRIMARY).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  current  ", Style::default().fg(TEXT_SECONDARY)),
            Span::styled(current_version, Style::default().fg(TEXT_PRIMARY)),
        ]),
        Line::from(vec![
            Span::styled("  new      ", Style::default().fg(TEXT_SECONDARY)),
            Span::styled(new_version, Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("The update will be downloaded and installed automatically.", Style::default().fg(TEXT_SECONDARY)),
        ]),
    ];

    f.render_widget(Paragraph::new(text).alignment(Alignment::Center).wrap(Wrap { trim: true }), chunks[0]);

    let controls = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("enter/u", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
            Span::styled(" update   ", Style::default().fg(TEXT_PRIMARY)),
            Span::styled("esc/l", Style::default().fg(ratatui::style::Color::Rgb(255, 100, 100)).add_modifier(Modifier::BOLD)),
            Span::styled(" later", Style::default().fg(TEXT_PRIMARY)),
        ])
    ]).alignment(Alignment::Center);

    f.render_widget(controls, chunks[1]);
}

pub fn render_cast_picker_popup(f: &mut Frame, app: &App, area: Rect) {
    let area = centered_rect(50, 50, area);
    f.render_widget(Clear, area);
    let inner = crate::ui::common::render_composite_block(f, area, Some("cast to device"));
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(inner);

    let title_text = if app.cast_discovering {
        "⟳ Scanning for Chromecast devices..."
    } else if app.cast_devices.is_empty() {
        "No Chromecast devices found"
    } else {
        "Select a Chromecast device:"
    };
    
    let title = Paragraph::new(title_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(TEXT_PRIMARY).add_modifier(Modifier::BOLD));
    f.render_widget(title, chunks[0]);

    if app.cast_devices.is_empty() {
        let empty_msg = if app.cast_discovering { "Please wait..." } else { "Press R to rescan or check your network" };
        let empty = Paragraph::new(empty_msg)
            .alignment(Alignment::Center)
            .style(Style::default().fg(TEXT_DIM));
        f.render_widget(empty, chunks[1]);
    } else {
        let items: Vec<ListItem> = app.cast_devices
            .iter()
            .map(|device| {
                let model_str = device.model.as_deref().unwrap_or("Chromecast");
                ListItem::new(Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled(&device.name, Style::default().fg(TEXT_PRIMARY)),
                    Span::styled(format!("  ({})", model_str), Style::default().fg(TEXT_DIM)),
                ]))
            })
            .collect();

        let mut list_state = app.cast_device_list_state.clone();
        let list = List::new(items)
            .highlight_style(Style::default().bg(HIGHLIGHT_BG).fg(MATRIX_GREEN).add_modifier(Modifier::BOLD))
            .highlight_symbol(" ▎");
        f.render_stateful_widget(list, chunks[1], &mut list_state);
    }

    let controls = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("enter", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
            Span::styled(" cast   ", Style::default().fg(TEXT_PRIMARY)),
            Span::styled("r", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
            Span::styled(" rescan   ", Style::default().fg(TEXT_PRIMARY)),
            Span::styled("esc", Style::default().fg(ratatui::style::Color::Rgb(255, 100, 100)).add_modifier(Modifier::BOLD)),
            Span::styled(" cancel", Style::default().fg(TEXT_PRIMARY)),
        ])
    ]).alignment(Alignment::Center);

    f.render_widget(controls, chunks[2]);
}
