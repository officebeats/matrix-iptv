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
        // Global Search Value Prop (Gold)
        spans.push(Span::styled(" Alt+Space ", Style::default().fg(ratatui::style::Color::Yellow).add_modifier(Modifier::BOLD)));
        spans.push(Span::styled("Global Search  ", Style::default().fg(ratatui::style::Color::Yellow)));

        spans.push(Span::styled(" f ", key_style));
        spans.push(Span::styled("Search ", label_style));
        
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
    let playlist_mode = app.config.playlist_mode;
    if playlist_mode != crate::config::PlaylistMode::Default {
        let mut mode_spans = vec![
            Span::styled(" MODE: ", Style::default().fg(DARK_GREEN)),
        ];

        match playlist_mode {
            crate::config::PlaylistMode::Merica => {
                let gold_style = Style::default().fg(ratatui::style::Color::Yellow).add_modifier(Modifier::BOLD);
                mode_spans.extend(vec![
                    Span::styled("'", gold_style),
                    Span::styled("m", Style::default().fg(ratatui::style::Color::Rgb(255, 50, 50)).add_modifier(Modifier::BOLD)),
                    Span::styled("e", Style::default().fg(ratatui::style::Color::White).add_modifier(Modifier::BOLD)),
                    Span::styled("r", Style::default().fg(ratatui::style::Color::LightBlue).add_modifier(Modifier::BOLD)),
                    Span::styled("i", gold_style),
                    Span::styled("c", gold_style),
                    Span::styled("a", gold_style),
                ]);
            }
            crate::config::PlaylistMode::Sports => {
                mode_spans.push(Span::styled("[SPORTS]", Style::default().fg(ratatui::style::Color::Yellow).add_modifier(Modifier::BOLD)));
            }
            crate::config::PlaylistMode::AllEnglish => {
                mode_spans.push(Span::styled("[ALL_ENGLISH]", Style::default().fg(ratatui::style::Color::LightBlue).add_modifier(Modifier::BOLD)));
            }
            crate::config::PlaylistMode::SportsMerica => {
                mode_spans.push(Span::styled("[USA SPORTS]", Style::default().fg(ratatui::style::Color::Rgb(255, 215, 0)).add_modifier(Modifier::BOLD)));
            }
            _ => {}
        }
        
        mode_spans.push(Span::styled(" ", Style::default()));
        
        let right_p = Paragraph::new(Line::from(mode_spans))
            .alignment(Alignment::Right);
        f.render_widget(right_p, area);
    }
}
