use crate::app::{App, Pane};
use crate::parser::parse_stream;
use crate::ui::colors::{
    DARK_GREEN, HIGHLIGHT_BG, MATRIX_GREEN, SOFT_GREEN, TEXT_DIM, TEXT_PRIMARY,
};
use crate::ui::common::stylize_channel_name;
use crate::ui::utils::visible_window;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::Line,
    widgets::{List, ListItem},
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
    let visible_height = area.height.saturating_sub(2) as usize;
    let (start, end) = visible_window(
        app.selected_series_stream_index,
        app.series_streams.len(),
        visible_height,
    );
    let selected = app.selected_series_stream_index;

    let items: Vec<ListItem> = app
        .series_streams
        .iter()
        .enumerate()
        .skip(start)
        .take(end - start)
        .map(|(_, s)| {
            let mut spans = vec![];

            // O(1) metadata retrieval from pre-computed cache
            let (display_name, quality) = if let Some(ref parsed) = s.cached_parsed {
                (parsed.display_name.clone(), parsed.quality)
            } else {
                let p = parse_stream(&s.name, app.session.provider_timezone.as_deref());
                (p.display_name, p.quality)
            };

            let mut name = display_name;
            let re_year = regex::Regex::new(r"[\(\[](19|20)\d{2}[\)\]]").unwrap();
            if let Some(mat) = re_year.find(&s.name) {
                let year_clean = mat.as_str().replace('[', "(").replace(']', ")");
                if !name.contains(&year_clean) {
                    name.push_str(" ");
                    name.push_str(&year_clean);
                }
            }

            // Check if recently watched (progress indicator)
            let is_watched = app
                .config
                .recently_watched
                .iter()
                .any(|(id, _)| id == &s.stream_id.to_string());
            let prefix = if is_watched { "✓ " } else { "" };
            let prefixed_name = format!("{}{}", prefix, name);

            let (mut styled_name, _) = stylize_channel_name(
                &prefixed_name,
                false,
                false,
                quality,
                None,
                None,
                Style::default().fg(TEXT_PRIMARY),
                true, // show_quality: true because Series doesn't have a QUAL column
            );

            // Colorize the checkmark if it's there
            if is_watched {
                if let Some(first_span) = styled_name.first_mut() {
                    if first_span.content.starts_with("✓") {
                        let check_span = ratatui::text::Span::styled(
                            "✓ ",
                            Style::default().fg(crate::ui::colors::SOFT_GREEN),
                        );
                        first_span.content =
                            std::borrow::Cow::Owned(first_span.content[2..].to_string());

                        let mut new_spans = vec![check_span];
                        new_spans.extend(styled_name);
                        styled_name = new_spans;
                    }
                }
            }

            spans.extend(styled_name);

            if let Some(rating_f) = s.rating {
                if rating_f > 0.0 {
                    let rating_str = format!("{:.1}", rating_f);
                    let rating_color = crate::ui::utils::get_rating_color(&rating_str);
                    spans.push(ratatui::text::Span::styled(
                        format!(" {}", rating_str),
                        Style::default().fg(rating_color),
                    ));
                }
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    let title = "series";
    let is_active = app.active_pane == Pane::Streams;
    let border_color = if is_active { SOFT_GREEN } else { DARK_GREEN };
    let inner_area =
        crate::ui::common::render_matrix_box_active(f, area, &title, border_color, is_active);

    let list = List::new(items)
        .highlight_style(
            Style::default()
                .bg(HIGHLIGHT_BG)
                .fg(MATRIX_GREEN)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" ▎");

    // Convert TableState selection to a temporary ListState for List widget rendering
    let sel = Some(selected - start);
    let mut adjusted_state = ratatui::widgets::ListState::default();
    adjusted_state.select(sel);
    f.render_stateful_widget(list, inner_area, &mut adjusted_state);
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
        crate::ui::common::render_matrix_box_active(f, area, &title, border_color, is_active);

    let list = List::new(items)
        .highlight_style(
            Style::default()
                .bg(HIGHLIGHT_BG)
                .fg(MATRIX_GREEN)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" ▎");

    let mut adjusted_state = app.series_episode_list_state.clone();
    adjusted_state.select(Some(selected - start));
    f.render_stateful_widget(list, inner_area, &mut adjusted_state);
}
