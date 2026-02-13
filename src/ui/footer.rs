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

    let mut spans = Vec::new();

    // Version — subtle
    let version = format!("v{}", env!("CARGO_PKG_VERSION"));
    spans.push(Span::styled(version, Style::default().fg(TEXT_DIM)));

    macro_rules! hint {
        ($key:expr, $label:expr) => {
            if !spans.is_empty() {
                spans.push(Span::styled("  ", sep_style));
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
            hint!("1-3", "help");
            hint!("n", "add");
            hint!("e", "edit");
            hint!("d", "del");
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
                hint!("esc", "stop");
                hint!("enter", "done");
            } else {
                hint!("q", "quit");
                hint!("esc", "back");
                hint!("enter", "select");
                hint!("m", "mode");
                hint!("ctrl+space", "search");
                hint!("f", "filter");
                hint!("v", "fav");
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
            hint!("1-3", "pick");
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
