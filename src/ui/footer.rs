use ratatui::{
    layout::{Alignment, Rect},
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use crate::app::{App, CurrentScreen, InputMode, SettingsState};
use crate::ui::colors::{MATRIX_GREEN, TEXT_DIM, TEXT_SECONDARY};

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

    let p = Paragraph::new(Line::from(spans)).alignment(Alignment::Left);
    f.render_widget(p, area);
}
