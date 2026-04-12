use crate::app::{App, CurrentScreen, InputMode, SettingsState};
use crate::ui::colors::{MATRIX_GREEN, MODERN_BG, SOFT_GREEN, TEXT_DIM, TEXT_SECONDARY};
use crate::ui::loading::get_loading_status_line;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    // If loading, render inline loading status instead of normal hints
    if let Some(loading_line) = get_loading_status_line(app) {
        render_loading_footer(f, app, area, loading_line);
        return;
    }

    render_normal_footer(f, app, area);
}

fn render_loading_footer(f: &mut Frame, _app: &App, area: Rect, loading_line: Line<'static>) {
    use ratatui::symbols::border;
    use ratatui::widgets::{Block, Borders};

    let bar_block = Block::default()
        .borders(Borders::TOP)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(SOFT_GREEN))
        .bg(MODERN_BG);
    let bar_inner = bar_block.inner(area);
    f.render_widget(bar_block, area);

    let p = Paragraph::new(loading_line).alignment(Alignment::Left);
    f.render_widget(p, bar_inner);
}

fn render_normal_footer(f: &mut Frame, app: &App, area: Rect) {
    let key_style = Style::default().fg(MATRIX_GREEN);
    let label_style = Style::default().fg(TEXT_SECONDARY);
    let sep_style = Style::default().fg(TEXT_DIM);

    let mut spans: Vec<Span> = Vec::new();

    // Version — subtle
    let version = format!("v{}", env!("CARGO_PKG_VERSION"));
    spans.push(Span::styled(version, Style::default().fg(TEXT_DIM)));

    macro_rules! hint {
        ($key:expr, $label:expr) => {
            if !spans.is_empty() {
                spans.push(Span::styled(" · ", sep_style));
            }
            spans.push(Span::styled($key, key_style));
            spans.push(Span::styled(concat!(" ", $label), label_style));
        };
    }

    match app.current_screen {
        CurrentScreen::Home => {
            hint!("q", "quit");
            hint!("ctrl+c", "quit");
            hint!("ctrl+l", "redraw");
            hint!("enter", "load");
            hint!("n", "add");
            hint!("e", "edit");
            hint!("d", "del");
            hint!("s", "sports");
            hint!("m", "filter");
            hint!("x", "settings");
            hint!("?", "help");
        }
        CurrentScreen::Login => {
            if app.input_mode == InputMode::Editing {
                hint!("esc", "stop");
                hint!("enter", "save");
            } else {
                hint!("esc", "back");
                hint!("enter", "edit");
            }
        }
        CurrentScreen::Categories
        | CurrentScreen::Streams
        | CurrentScreen::VodCategories
        | CurrentScreen::VodStreams
        | CurrentScreen::SeriesCategories
        | CurrentScreen::SeriesStreams => {
            if app.search_mode {
                hint!("esc", "cancel");
                hint!("enter", "done");
            } else {
                match app.active_pane {
                    crate::app::Pane::Categories => {
                        hint!("esc", "back");
                        hint!("enter", "select");
                        hint!("/", "search");
                        hint!("tab", "streams");
                        hint!("PgDn", "page");
                        hint!("g", "grid/list");
                    }
                    crate::app::Pane::Streams => {
                        hint!("esc", "back");
                        hint!("enter", "play");
                        hint!("/", "search");
                        hint!("PgDn", "page");
                        hint!("v", "fav");
                        hint!("i", "info");
                        hint!("g", "groups");
                        hint!("?", "help");
                    }
                    crate::app::Pane::Episodes => {
                        hint!("esc", "back");
                        hint!("enter", "play");
                        hint!("PgDn", "page");
                        hint!("tab", "series");
                    }
                }
            }
        }
        CurrentScreen::Settings => match app.settings_state {
            SettingsState::ManageAccounts => {
                hint!("esc", "back");
                hint!("a", "add");
                hint!("d", "del");
                hint!("enter", "edit");
            }
            _ => {
                hint!("esc", "back");
                hint!("enter", "select");
            }
        },
        CurrentScreen::TimezoneSettings => {
            hint!("esc", "back");
            hint!("enter", "select");
        }
        CurrentScreen::GlobalSearch => {
            if app.search_mode {
                hint!("esc", "clear");
                hint!("enter", "done");
            } else {
                hint!("esc", "back");
                hint!("enter", "select");
            }
        }
        CurrentScreen::ContentTypeSelection => {
            hint!("esc", "back");
            hint!("enter", "open");
            hint!("↑↓", "navigate");
            hint!("R", "refresh");
        }
        CurrentScreen::GroupManagement => {
            hint!("esc", "back");
            hint!("n", "new");
            hint!("d", "del");
            hint!("enter", "view");
        }
        _ => {
            hint!("q", "quit");
            hint!("ctrl+c", "quit");
            hint!("esc", "back");
        }
    }

    // Right-aligned page info (JiraTUI-style)
    let mut right_spans: Vec<Span> = Vec::new();
    match app.current_screen {
        CurrentScreen::Categories
        | CurrentScreen::Streams
        | CurrentScreen::VodCategories
        | CurrentScreen::VodStreams
        | CurrentScreen::SeriesCategories
        | CurrentScreen::SeriesStreams => {
            if !app.streams.is_empty() && app.active_pane == crate::app::Pane::Streams {
                let page = app.selected_stream_index + 1;
                let total = app.streams.len();
                right_spans.push(Span::styled(
                    format!("─ {}/{} ─", page, total),
                    Style::default().fg(TEXT_DIM),
                ));
            }
        }
        _ => {}
    }

    // Add connection status indicator
    let status_indicator = if app.session.is_connected() {
        Span::styled(" ● ", Style::default().fg(Color::Rgb(0, 255, 65)))
    } else if app.session.state_loading {
        Span::styled(" ◐ ", Style::default().fg(Color::Rgb(255, 200, 80)))
    } else {
        Span::styled(" ○ ", Style::default().fg(TEXT_DIM))
    };
    right_spans.insert(0, status_indicator);

    // Use bordered bottom bar
    use ratatui::symbols::border;
    use ratatui::widgets::{Block, Borders};

    let bar_block = Block::default()
        .borders(Borders::TOP)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(SOFT_GREEN))
        .bg(MODERN_BG);
    let bar_inner = bar_block.inner(area);
    f.render_widget(bar_block, area);

    // Split inner area: left for key hints, right for page indicator
    let bar_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(20)])
        .split(bar_inner);

    let p = Paragraph::new(Line::from(spans)).alignment(Alignment::Left);
    f.render_widget(p, bar_chunks[0]);

    if !right_spans.is_empty() {
        let right_p = Paragraph::new(Line::from(right_spans)).alignment(Alignment::Right);
        f.render_widget(right_p, bar_chunks[1]);
    }
}
