use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, List, ListItem, Paragraph},
    Frame,
};
use crate::app::{App, LoginField, InputMode, SettingsState};
use crate::ui::colors::{MATRIX_GREEN, SOFT_GREEN, HIGHLIGHT_BG, TEXT_PRIMARY, TEXT_SECONDARY, TEXT_DIM};

pub fn render_login(f: &mut Frame, app: &App, area: Rect) {
    let title = if app.editing_account_index.is_some() {
        "edit playlist"
    } else {
        "add playlist"
    };

    let title_area = Rect { x: area.x, y: area.y, width: area.width, height: 1 };
    let title_widget = Paragraph::new(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(title, Style::default().fg(SOFT_GREEN).add_modifier(Modifier::BOLD)),
    ]));
    f.render_widget(title_widget, title_area);

    let constraints = vec![
        Constraint::Length(3), // Name (label + value + gap)
        Constraint::Length(3), // URL
        Constraint::Length(3), // User
        Constraint::Length(3), // Pass
        Constraint::Length(3), // EPG
        Constraint::Length(2), // Footer hints
        Constraint::Min(1),    // Error
    ];

    let form_area = Rect {
        x: area.x,
        y: area.y + 2,
        width: area.width,
        height: area.height.saturating_sub(2),
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(constraints)
        .split(form_area);

    let active = &app.login_field_focus;
    let mode = app.input_mode == InputMode::Editing;

    let mut current_chunk = 0;
    f.render_widget(render_input("playlist name", app.input_name.value(), matches!(active, LoginField::Name), mode, app.input_name.visual_cursor(), app.loading_tick), chunks[current_chunk]);
    current_chunk += 1;
    
    f.render_widget(render_input("server url", app.input_url.value(), matches!(active, LoginField::Url), mode, app.input_url.visual_cursor(), app.loading_tick), chunks[current_chunk]);
    current_chunk += 1;

    f.render_widget(render_input("username", app.input_username.value(), matches!(active, LoginField::Username), mode, app.input_username.visual_cursor(), app.loading_tick), chunks[current_chunk]);
    current_chunk += 1;
    
    let mask: String = app.input_password.value().chars().map(|_| '*').collect();
    f.render_widget(render_input("password", &mask, matches!(active, LoginField::Password), mode, app.input_password.visual_cursor(), app.loading_tick), chunks[current_chunk]);
    current_chunk += 1;

    f.render_widget(render_input("epg url (optional)", app.input_epg_url.value(), matches!(active, LoginField::EpgUrl), mode, app.input_epg_url.visual_cursor(), app.loading_tick), chunks[current_chunk]);
    let hints_chunk = current_chunk + 1;
    let error_chunk = current_chunk + 2;
    
    let key_style = Style::default().fg(MATRIX_GREEN);
    let label_style = Style::default().fg(TEXT_SECONDARY);
    
    let hints = if mode {
        Line::from(vec![
            Span::styled("tab", key_style), Span::styled(" next  ", label_style),
            Span::styled("enter", key_style), Span::styled(" submit  ", label_style),
            Span::styled("esc", key_style), Span::styled(" cancel", label_style),
        ])
    } else {
        Line::from(vec![
            Span::styled("↑↓", key_style), Span::styled(" navigate  ", label_style),
            Span::styled("enter", key_style), Span::styled(" edit  ", label_style),
            Span::styled("esc", key_style), Span::styled(" back", label_style),
        ])
    };
    let hints_para = Paragraph::new(hints).alignment(Alignment::Center);
    f.render_widget(hints_para, chunks[hints_chunk]);
    
    if let Some(err) = &app.login_error {
        let error_msg = Paragraph::new(format!(" error: {}", err))
            .style(Style::default().fg(TEXT_PRIMARY).bg(Color::Rgb(80, 0, 0)).add_modifier(Modifier::BOLD));
        f.render_widget(error_msg, chunks[error_chunk]);
    }
}

fn render_input<'a>(label: &'a str, value: &'a str, is_active: bool, is_editing: bool, cursor_pos: usize, tick: u64) -> Paragraph<'a> {
    let (label_style, content_style) = if is_active && is_editing {
        (
            Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD),
            Style::default().fg(TEXT_PRIMARY).add_modifier(Modifier::BOLD),
        )
    } else if is_active {
        (
            Style::default().fg(SOFT_GREEN).add_modifier(Modifier::BOLD),
            Style::default().fg(TEXT_PRIMARY),
        )
    } else {
        (
            Style::default().fg(TEXT_DIM),
            Style::default().fg(TEXT_SECONDARY),
        )
    };

    let mut display_value = if is_active && !is_editing && value.is_empty() {
        "press enter to edit".to_string()
    } else {
        value.to_string()
    };

    if is_active && is_editing && (tick / 15) % 2 == 0 {
        if cursor_pos >= display_value.len() {
            display_value.push('█');
        } else {
            let mut new_val = String::new();
            for (i, c) in display_value.chars().enumerate() {
                if i == cursor_pos { new_val.push('█'); } else { new_val.push(c); }
            }
            display_value = new_val;
        }
    }

    let prompt = if is_active && is_editing { ">_ " } else if is_active { "> " } else { "  " };

    let lines = vec![
        Line::from(Span::styled(format!("  {}", label), label_style)),
        Line::from(vec![
            Span::styled(format!("  {}", prompt), label_style),
            Span::styled(display_value, content_style),
        ]),
    ];

    Paragraph::new(lines)
}

/// Helper to render a settings sub-screen with list + description + hints
fn render_settings_subscreen(
    f: &mut Frame,
    area: Rect,
    title: &str,
    items: Vec<ListItem>,
    list_state: &mut ratatui::widgets::ListState,
    description: &str,
    list_height: u16,
    hints: Line,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(list_height),
            Constraint::Min(3),
            Constraint::Length(2),
        ])
        .split(area);

    let inner_list_area = crate::ui::common::render_composite_block(f, chunks[0], Some(title));
    
    let list = List::new(items)
        .highlight_style(Style::default().bg(HIGHLIGHT_BG).fg(MATRIX_GREEN).add_modifier(Modifier::BOLD))
        .highlight_symbol(" ▎");
    f.render_stateful_widget(list, inner_list_area, list_state);

    if !description.is_empty() {
        let inner_desc = crate::ui::common::render_matrix_box(f, chunks[1], "info", TEXT_DIM);
        let desc_block = Paragraph::new(description)
            .style(Style::default().fg(TEXT_SECONDARY))
            .wrap(ratatui::widgets::Wrap { trim: true });
        f.render_widget(desc_block, inner_desc);
    }

    let hints_para = Paragraph::new(hints).alignment(Alignment::Center);
    f.render_widget(hints_para, chunks[2]);
}

/// Standard navigation hints
fn nav_hints() -> Line<'static> {
    let key_style = Style::default().fg(MATRIX_GREEN);
    let label_style = Style::default().fg(TEXT_SECONDARY);
    let sep_style = Style::default().fg(TEXT_DIM);
    Line::from(vec![
        Span::styled("enter", key_style), Span::styled(" select", label_style),
        Span::styled(" · ", sep_style),
        Span::styled("esc", key_style), Span::styled(" back", label_style),
    ])
}

pub fn render_settings(f: &mut Frame, app: &mut App, area: Rect) {
    if app.current_screen == crate::app::CurrentScreen::TimezoneSettings {
        let items: Vec<ListItem> = app.timezone_list.iter().enumerate().map(|(i, tz)| {
            let is_current = app.config.timezone.as_ref().map(|t| t == tz).unwrap_or(i == 0);
            let prefix = if is_current { "✓ " } else { "  " };
            ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(if is_current { MATRIX_GREEN } else { TEXT_DIM })),
                Span::styled(tz.as_str(), Style::default().fg(if is_current { MATRIX_GREEN } else { TEXT_PRIMARY })),
            ]))
        }).collect();
        
        render_settings_subscreen(f, area, "timezone", items, &mut app.timezone_list_state, "", area.height.saturating_sub(4), nav_hints());
        return;
    }

    match app.settings_state {
        SettingsState::Main => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(10), Constraint::Length(4)])
                .split(area);

            let items: Vec<ListItem> = app.settings_options.iter().map(|s| {
                if let Some(colon_pos) = s.find(':') {
                    let label = &s[..colon_pos + 1];
                    let value = &s[colon_pos + 1..];
                    ListItem::new(Line::from(vec![
                        Span::styled(format!("  {}", label), Style::default().fg(TEXT_PRIMARY)),
                        Span::styled(value.to_string(), Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
                    ]))
                } else if s.contains(" (") && s.ends_with(")") {
                    if let Some(paren_pos) = s.find(" (") {
                        let label = &s[..paren_pos];
                        let value = &s[paren_pos..];
                        ListItem::new(Line::from(vec![
                            Span::styled(format!("  {}", label), Style::default().fg(TEXT_PRIMARY)),
                            Span::styled(value.to_string(), Style::default().fg(MATRIX_GREEN)),
                        ]))
                    } else {
                        ListItem::new(Line::from(Span::styled(format!("  {}", s), Style::default().fg(TEXT_PRIMARY))))
                    }
                } else {
                    ListItem::new(Line::from(Span::styled(format!("  {}", s), Style::default().fg(TEXT_PRIMARY))))
                }
            }).collect();
            let inner_list_area = crate::ui::common::render_composite_block(f, chunks[0], Some("settings"));
            
            let list = List::new(items)
                .highlight_style(Style::default().bg(HIGHLIGHT_BG).fg(MATRIX_GREEN).add_modifier(Modifier::BOLD))
                .highlight_symbol(" ▎");
            f.render_stateful_widget(list, inner_list_area, &mut app.settings_list_state);

            let desc = app.settings_descriptions.get(app.selected_settings_index)
                .map(|s| s.as_str())
                .unwrap_or("");
            let inner_desc = crate::ui::common::render_matrix_box(f, chunks[1], "info", TEXT_DIM);
            let desc_block = Paragraph::new(desc)
                .style(Style::default().fg(TEXT_SECONDARY))
                .wrap(ratatui::widgets::Wrap { trim: true });
            f.render_widget(desc_block, inner_desc);
        }
        SettingsState::ManageAccounts => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(5), Constraint::Length(2)])
                .split(area);

            let accounts: Vec<ListItem> = app.config.accounts.iter().map(|acc| {
                ListItem::new(Line::from(vec![
                    Span::styled(format!("  {} ", acc.name), Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("({})", acc.base_url), Style::default().fg(TEXT_DIM)),
                ]))
            }).collect();

            let inner_area = crate::ui::common::render_matrix_box(f, chunks[0], &format!("playlists ({})", app.config.accounts.len()), SOFT_GREEN);
            let list = List::new(accounts)
                .highlight_style(Style::default().bg(HIGHLIGHT_BG).fg(MATRIX_GREEN).add_modifier(Modifier::BOLD))
                .highlight_symbol(" ▎");
            f.render_stateful_widget(list, inner_area, &mut app.account_list_state);

            let key_style = Style::default().fg(MATRIX_GREEN);
            let label_style = Style::default().fg(TEXT_SECONDARY);
            let hints = Line::from(vec![
                Span::styled("a", key_style), Span::styled(" add · ", label_style),
                Span::styled("enter", key_style), Span::styled(" edit · ", label_style),
                Span::styled("d", Style::default().fg(Color::Rgb(255, 100, 100))), Span::styled(" delete · ", label_style),
                Span::styled("esc", key_style), Span::styled(" back", label_style),
            ]);
            let hints_para = Paragraph::new(hints).alignment(Alignment::Center);
            f.render_widget(hints_para, chunks[1]);
        }
        SettingsState::About => {
            let about_lines: Vec<Line> = app.about_text.lines().map(|line| {
                if line.starts_with("# ") {
                    Line::from(Span::styled(line.trim_start_matches("# ").trim(), Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)))
                } else if line.contains("Built by") || line.contains("www.") {
                    Line::from(Span::styled(line, Style::default().fg(TEXT_PRIMARY)))
                } else {
                    Line::from(Span::styled(line, Style::default().fg(MATRIX_GREEN)))
                }
            }).collect();
            f.render_widget(Clear, area);
            let inner_area = crate::ui::common::render_matrix_box(f, area, "about", SOFT_GREEN);
            let p = Paragraph::new(about_lines)
                .alignment(Alignment::Center)
                .scroll((app.about_scroll, 0));
            f.render_widget(p, inner_area);
        }
        SettingsState::DnsSelection => {
            let providers = crate::config::DnsProvider::all();
            let items: Vec<ListItem> = providers.iter().map(|p| {
                let is_current = *p == app.config.dns_provider;
                ListItem::new(Line::from(vec![
                    Span::styled(if is_current { "✓ " } else { "  " }, Style::default().fg(if is_current { MATRIX_GREEN } else { TEXT_DIM })),
                    Span::styled(p.display_name(), Style::default().fg(if is_current { MATRIX_GREEN } else { TEXT_PRIMARY })),
                ]))
            }).collect();
            
            render_settings_subscreen(f, area, "dns provider", items, &mut app.dns_list_state, "", area.height.saturating_sub(6), nav_hints());
        }
        SettingsState::VideoModeSelection => {
            let modes = vec![
                ("Enhanced", "Interpolation, upscaling, soap opera effect for smoother video"),
                ("MPV Default", "Standard MPV settings with no enhancements"),
            ];
            let items: Vec<ListItem> = modes.iter().enumerate().map(|(i, (name, _))| {
                let is_current = (i == 0 && !app.config.use_default_mpv) || (i == 1 && app.config.use_default_mpv);
                ListItem::new(Line::from(vec![
                    Span::styled(if is_current { "✓ " } else { "  " }, Style::default().fg(if is_current { MATRIX_GREEN } else { TEXT_DIM })),
                    Span::styled(*name, Style::default().fg(if is_current { MATRIX_GREEN } else { TEXT_PRIMARY })),
                ]))
            }).collect();

            let desc = if let Some(idx) = app.video_mode_list_state.selected() {
                modes.get(idx).map(|(_, d)| *d).unwrap_or("")
            } else { "" };
            
            render_settings_subscreen(f, area, "video mode", items, &mut app.video_mode_list_state, desc, 6, nav_hints());
        }
        SettingsState::PlayerEngineSelection => {
            let engines = crate::config::PlayerEngine::all();
            let items: Vec<ListItem> = engines.iter().map(|e| {
                let is_current = *e == app.config.preferred_player;
                ListItem::new(Line::from(vec![
                    Span::styled(if is_current { "✓ " } else { "  " }, Style::default().fg(if is_current { MATRIX_GREEN } else { TEXT_DIM })),
                    Span::styled(e.display_name(), Style::default().fg(if is_current { MATRIX_GREEN } else { TEXT_PRIMARY })),
                ]))
            }).collect();

            let desc = if let Some(idx) = app.player_engine_list_state.selected() {
                match idx {
                    0 => "MPV: High performance, advanced upscaling, best for high-end machines.",
                    1 => "VLC: High stability, optimized for jittery streams and low-end hardware.",
                    _ => ""
                }
            } else { "" };
            
            render_settings_subscreen(f, area, "player engine", items, &mut app.player_engine_list_state, desc, 6, nav_hints());
        }
        SettingsState::PlaylistModeSelection => {
            let modes = crate::config::ProcessingMode::all();
            let mut items: Vec<ListItem> = modes.iter().map(|m| {
                let is_selected = app.config.processing_modes.contains(m);
                let checkbox = if is_selected { "◉ " } else { "○ " };
                ListItem::new(Line::from(vec![
                    Span::styled(checkbox, Style::default().fg(if is_selected { MATRIX_GREEN } else { TEXT_DIM })),
                    Span::styled(m.display_name(), Style::default().fg(if is_selected { MATRIX_GREEN } else { TEXT_PRIMARY })),
                ]))
            }).collect();
            
            items.push(ListItem::new(Line::from(vec![
                Span::styled("  ─── ", Style::default().fg(TEXT_DIM)),
                Span::styled("apply & save", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD))
            ])));
            
            let desc = if let Some(idx) = app.playlist_mode_list_state.selected() {
                match idx {
                    0 => "'merica: Intelligent geo-blocking buffer. Removes international channels from optimized playlists.",
                    1 => "Sports: Prioritizes sports categories and adds league icons for rapid recognition.",
                    2 => "All English: Broadest filter. Retains all content tagged as English (US, UK, CA, AU).",
                    3 => "Save configuration and refresh playlist with selected filters.",
                    _ => ""
                }
            } else { "" };

            let key_style = Style::default().fg(MATRIX_GREEN);
            let label_style = Style::default().fg(TEXT_SECONDARY);
            let sep_style = Style::default().fg(TEXT_DIM);
            let hints = Line::from(vec![
                Span::styled("space", key_style), Span::styled(" toggle", label_style),
                Span::styled(" · ", sep_style),
                Span::styled("enter", key_style), Span::styled(" done", label_style),
            ]);
            
            render_settings_subscreen(f, area, "playlist filters (space to toggle)", items, &mut app.playlist_mode_list_state, desc, 8, hints);
        }
        SettingsState::AutoRefreshSelection => {
            let intervals = vec![
                ("Disabled", "Never auto-refresh playlist data on login"),
                ("Every 6 hours", "Refresh if last sync was more than 6 hours ago"),
                ("Every 12 hours", "Refresh if last sync was more than 12 hours ago (Recommended)"),
                ("Every 24 hours", "Refresh if last sync was more than 24 hours ago"),
                ("Every 48 hours", "Refresh if last sync was more than 48 hours ago"),
            ];
            let items: Vec<ListItem> = intervals.iter().enumerate().map(|(i, (name, _))| {
                let current_idx = match app.config.auto_refresh_hours {
                    0 => 0, 6 => 1, 12 => 2, 24 => 3, 48 => 4, _ => 2
                };
                let is_current = i == current_idx;
                ListItem::new(Line::from(vec![
                    Span::styled(if is_current { "✓ " } else { "  " }, Style::default().fg(if is_current { MATRIX_GREEN } else { TEXT_DIM })),
                    Span::styled(*name, Style::default().fg(if is_current { MATRIX_GREEN } else { TEXT_PRIMARY })),
                ]))
            }).collect();

            let desc = if let Some(idx) = app.auto_refresh_list_state.selected() {
                intervals.get(idx).map(|(_, d)| *d).unwrap_or("")
            } else { "" };
            
            render_settings_subscreen(f, area, "auto-refresh interval", items, &mut app.auto_refresh_list_state, desc, 9, nav_hints());
        }
    }
}
