use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use crate::app::{App, CurrentScreen, InputMode, SettingsState};
use crate::ui::colors::{MATRIX_GREEN, SOFT_GREEN, TEXT_DIM, TEXT_SECONDARY};

pub fn render_footer(f: &mut Frame, app: &App, area: Rect) {
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
            hint!("enter", "load");
            hint!("m", "mode");
            hint!("x", "settings");
            hint!("n", "add");
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
        CurrentScreen::Categories | CurrentScreen::Streams |
        CurrentScreen::VodCategories | CurrentScreen::VodStreams |
        CurrentScreen::SeriesCategories | CurrentScreen::SeriesStreams => {
            if app.search_mode {
                hint!("esc", "cancel");
                hint!("enter", "done");
            } else {
                match app.active_pane {
                    crate::app::Pane::Categories => {
                        hint!("esc", "back");
                        hint!("enter", "select");
                        hint!("ctrl+space", "search");
                        hint!("tab", "streams");
                    }
                    crate::app::Pane::Streams => {
                        hint!("esc", "back");
                        hint!("enter", "play");
                        hint!("ctrl+space", "search");
                        hint!("v", "fav");
                        hint!("i", "info");
                        hint!("g", "groups");
                    }
                    crate::app::Pane::Episodes => {
                        hint!("esc", "back");
                        hint!("enter", "play");
                        hint!("tab", "series");
                    }
                }
            }
        }
        CurrentScreen::Settings => {
            match app.settings_state {
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
            }
        }
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
            hint!("enter", "select");
            hint!("R", "refresh");
        }
        _ => {
            hint!("q", "quit");
            hint!("esc", "back");
        }
    }

    // Right-aligned page info (JiraTUI-style)
    let mut right_spans: Vec<Span> = Vec::new();
    match app.current_screen {
        CurrentScreen::Categories | CurrentScreen::Streams |
        CurrentScreen::VodCategories | CurrentScreen::VodStreams |
        CurrentScreen::SeriesCategories | CurrentScreen::SeriesStreams => {
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

    // Use bordered bottom bar
    use ratatui::widgets::{Block, Borders};
    use ratatui::symbols::border;
    
    let bar_block = Block::default()
        .borders(Borders::TOP)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(SOFT_GREEN));
    let bar_inner = bar_block.inner(area);
    f.render_widget(bar_block, area);

    // Split inner area: left for key hints, right for page indicator
    let bar_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(20),
        ])
        .split(bar_inner);

    let p = Paragraph::new(Line::from(spans)).alignment(Alignment::Left);
    f.render_widget(p, bar_chunks[0]);

    if !right_spans.is_empty() {
        let right_p = Paragraph::new(Line::from(right_spans)).alignment(Alignment::Right);
        f.render_widget(right_p, bar_chunks[1]);
    }
}
