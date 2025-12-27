use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use crate::app::{App, CurrentScreen, InputMode};

pub fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let key_style = Style::default().fg(ratatui::style::Color::Cyan).add_modifier(Modifier::BOLD);
    let label_style = Style::default().fg(ratatui::style::Color::White);

    let mut spans = vec![
        Span::styled(" q ", key_style),
        Span::styled("Quit  ", label_style),
        
        Span::styled(" Esc/Bksp ", key_style),
        Span::styled("Back  ", label_style),

        Span::styled(" ↑↓ ", key_style),
        Span::styled("Move", label_style),
        Span::styled("  ", Style::default()),
    ];

    if app.input_mode == InputMode::Editing {
        spans.push(Span::styled(" Esc ", key_style));
        spans.push(Span::styled("Stop Editing", label_style));
    } else {
        // Global Search with globe icon (🌐)
        spans.push(Span::styled(" Alt+Space ", Style::default().fg(ratatui::style::Color::Yellow).add_modifier(Modifier::BOLD)));
        spans.push(Span::styled("🌐 Search  ", Style::default().fg(ratatui::style::Color::Yellow)));

        spans.push(Span::styled(" f ", key_style));
        spans.push(Span::styled("Search  ", label_style));
        
        spans.push(Span::styled(" h ", key_style));
        spans.push(Span::styled("Help", label_style));
    }

    // Home screen: show Settings key
    if let CurrentScreen::Home = app.current_screen {
        spans.push(Span::styled("  ", Style::default()));
        spans.push(Span::styled(" x ", key_style));
        spans.push(Span::styled("Settings", label_style));
    }

    if let CurrentScreen::Streams | CurrentScreen::Categories = app.current_screen {
        spans.push(Span::styled("  ", Style::default()));
        spans.push(Span::styled(" Enter ", key_style));
        spans.push(Span::styled("Select/Play", label_style));
    }

    // Settings-specific hints
    if let CurrentScreen::Settings | CurrentScreen::TimezoneSettings = app.current_screen {
        spans.push(Span::styled("  ", Style::default()));
        spans.push(Span::styled(" Enter ", key_style));
        spans.push(Span::styled("Select", label_style));
    }

    let left_p = Paragraph::new(Line::from(spans))
        .alignment(Alignment::Left);
    f.render_widget(left_p, area);
}

