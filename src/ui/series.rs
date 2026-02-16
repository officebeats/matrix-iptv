use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::Line,
    widgets::{List, ListItem},
    Frame,
};
use crate::app::{App, Pane};
use crate::ui::colors::{MATRIX_GREEN, SOFT_GREEN, DARK_GREEN, HIGHLIGHT_BG, TEXT_PRIMARY, TEXT_DIM};
use crate::ui::utils::calculate_three_column_split;
use crate::parser::{parse_category, parse_stream};
use crate::ui::common::stylize_channel_name;

pub fn render_series_view(f: &mut Frame, app: &mut App, area: Rect) {
    let (cat_width, series_width, episode_width) = calculate_three_column_split(
        &app.series_categories,
        &app.series_streams,
        &app.series_episodes,
        area.width,
    );

    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            ratatui::layout::Constraint::Length(cat_width),
            ratatui::layout::Constraint::Length(series_width),
            ratatui::layout::Constraint::Min(episode_width),
        ])
        .split(area);

    app.area_categories = chunks[0];
    app.area_streams = chunks[1];

    render_series_categories_pane(f, app, chunks[0]);
    render_series_streams_pane(f, app, chunks[1]);
    render_series_episodes_pane(f, app, chunks[2]);
}

fn render_series_categories_pane(f: &mut Frame, app: &mut App, area: Rect) {
    let visible_height = area.height.saturating_sub(2) as usize;
    let total = app.series_categories.len();
    let selected = app.selected_series_category_index;

    let half_window = visible_height / 2;
    let start = if selected > half_window { selected - half_window } else { 0 };
    let end = (start + visible_height + half_window).min(total);
    let adjusted_start = if end == total && end > visible_height + half_window {
        end.saturating_sub(visible_height + half_window)
    } else { start };

    let items: Vec<ListItem> = app.series_categories.iter().enumerate()
        .skip(adjusted_start)
        .take(end - adjusted_start)
        .map(|(_, c)| {
            let parsed = parse_category(&c.category_name);
            let (styled_name, _) = stylize_channel_name(
                &parsed.display_name, parsed.is_vip, false,
                parsed.quality, parsed.content_type, None,
                Style::default().fg(MATRIX_GREEN),
            );
            ListItem::new(Line::from(styled_name))
        }).collect();

    let title = if total == 0 {
        "categories".to_string()
    } else {
        format!("categories ({}/{})", selected.saturating_add(1), total)
    };
    let is_active = app.active_pane == Pane::Categories;
    let border_color = if is_active { SOFT_GREEN } else { DARK_GREEN };
    let inner_area = crate::ui::common::render_matrix_box(f, area, &title, border_color);

    let list = List::new(items)
        .highlight_style(Style::default().bg(HIGHLIGHT_BG).fg(MATRIX_GREEN).add_modifier(Modifier::BOLD))
        .highlight_symbol(" ▎");

    let mut adjusted_state = app.series_category_list_state.clone();
    if adjusted_start > 0 { adjusted_state.select(Some(selected - adjusted_start)); }
    f.render_stateful_widget(list, inner_area, &mut adjusted_state);
}

fn render_series_streams_pane(f: &mut Frame, app: &mut App, area: Rect) {
    let visible_height = area.height.saturating_sub(2) as usize;
    let total = app.series_streams.len();
    let selected = app.selected_series_stream_index;

    let half_window = visible_height / 2;
    let start = if selected > half_window { selected - half_window } else { 0 };
    let end = (start + visible_height + half_window).min(total);
    let adjusted_start = if end == total && end > visible_height + half_window {
        end.saturating_sub(visible_height + half_window)
    } else { start };

    let items: Vec<ListItem> = app.series_streams.iter().enumerate()
        .skip(adjusted_start)
        .take(end - adjusted_start)
        .map(|(_, s)| {
            let parsed = parse_stream(&s.name, app.provider_timezone.as_deref());
            let mut spans = vec![];

            let mut name = parsed.display_name.clone();
            let re_year = regex::Regex::new(r"[\(\[](19|20)\d{2}[\)\]]").unwrap();
            if let Some(mat) = re_year.find(&s.name) {
                let year_clean = mat.as_str().replace('[', "(").replace(']', ")");
                if !name.contains(&year_clean) {
                    name.push_str(" ");
                    name.push_str(&year_clean);
                }
            }

            let (styled_name, _) = stylize_channel_name(
                &name, false, false, parsed.quality, None, None,
                Style::default().fg(TEXT_PRIMARY),
            );
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
        }).collect();

    let title = if total == 0 {
        "series".to_string()
    } else {
        format!("series ({}/{})", selected.saturating_add(1), total)
    };
    let is_active = app.active_pane == Pane::Streams;
    let border_color = if is_active { SOFT_GREEN } else { DARK_GREEN };
    let inner_area = crate::ui::common::render_matrix_box(f, area, &title, border_color);

    let list = List::new(items)
        .highlight_style(Style::default().bg(HIGHLIGHT_BG).fg(MATRIX_GREEN).add_modifier(Modifier::BOLD))
        .highlight_symbol(" ▎");

    let mut adjusted_state = app.series_stream_list_state.clone();
    if adjusted_start > 0 { adjusted_state.select(Some(selected - adjusted_start)); }
    f.render_stateful_widget(list, inner_area, &mut adjusted_state);
}

fn render_series_episodes_pane(f: &mut Frame, app: &mut App, area: Rect) {
    let visible_height = area.height.saturating_sub(2) as usize;
    let total = app.series_episodes.len();
    let selected = app.selected_series_episode_index;

    let half_window = visible_height / 2;
    let start = if selected > half_window { selected - half_window } else { 0 };
    let end = (start + visible_height + half_window).min(total);
    let adjusted_start = if end == total && end > visible_height + half_window {
        end.saturating_sub(visible_height + half_window)
    } else { start };

    let items: Vec<ListItem> = app.series_episodes.iter().enumerate()
        .skip(adjusted_start)
        .take(end - adjusted_start)
        .map(|(_, ep)| {
            let title = ep.title.as_deref().unwrap_or("Untitled");
            let spans = vec![
                ratatui::text::Span::styled(
                    format!("S{:02}E{:02}", ep.season, ep.episode_num),
                    Style::default().fg(MATRIX_GREEN).add_modifier(Modifier::BOLD)
                ),
                ratatui::text::Span::styled(" · ", Style::default().fg(TEXT_DIM)),
                ratatui::text::Span::styled(title.to_string(), Style::default().fg(TEXT_PRIMARY)),
            ];
            ListItem::new(Line::from(spans))
        }).collect();

    let title = if total == 0 {
        "episodes".to_string()
    } else {
        format!("episodes ({}/{})", selected.saturating_add(1), total)
    };
    let is_active = app.active_pane == Pane::Episodes;
    let border_color = if is_active { SOFT_GREEN } else { DARK_GREEN };
    let inner_area = crate::ui::common::render_matrix_box(f, area, &title, border_color);

    let list = List::new(items)
        .highlight_style(Style::default().bg(HIGHLIGHT_BG).fg(MATRIX_GREEN).add_modifier(Modifier::BOLD))
        .highlight_symbol(" ▎");

    let mut adjusted_state = app.series_episode_list_state.clone();
    if adjusted_start > 0 { adjusted_state.select(Some(selected - adjusted_start)); }
    f.render_stateful_widget(list, inner_area, &mut adjusted_state);
}
