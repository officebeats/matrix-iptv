use crate::app::{App, Pane};
use crate::parser::parse_stream;
use crate::ui::colors::{DARK_GREEN, MATRIX_GREEN, SOFT_GREEN, TEXT_DIM, TEXT_PRIMARY, TEXT_SECONDARY};
use crate::ui::common::stylize_channel_name;
use crate::ui::utils::visible_window;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Cell, List, ListItem, Paragraph, Row, Table, TableState},
    Frame,
};

pub fn render_series_view(f: &mut Frame, app: &mut App, area: Rect) {
    let is_active = true; // Overall view is active
    let border_color = if is_active { SOFT_GREEN } else { DARK_GREEN };

    if app.active_pane == Pane::Categories {
        crate::ui::panes::render_categories_pane(f, app, area, border_color);
    } else if app.active_pane == Pane::Streams {
        // Series List view
        render_series_streams_pane(f, app, area);
    } else {
        // Episodes List view
        // Show Series Name at top and then Episodes? Or split?
        // Let's do a 40/60 split for Series info and Episodes
        let chunks = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([
                ratatui::layout::Constraint::Percentage(40),
                ratatui::layout::Constraint::Percentage(60),
            ])
            .split(area);

        render_series_streams_pane(f, app, chunks[0]);
        render_series_episodes_pane(f, app, chunks[1]);
    }
}

fn render_series_streams_pane(f: &mut Frame, app: &mut App, area: Rect) {
    let visible_height = area.height.saturating_sub(3) as usize;
    let (start, end) = visible_window(
        app.selected_series_stream_index,
        app.series_streams.len(),
        visible_height,
    );
    let selected = app.selected_series_stream_index;

    let items: Vec<Row> = app
        .series_streams
        .iter()
        .enumerate()
        .skip(start)
        .take(end - start)
        .map(|(idx, s)| {
            let (display_name, quality) = if let Some(ref parsed) = s.cached_parsed {
                (parsed.display_name.clone(), parsed.quality)
            } else {
                let p = parse_stream(&s.name, app.session.provider_timezone.as_deref());
                (p.display_name, p.quality)
            };

            // 0. Index Number
            let num_cell = Cell::from(format!("{:>3}", idx + 1)).style(Style::default().fg(TEXT_DIM));

            // 1. Quality Badge
            let (quality_text, quality_color) = match quality {
                Some(crate::parser::Quality::UHD4K) => ("[4K] ", Color::Rgb(255, 0, 255)),
                Some(crate::parser::Quality::FHD) => ("[FHD]", Color::Rgb(255, 215, 0)),
                Some(crate::parser::Quality::HD) => ("[HD] ", Color::Rgb(0, 255, 255)),
                Some(crate::parser::Quality::SD) => ("[SD] ", Color::White),
                None => ("     ", Color::White),
            };
            let quality_cell = Cell::from(quality_text).style(Style::default().fg(quality_color).add_modifier(Modifier::BOLD));

            // 2. Title
            let is_watched = app
                .config
                .recently_watched
                .iter()
                .any(|(id, _)| id == &s.stream_id.to_string());
            let mut title_spans = Vec::new();
            if is_watched {
                title_spans.push(Span::styled("✓ ", Style::default().fg(crate::ui::colors::SOFT_GREEN)));
            }
            title_spans.push(Span::styled(display_name, Style::default().fg(TEXT_PRIMARY)));
            let title_cell = Cell::from(Line::from(title_spans));

            // 3. Status / Rating
            let rating_str = if let Some(rating_f) = s.rating {
                if rating_f > 0.0 {
                    format!("★ {:.1}", rating_f)
                } else {
                    "Episodic".to_string()
                }
            } else {
                "Episodic".to_string()
            };
            let status_cell = Cell::from(rating_str).style(Style::default().fg(Color::Rgb(255, 200, 80)));

            Row::new(vec![num_cell, quality_cell, title_cell, status_cell])
        })
        .collect();

    let title = "series";
    let is_active = app.active_pane == Pane::Streams;
    let border_color = if is_active { SOFT_GREEN } else { DARK_GREEN };
    let inner_area =
        crate::ui::common::render_matrix_box_active(f, area, title, border_color, is_active);

    if app.series_streams.is_empty() {
        let empty_msg = Paragraph::new("No series found")
            .style(Style::default().fg(TEXT_DIM))
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(empty_msg, inner_area);
        return;
    }

    let constraints = [
        Constraint::Length(4),  // #
        Constraint::Length(6),  // QUAL
        Constraint::Fill(3),    // SERIES TITLE
        Constraint::Length(12), // STATUS / RATING
    ];
    let header_style = Style::default().fg(TEXT_SECONDARY).add_modifier(Modifier::BOLD);
    let header = Row::new(vec![
        Cell::from("  #").style(header_style),
        Cell::from("QUAL").style(header_style),
        Cell::from("SERIES TITLE").style(header_style),
        Cell::from("STATUS").style(header_style),
    ])
    .height(1)
    .style(Style::default().fg(TEXT_DIM));

    let table = Table::new(items, constraints)
        .header(header)
        .column_spacing(1)
        .row_highlight_style(crate::ui::panes::list_highlight_style())
        .highlight_symbol("❯ ");

    let mut table_state = TableState::default();
    table_state.select(Some(selected - start));
    f.render_stateful_widget(table, inner_area, &mut table_state);
}

fn render_series_episodes_pane(f: &mut Frame, app: &mut App, area: Rect) {
    let visible_height = area.height.saturating_sub(2) as usize;
    let (start, end) = visible_window(
        app.selected_series_episode_index,
        app.series_episodes.len(),
        visible_height,
    );
    let selected = app.selected_series_episode_index;

    let items: Vec<ListItem> = app
        .series_episodes
        .iter()
        .enumerate()
        .skip(start)
        .take(end - start)
        .map(|(_, ep)| {
            let title = ep.title.as_deref().unwrap_or("Untitled");
            let spans = vec![
                ratatui::text::Span::styled(
                    format!("S{:02}E{:02}", ep.season, ep.episode_num),
                    Style::default()
                        .fg(MATRIX_GREEN)
                        .add_modifier(Modifier::BOLD),
                ),
                ratatui::text::Span::styled(" · ", Style::default().fg(TEXT_DIM)),
                ratatui::text::Span::styled(title.to_string(), Style::default().fg(TEXT_PRIMARY)),
            ];
            ListItem::new(Line::from(spans))
        })
        .collect();

    let title = "episodes";
    let is_active = app.active_pane == Pane::Episodes;
    let border_color = if is_active { SOFT_GREEN } else { DARK_GREEN };
    let inner_area =
        crate::ui::common::render_matrix_box_active(f, area, title, border_color, is_active);

    let list = List::new(items)
        .highlight_style(crate::ui::common::select_list_theme().0)
        .highlight_symbol(crate::ui::common::select_list_theme().1);

    let mut adjusted_state = app.series_episode_list_state;
    adjusted_state.select(Some(selected - start));
    f.render_stateful_widget(list, inner_area, &mut adjusted_state);
}
