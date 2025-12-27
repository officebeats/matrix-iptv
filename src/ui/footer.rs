use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use crate::app::{App, CurrentScreen, InputMode};
use crate::ui::colors::DARK_GREEN;

pub fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let key_style = Style::default().fg(ratatui::style::Color::Cyan).add_modifier(Modifier::BOLD);
    let label_style = Style::default().fg(ratatui::style::Color::White);

    let mut spans = vec![
        Span::styled(" q ", key_style),
        Span::styled("Quit App  ", label_style),
        
        Span::styled(" Esc/Bksp ", key_style),
        Span::styled("Back  ", label_style),

        Span::styled(" \u{2191}\u{2193} ", key_style),
        Span::styled("Move", label_style),
        Span::styled("  ", Style::default()),
    ];

    if app.input_mode == InputMode::Editing {
        spans.push(Span::styled(" Esc ", key_style));
        spans.push(Span::styled("Stop Editing", label_style));
    } else {
        spans.push(Span::styled(" f ", key_style));
        spans.push(Span::styled("Search", label_style));
        spans.push(Span::styled("  ", Style::default()));
        
        spans.push(Span::styled(" h ", key_style));
        spans.push(Span::styled("Help", label_style));
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

    // MODE Indicator (Bottom Right)
    if app.config.american_mode {
        let mode_spans = vec![
            Span::styled(" MODE: ", Style::default().fg(DARK_GREEN)),
            Span::styled("(", Style::default().fg(DARK_GREEN)),
            Span::styled("u", Style::default().fg(ratatui::style::Color::Rgb(255, 50, 50)).add_modifier(Modifier::BOLD)),
            Span::styled("s", Style::default().fg(ratatui::style::Color::White).add_modifier(Modifier::BOLD)),
            Span::styled("a", Style::default().fg(ratatui::style::Color::LightBlue).add_modifier(Modifier::BOLD)),
            Span::styled(") ", Style::default().fg(DARK_GREEN)),
        ];
        
        let right_p = Paragraph::new(Line::from(mode_spans))
            .alignment(Alignment::Right);
        f.render_widget(right_p, area);
    }
}
