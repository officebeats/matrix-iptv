use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};
use crate::app::{App, LoginField, InputMode, SettingsState};
use crate::ui::colors::{MATRIX_GREEN, DARK_GREEN, BRIGHT_GREEN};

pub fn render_login(f: &mut Frame, app: &App, area: Rect) {
    let title = if app.editing_account_index.is_some() {
        " EDIT PLAYLIST "
    } else {
        " ADD NEW PLAYLIST "
    };
    
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(MATRIX_GREEN));
    f.render_widget(block.clone(), area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // Name
            Constraint::Length(3), // URL
            Constraint::Length(3), // User
            Constraint::Length(3), // Pass
            Constraint::Length(3), // EPG
            Constraint::Length(2), // Footer hints
            Constraint::Min(1),    // Error
        ])
        .split(area);

    let active = &app.login_field_focus;
    let mode = app.input_mode == InputMode::Editing;

    f.render_widget(render_input("Playlist Name", app.input_name.value(), matches!(active, LoginField::Name), mode, app.input_name.visual_cursor(), app.loading_tick), chunks[0]);
    f.render_widget(render_input("Server URL", app.input_url.value(), matches!(active, LoginField::Url), mode, app.input_url.visual_cursor(), app.loading_tick), chunks[1]);
    f.render_widget(render_input("Username", app.input_username.value(), matches!(active, LoginField::Username), mode, app.input_username.visual_cursor(), app.loading_tick), chunks[2]);
    let mask: String = app.input_password.value().chars().map(|_| '*').collect();
    f.render_widget(render_input("Password", &mask, matches!(active, LoginField::Password), mode, app.input_password.visual_cursor(), app.loading_tick), chunks[3]);
    f.render_widget(render_input("EPG URL (Optional)", app.input_epg_url.value(), matches!(active, LoginField::EpgUrl), mode, app.input_epg_url.visual_cursor(), app.loading_tick), chunks[4]);
    
    // Navigation hints footer
    let key_style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);
    let label_style = Style::default().fg(Color::White);
    
    let hints = if mode {
        // Editing mode hints
        Line::from(vec![
            Span::styled(" Tab ", key_style), Span::styled("Next Field  ", label_style),
            Span::styled(" Enter ", key_style), Span::styled("Submit  ", label_style),
            Span::styled(" Esc ", key_style), Span::styled("Cancel", label_style),
        ])
    } else {
        // Navigation mode hints
        Line::from(vec![
            Span::styled(" ↑↓ ", key_style), Span::styled("Navigate  ", label_style),
            Span::styled(" Enter ", key_style), Span::styled("Edit Field  ", label_style),
            Span::styled(" Esc ", key_style), Span::styled("Back", label_style),
        ])
    };
    let hints_para = Paragraph::new(hints).alignment(Alignment::Center);
    f.render_widget(hints_para, chunks[5]);
    
    if let Some(err) = &app.login_error {
        let error_msg = Paragraph::new(format!(" // ERROR_OVERRIDE: {}", err))
            .style(Style::default().fg(Color::Black).bg(Color::Red).add_modifier(Modifier::BOLD));
        f.render_widget(error_msg, chunks[6]);
    }
}

fn render_input<'a>(label: &'a str, value: &'a str, is_active: bool, is_editing: bool, cursor_pos: usize, tick: u64) -> Paragraph<'a> {
    // Clear visual distinction between states:
    // - Normal (not focused): dim green border, gray text
    // - Focused (but not editing): bright cyan border, white text, > indicator
    // - Editing: yellow border, blinking cursor, white text
    
    let (title_style, border_style, content_style, border_type) = if is_active && is_editing {
        // EDITING: Yellow border, thick, white text
        (
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            Style::default().fg(Color::Yellow),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            BorderType::Thick,
        )
    } else if is_active {
        // FOCUSED: Cyan border, double, ready to edit
        (
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            Style::default().fg(Color::Cyan),
            Style::default().fg(Color::White),
            BorderType::Double,
        )
    } else {
        // NOT FOCUSED: Dark green, dim
        (
            Style::default().fg(DARK_GREEN),
            Style::default().fg(DARK_GREEN),
            Style::default().fg(Color::DarkGray),
            BorderType::Rounded,
        )
    };

    // Build display value with cursor if editing
    let mut display_value = if is_active && !is_editing && value.is_empty() {
        "← Press Enter to edit".to_string()
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

    // Add indicator to title for focused field
    let title_text = if is_active && is_editing {
        format!(" {} [EDITING] ", label.to_uppercase())
    } else if is_active {
        format!(" > {} ", label.to_uppercase())
    } else {
        format!("   {} ", label.to_uppercase())
    };

    Paragraph::new(display_value)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(border_type)
            .title(Span::styled(title_text, title_style))
            .border_style(border_style))
        .style(content_style)
}

pub fn render_settings(f: &mut Frame, app: &mut App, area: Rect) {
    // Check if we're in timezone selection mode
    if app.current_screen == crate::app::CurrentScreen::TimezoneSettings {
        // Split area: list on top, hints on bottom
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(5), Constraint::Length(3)])
            .split(area);

        // Timezone dropdown list
        let items: Vec<ListItem> = app.timezone_list.iter().enumerate().map(|(i, tz)| {
            let is_current = app.config.timezone.as_ref().map(|t| t == tz).unwrap_or(i == 0);
            let prefix = if is_current { "✓ " } else { "  " };
            ListItem::new(format!("{}{}", prefix, tz))
        }).collect();
        
        let inner_list_area = crate::ui::common::render_composite_block(f, chunks[0], Some(" SELECT TIMEZONE "));
        
        let list = List::new(items)
            .highlight_style(Style::default().bg(MATRIX_GREEN).fg(Color::Black).add_modifier(Modifier::BOLD))
            .highlight_symbol(" > ");
        f.render_stateful_widget(list, inner_list_area, &mut app.timezone_list_state);

        // Navigation hints
        let hints = Line::from(vec![
            Span::styled(" ↑↓ ", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
            Span::styled("Navigate  ", Style::default().fg(Color::White)),
            Span::styled(" Enter ", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
            Span::styled("Select  ", Style::default().fg(Color::White)),
            Span::styled(" Esc ", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
            Span::styled("Cancel", Style::default().fg(Color::White)),
        ]);
        let hints_para = Paragraph::new(hints).alignment(Alignment::Center);
        f.render_widget(hints_para, chunks[1]);
        return;
    }

    match app.settings_state {
        SettingsState::Main => {
            // Split area: settings list on top, description on bottom
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(10), Constraint::Length(4)])
                .split(area);

            // Create styled list items with color differentiation
            let items: Vec<ListItem> = app.settings_options.iter().map(|s| {
                // Parse settings that have values (e.g., "Label: Value" or "Label (Value)")
                if let Some(colon_pos) = s.find(':') {
                    // Format: "Label: Value"
                    let label = &s[..colon_pos + 1];
                    let value = &s[colon_pos + 1..];
                    ListItem::new(Line::from(vec![
                        Span::styled(format!("  {}", label), Style::default().fg(Color::White)),
                        Span::styled(value.to_string(), Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
                    ]))
                } else if s.contains(" (") && s.ends_with(")") {
                    // Format: "Label (Value)"
                    if let Some(paren_pos) = s.find(" (") {
                        let label = &s[..paren_pos];
                        let value = &s[paren_pos..];
                        ListItem::new(Line::from(vec![
                            Span::styled(format!("  {}", label), Style::default().fg(Color::White)),
                            Span::styled(value.to_string(), Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
                        ]))
                    } else {
                        ListItem::new(Line::from(Span::styled(format!("  {}", s), Style::default().fg(Color::White))))
                    }
                } else {
                    // Plain label without value
                    ListItem::new(Line::from(Span::styled(format!("  {}", s), Style::default().fg(Color::White))))
                }
            }).collect();
            let inner_list_area = crate::ui::common::render_composite_block(f, chunks[0], Some(" // AUTHORIZED_NODES [v3.0.2] "));
            
            let list = List::new(items)
                .highlight_style(Style::default().bg(MATRIX_GREEN).fg(Color::Black).add_modifier(Modifier::BOLD))
                .highlight_symbol(" > ");
            f.render_stateful_widget(list, inner_list_area, &mut app.settings_list_state);

            // Show description for selected setting
            let desc = app.settings_descriptions.get(app.selected_settings_index)
                .map(|s| s.as_str())
                .unwrap_or("");
            let desc_block = Paragraph::new(desc)
                .block(Block::default().title(" INFO ").borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(DARK_GREEN)))
                .style(Style::default().fg(Color::White))
                .wrap(ratatui::widgets::Wrap { trim: true });
            f.render_widget(desc_block, chunks[1]);
        }
        SettingsState::ManageAccounts => {
            // Split area for hints
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(5), Constraint::Length(2)])
                .split(area);

            let accounts: Vec<ListItem> = app.config.accounts.iter().map(|acc| {
                ListItem::new(Line::from(vec![
                    Span::styled(format!("  {} ", acc.name), Style::default().fg(BRIGHT_GREEN).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("({})", acc.base_url), Style::default().fg(Color::DarkGray)),
                ]))
            }).collect();
            let list = List::new(accounts)
                .block(Block::default().title(format!(" MANAGE PLAYLISTS ({}) ", app.config.accounts.len())).borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(MATRIX_GREEN)))
                .highlight_style(Style::default().bg(MATRIX_GREEN).fg(Color::Black).add_modifier(Modifier::BOLD))
                .highlight_symbol(" > ");
            f.render_stateful_widget(list, chunks[0], &mut app.account_list_state);

            // Navigation hints
            let hints = Line::from(vec![
                Span::styled(" a ", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
                Span::styled("Add  ", Style::default().fg(Color::White)),
                Span::styled(" Enter ", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
                Span::styled("Edit  ", Style::default().fg(Color::White)),
                Span::styled(" d ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::styled("Delete  ", Style::default().fg(Color::White)),
                Span::styled(" Esc ", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
                Span::styled("Back", Style::default().fg(Color::White)),
            ]);
            let hints_para = Paragraph::new(hints).alignment(Alignment::Center);
            f.render_widget(hints_para, chunks[1]);
        }
        SettingsState::About => {
            let about_lines: Vec<Line> = app.about_text.lines().map(|line| {
                if line.starts_with("# ") {
                    Line::from(Span::styled(line.trim_start_matches("# ").trim(), Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)))
                } else if line.contains("Built by") || line.contains("www.") {
                    Line::from(Span::styled(line, Style::default().fg(BRIGHT_GREEN)))
                } else {
                    Line::from(Span::styled(line, Style::default().fg(MATRIX_GREEN)))
                }
            }).collect();
            let p = Paragraph::new(about_lines).alignment(Alignment::Center).block(Block::default().title(" // SYSTEM_MANIFEST ").borders(Borders::ALL).border_type(BorderType::Thick).border_style(Style::default().fg(DARK_GREEN))).scroll((app.about_scroll, 0));
            f.render_widget(Clear, area);
            f.render_widget(p, area);
        }
        SettingsState::DnsSelection => {
            // Split area: list on top, hints on bottom
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(5), Constraint::Length(3)])
                .split(area);

            // DNS provider dropdown list
            let providers = crate::config::DnsProvider::all();
            let items: Vec<ListItem> = providers.iter().map(|p| {
                let is_current = *p == app.config.dns_provider;
                let prefix = if is_current { "✓ " } else { "  " };
                ListItem::new(format!("{}{}", prefix, p.display_name()))
            }).collect();
            
            let inner_list_area = crate::ui::common::render_composite_block(f, chunks[0], Some(" SELECT DNS PROVIDER "));
            
            let list = List::new(items)
                .highlight_style(Style::default().bg(MATRIX_GREEN).fg(Color::Black).add_modifier(Modifier::BOLD))
                .highlight_symbol(" > ");
            f.render_stateful_widget(list, inner_list_area, &mut app.dns_list_state);

            // Navigation hints
            let hints = Line::from(vec![
                Span::styled(" ↑↓ ", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
                Span::styled("Navigate  ", Style::default().fg(Color::White)),
                Span::styled(" Enter ", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
                Span::styled("Select  ", Style::default().fg(Color::White)),
                Span::styled(" Esc ", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
                Span::styled("Cancel", Style::default().fg(Color::White)),
            ]);
            let hints_para = Paragraph::new(hints).alignment(Alignment::Center);
            f.render_widget(hints_para, chunks[1]);
        }
        SettingsState::VideoModeSelection => {
            // Split area: list on top, description in middle, hints on bottom
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(6), Constraint::Min(3), Constraint::Length(3)])
                .split(area);

            // Video mode options
            let modes = vec![
                ("Enhanced", "Interpolation, upscaling, soap opera effect for smoother video"),
                ("MPV Default", "Standard MPV settings with no enhancements"),
            ];
            let items: Vec<ListItem> = modes.iter().enumerate().map(|(i, (name, _))| {
                let is_current = (i == 0 && !app.config.use_default_mpv) || (i == 1 && app.config.use_default_mpv);
                let prefix = if is_current { "✓ " } else { "  " };
                ListItem::new(format!("{}{}", prefix, name))
            }).collect();
            
            let inner_list_area = crate::ui::common::render_composite_block(f, chunks[0], Some(" SELECT VIDEO MODE "));
            
            let list = List::new(items)
                .highlight_style(Style::default().bg(MATRIX_GREEN).fg(Color::Black).add_modifier(Modifier::BOLD))
                .highlight_symbol(" > ");
            f.render_stateful_widget(list, inner_list_area, &mut app.video_mode_list_state);

            // Show description for selected mode
            let desc = if let Some(idx) = app.video_mode_list_state.selected() {
                modes.get(idx).map(|(_, d)| *d).unwrap_or("")
            } else {
                ""
            };
            let desc_block = Paragraph::new(desc)
                .block(Block::default().title(" INFO ").borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(DARK_GREEN)))
                .style(Style::default().fg(Color::White))
                .wrap(ratatui::widgets::Wrap { trim: true });
            f.render_widget(desc_block, chunks[1]);

            // Navigation hints
            let hints = Line::from(vec![
                Span::styled(" ↑↓ ", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
                Span::styled("Navigate  ", Style::default().fg(Color::White)),
                Span::styled(" Enter ", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
                Span::styled("Select  ", Style::default().fg(Color::White)),
                Span::styled(" Esc ", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
                Span::styled("Cancel", Style::default().fg(Color::White)),
            ]);
            let hints_para = Paragraph::new(hints).alignment(Alignment::Center);
            f.render_widget(hints_para, chunks[2]);
        }
        SettingsState::PlaylistModeSelection => {
            // Split area: list on top, description in middle, hints on bottom
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(8), Constraint::Min(3), Constraint::Length(3)])
                .split(area);

            // Processing mode options (Multi-select)
            let modes = crate::config::ProcessingMode::all();
            let mut items: Vec<ListItem> = modes.iter().map(|m| {
                let is_selected = app.config.processing_modes.contains(m);
                let checkbox = if is_selected { "[x] " } else { "[ ] " };
                ListItem::new(format!("{}{}", checkbox, m.display_name()))
            }).collect();
            
            // Add Done button
            items.push(ListItem::new(Line::from(vec![
                Span::styled("   [ APPLY & SAVE ]", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
            ])));
            
            let inner_list_area = crate::ui::common::render_composite_block(f, chunks[0], Some(" PLAYLIST FILTRATION (Space to Toggle) "));
            
            let list = List::new(items)
                .highlight_style(Style::default().bg(MATRIX_GREEN).fg(Color::Black).add_modifier(Modifier::BOLD))
                .highlight_symbol(" > ");
            f.render_stateful_widget(list, inner_list_area, &mut app.playlist_mode_list_state);

            // Show description for selected mode
            let desc = if let Some(idx) = app.playlist_mode_list_state.selected() {
                match idx {
                    0 => "'merica: Intelligent geo-blocking buffer. Removes international channels (AR, FR, DE, etc) from optimized playlists.",
                    1 => "Sports: Prioritizes sports categories and adds league icons (NBA, NFL, MLB, NHL) for rapid recognition.",
                    2 => "All English: Broadest filter. Retains all content tagged as English (US, UK, CA, AU).",
                    3 => "Save configuration and refresh playlist with selected filters.",
                    _ => ""
                }
            } else {
                ""
            };
            let desc_block = Paragraph::new(desc)
                .block(Block::default().title(" MODE_MANIFEST ").borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(DARK_GREEN)))
                .style(Style::default().fg(Color::White))
                .wrap(ratatui::widgets::Wrap { trim: true });
            f.render_widget(desc_block, chunks[1]);

            // Navigation hints
            let hints = Line::from(vec![
                Span::styled(" ↑↓ ", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
                Span::styled("Navigate  ", Style::default().fg(Color::White)),
                Span::styled(" Space ", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
                Span::styled("Toggle  ", Style::default().fg(Color::White)),
                Span::styled(" Enter ", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
                Span::styled("Done  ", Style::default().fg(Color::White)),
            ]);
            let hints_para = Paragraph::new(hints).alignment(Alignment::Center);
            f.render_widget(hints_para, chunks[2]);
        }
        SettingsState::AutoRefreshSelection => {
            // Split area: list on top, description in middle, hints on bottom
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(9), Constraint::Min(3), Constraint::Length(3)])
                .split(area);

            // Auto-refresh interval options
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
                let prefix = if is_current { "✓ " } else { "  " };
                ListItem::new(format!("{}{}", prefix, name))
            }).collect();
            
            let inner_list_area = crate::ui::common::render_composite_block(f, chunks[0], Some(" AUTO-REFRESH INTERVAL "));
            
            let list = List::new(items)
                .highlight_style(Style::default().bg(MATRIX_GREEN).fg(Color::Black).add_modifier(Modifier::BOLD))
                .highlight_symbol(" > ");
            f.render_stateful_widget(list, inner_list_area, &mut app.auto_refresh_list_state);

            // Show description for selected interval
            let desc = if let Some(idx) = app.auto_refresh_list_state.selected() {
                intervals.get(idx).map(|(_, d)| *d).unwrap_or("")
            } else {
                ""
            };
            let desc_block = Paragraph::new(desc)
                .block(Block::default().title(" INFO ").borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(DARK_GREEN)))
                .style(Style::default().fg(Color::White))
                .wrap(ratatui::widgets::Wrap { trim: true });
            f.render_widget(desc_block, chunks[1]);

            // Navigation hints
            let hints = Line::from(vec![
                Span::styled(" ↑↓ ", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
                Span::styled("Navigate  ", Style::default().fg(Color::White)),
                Span::styled(" Enter ", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
                Span::styled("Select  ", Style::default().fg(Color::White)),
                Span::styled(" Esc ", Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)),
                Span::styled("Cancel", Style::default().fg(Color::White)),
            ]);
            let hints_para = Paragraph::new(hints).alignment(Alignment::Center);
            f.render_widget(hints_para, chunks[2]);
        }
    }
}
