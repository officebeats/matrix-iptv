use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use crate::app::{App, CurrentScreen, InputMode, SettingsState};
use crate::ui::colors::MATRIX_GREEN;

pub fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let key_style = Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD);
    let label_style = Style::default().fg(Color::White);
    let sep_style = Style::default().fg(Color::DarkGray);

    let mut spans = Vec::new();

    macro_rules! push_sep {
        () => {
            if !spans.is_empty() {
                spans.push(Span::styled(" │ ", sep_style));
            }
        };
    }

    // Start with version in the bottom left
    let version = format!("v{}", env!("CARGO_PKG_VERSION"));
    spans.push(Span::styled(version, Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)));
    push_sep!();

    match app.current_screen {
        CurrentScreen::Home => {
            push_sep!();
            spans.push(Span::styled("q", key_style));
            spans.push(Span::styled(": Quit", label_style));
            push_sep!();
            spans.push(Span::styled("Enter", key_style));
            spans.push(Span::styled(": Load", label_style));
            push_sep!();
            spans.push(Span::styled("m", key_style));
            spans.push(Span::styled(": Mode", label_style));
            push_sep!();
            spans.push(Span::styled("x", key_style));
            spans.push(Span::styled(": Settings", label_style));
            push_sep!();
            spans.push(Span::styled("1-3", key_style));
            spans.push(Span::styled(": Help", label_style));
            push_sep!();
            spans.push(Span::styled("n", key_style));
            spans.push(Span::styled(": Add", label_style));
            push_sep!();
            spans.push(Span::styled("e", key_style));
            spans.push(Span::styled(": Edit", label_style));
            push_sep!();
            spans.push(Span::styled("d", key_style));
            spans.push(Span::styled(": Del", label_style));
        }
        CurrentScreen::Login => {
            if app.input_mode == InputMode::Editing {
                push_sep!();
                spans.push(Span::styled("Esc", key_style));
                spans.push(Span::styled(": Stop", label_style));
                push_sep!();
                spans.push(Span::styled("Enter", key_style));
                spans.push(Span::styled(": Save", label_style));
            } else {
                push_sep!();
                spans.push(Span::styled("Esc", key_style));
                spans.push(Span::styled(": Back", label_style));
                push_sep!();
                spans.push(Span::styled("Enter", key_style));
                spans.push(Span::styled(": Edit", label_style));
            }
        }
        CurrentScreen::Categories | CurrentScreen::Streams | 
        CurrentScreen::VodCategories | CurrentScreen::VodStreams |
        CurrentScreen::SeriesCategories | CurrentScreen::SeriesStreams => {
            if app.search_mode {
                push_sep!();
                spans.push(Span::styled("Esc", key_style));
                spans.push(Span::styled(": Stop", label_style));
                push_sep!();
                spans.push(Span::styled("Enter", key_style));
                spans.push(Span::styled(": Done", label_style));
            } else {
                push_sep!();
                spans.push(Span::styled("q", key_style));
                spans.push(Span::styled(": Quit", label_style));
                push_sep!();
                spans.push(Span::styled("Esc", key_style));
                spans.push(Span::styled(": Back", label_style));
                push_sep!();
                spans.push(Span::styled("Enter", key_style));
                spans.push(Span::styled(": Select", label_style));
                push_sep!();
                spans.push(Span::styled("m", key_style));
                spans.push(Span::styled(": Mode", label_style));
                push_sep!();
                spans.push(Span::styled("Ctl+Space", key_style));
                spans.push(Span::styled(": 🔎 ", label_style));
                push_sep!();
                spans.push(Span::styled("f", key_style));
                spans.push(Span::styled(": 🔍", label_style));
                push_sep!();
                spans.push(Span::styled("v", key_style));
                spans.push(Span::styled(": Fav", label_style));
            }
        }
        CurrentScreen::Settings => {
            match app.settings_state {
                SettingsState::ManageAccounts => {
                    push_sep!();
                    spans.push(Span::styled("Esc", key_style));
                    spans.push(Span::styled(": Back", label_style));
                    push_sep!();
                    spans.push(Span::styled("a", key_style));
                    spans.push(Span::styled(": Add", label_style));
                    push_sep!();
                    spans.push(Span::styled("d", key_style));
                    spans.push(Span::styled(": Del", label_style));
                    push_sep!();
                    spans.push(Span::styled("Enter", key_style));
                    spans.push(Span::styled(": Edit", label_style));
                }
                _ => {
                    push_sep!();
                    spans.push(Span::styled("Esc", key_style));
                    spans.push(Span::styled(": Back", label_style));
                    push_sep!();
                    spans.push(Span::styled("Enter", key_style));
                    spans.push(Span::styled(": Select", label_style));
                }
            }
        }
        CurrentScreen::TimezoneSettings => {
            push_sep!();
            spans.push(Span::styled("Esc", key_style));
            spans.push(Span::styled(": Back", label_style));
            push_sep!();
            spans.push(Span::styled("Enter", key_style));
            spans.push(Span::styled(": Select", label_style));
        }
        CurrentScreen::GlobalSearch => {
            if app.search_mode {
                push_sep!();
                spans.push(Span::styled("Esc", key_style));
                spans.push(Span::styled(": Clear", label_style));
                push_sep!();
                spans.push(Span::styled("Enter", key_style));
                spans.push(Span::styled(": Done", label_style));
            } else {
                push_sep!();
                spans.push(Span::styled("Esc", key_style));
                spans.push(Span::styled(": Back", label_style));
                push_sep!();
                spans.push(Span::styled("Enter", key_style));
                spans.push(Span::styled(": Select", label_style));
            }
        }
        CurrentScreen::ContentTypeSelection => {
            push_sep!();
            spans.push(Span::styled("Esc", key_style));
            spans.push(Span::styled(": Back", label_style));
            push_sep!();
            spans.push(Span::styled("1-3", key_style));
            spans.push(Span::styled(": Pick", label_style));
            push_sep!();
            spans.push(Span::styled("Enter", key_style));
            spans.push(Span::styled(": Select", label_style));
            push_sep!();
            spans.push(Span::styled("R", key_style));
            spans.push(Span::styled(": Refresh", label_style));
        }
        _ => {
            push_sep!();
            spans.push(Span::styled("q", key_style));
            spans.push(Span::styled(": Quit", label_style));
            push_sep!();
            spans.push(Span::styled("Esc", key_style));
            spans.push(Span::styled(": Back", label_style));
        }
    }

    let p = Paragraph::new(Line::from(spans)).alignment(Alignment::Left);
    f.render_widget(p, area);
}
