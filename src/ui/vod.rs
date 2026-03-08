use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{List, ListItem, Paragraph, Wrap},
    Frame,
};
use crate::app::{App, Pane};
use crate::ui::colors::{MATRIX_GREEN, SOFT_GREEN, DARK_GREEN, HIGHLIGHT_BG, TEXT_PRIMARY, TEXT_SECONDARY, TEXT_DIM};
use crate::parser::{parse_category, parse_stream};
use crate::ui::common::stylize_channel_name;
use crate::ui::utils::calculate_vod_three_column_split;

/// Strip provider-prefixed country/region codes from a title.
/// e.g.  "EN| The Arborist"        → "The Arborist"
///        "MULTI-LANG| Iron Man"   → "Iron Man"
///        "No prefix"              → "No prefix"  (unchanged)
fn strip_provider_prefix(title: &str) -> &str {
    if let Some(idx) = title.find('|') {
        // Only strip if the prefix is ≤ 15 chars (avoid stripping mid-title pipes)
        if idx <= 15 {
            return title[idx + 1..].trim();
        }
    }
    title
}

/// Truncate a string to `max_chars` code-units, appending `…` when truncated.
fn truncate_to(s: &str, max_chars: usize) -> String {
    let count = s.chars().count();
    if count <= max_chars {
        s.to_string()
    } else if max_chars > 1 {
        let truncated: String = s.chars().take(max_chars - 1).collect();
        format!("{}…", truncated)
    } else {
        "…".to_string()
    }
}

pub fn render_vod_view(f: &mut Frame, app: &mut App, area: Rect) {
    let (cat_width, stream_width, details_width) = calculate_vod_three_column_split(
        &app.vod_categories,
        &app.vod_streams,
        area.width,
    );

    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            ratatui::layout::Constraint::Length(cat_width),
            ratatui::layout::Constraint::Length(stream_width),
            ratatui::layout::Constraint::Min(details_width),
        ])
        .split(area);

    app.area_categories = chunks[0];
    app.area_streams = chunks[1];

    render_vod_categories_pane(f, app, chunks[0]);
    render_vod_streams_pane(f, app, chunks[1]);
    render_vod_details_pane(f, app, chunks[2]);
}

pub fn render_vod_categories_pane(f: &mut Frame, app: &mut App, area: Rect) {
    let visible_height = area.height.saturating_sub(2) as usize;
    let total = app.vod_categories.len();
    let selected = app.selected_vod_category_index;

    let half_window = visible_height / 2;
    let start = if selected > half_window { selected - half_window } else { 0 };
    let end = (start + visible_height + half_window).min(total);
    let adjusted_start = if end == total && end > visible_height + half_window {
        end.saturating_sub(visible_height + half_window)
    } else { start };

    // Inner width for truncation: border left/right (2) + highlight symbol (2)
    let inner_w = area.width.saturating_sub(4) as usize;

    let items: Vec<ListItem> = app.vod_categories.iter().enumerate()
        .skip(adjusted_start)
        .take(end - adjusted_start)
        .map(|(_, c)| {
            let parsed = parse_category(&c.category_name);
            let display = truncate_to(&parsed.display_name, inner_w);
            let (styled_name, _) = stylize_channel_name(
                &display, parsed.is_vip, false,
                parsed.quality, parsed.content_type, None,
                Style::default().fg(MATRIX_GREEN),
            );
            ListItem::new(Line::from(styled_name))
        }).collect();

    let title = if total == 0 {
        "movies".to_string()
    } else {
        format!("movies ({}/{})", selected.saturating_add(1), total)
    };
    let is_active = app.active_pane == Pane::Categories;
    let border_color = if is_active { SOFT_GREEN } else { DARK_GREEN };
    let inner_area = crate::ui::common::render_matrix_box_active(f, area, &title, border_color, is_active);

    let list = List::new(items)
        .highlight_style(Style::default().bg(HIGHLIGHT_BG).fg(MATRIX_GREEN).add_modifier(Modifier::BOLD))
        .highlight_symbol(" ▎");

    let mut adjusted_state = app.vod_category_list_state.clone();
    if adjusted_start > 0 { adjusted_state.select(Some(selected - adjusted_start)); }
    f.render_stateful_widget(list, inner_area, &mut adjusted_state);
}

pub fn render_vod_streams_pane(f: &mut Frame, app: &mut App, area: Rect) {
    let visible_height = area.height.saturating_sub(2) as usize;
    let total = app.vod_streams.len();
    let selected = app.selected_vod_stream_index;

    let half_window = visible_height / 2;
    let start = if selected > half_window { selected - half_window } else { 0 };
    let end = (start + visible_height + half_window).min(total);
    let adjusted_start = if end == total && end > visible_height + half_window {
        end.saturating_sub(visible_height + half_window)
    } else { start };

    // Usable inner width = column width - 2 borders - 2 highlight symbol - 1 padding
    let inner_w = area.width.saturating_sub(5) as usize;

    let items: Vec<ListItem> = app.vod_streams.iter().enumerate()
        .skip(adjusted_start)
        .take(end - adjusted_start)
        .map(|(_, s)| {
            let parsed = parse_stream(&s.name, app.provider_timezone.as_deref());
            let mut spans = vec![];

            // Build display name (cleaned + optional year)
            let mut name = parsed.display_name.clone();
            let re_year = regex::Regex::new(r"[\(\[](19|20)\d{2}[\)\]]").unwrap();
            if let Some(mat) = re_year.find(&s.name) {
                let year_clean = mat.as_str().replace('[', "(").replace(']', ")");
                if !name.contains(&year_clean) {
                    name.push(' ');
                    name.push_str(&year_clean);
                }
            }

            // Check if recently watched (progress indicator)
            let is_watched = app.config.recently_watched.iter().any(|(id, _)| id == &s.stream_id.to_string());
            let prefix = if is_watched { "✓ "} else { "" };
            let prefixed_name = format!("{}{}", prefix, name);

            // Reserve space for rating — " 8.5" = 4 chars
            let has_rating = s.rating.map(|r| r > 0.0).unwrap_or(false);
            let rating_reserve = if has_rating { 5 } else { 0 };
            let max_name_len = inner_w.saturating_sub(rating_reserve);

            // Hard-truncate title so it never overflows the column
            let display_name = truncate_to(&prefixed_name, max_name_len);

            let (mut styled_name, _) = stylize_channel_name(
                &display_name, false, false, parsed.quality, None, None,
                Style::default().fg(TEXT_PRIMARY),
            );
            
            // Colorize the checkmark if it's there
            if is_watched {
                if let Some(first_span) = styled_name.first_mut() {
                    if first_span.content.starts_with("✓") {
                        let check_span = ratatui::text::Span::styled("✓ ", Style::default().fg(crate::ui::colors::SOFT_GREEN));
                        first_span.content = std::borrow::Cow::Owned(first_span.content[2..].to_string());
                        
                        let mut new_spans = vec![check_span];
                        new_spans.extend(styled_name);
                        styled_name = new_spans;
                    }
                }
            }

            spans.extend(styled_name);

            // Rating: only when > 0, with color coding
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

    // Show total count in title — position indicator is less useful than total here
    let title = if total == 0 {
        "movies".to_string()
    } else {
        format!("movies  {} / {}", selected.saturating_add(1), total)
    };
    let is_active = app.active_pane == Pane::Streams;
    let border_color = if is_active { SOFT_GREEN } else { DARK_GREEN };
    let inner_area = crate::ui::common::render_matrix_box_active(f, area, &title, border_color, is_active);

    let list = List::new(items)
        .highlight_style(Style::default().bg(HIGHLIGHT_BG).fg(MATRIX_GREEN).add_modifier(Modifier::BOLD))
        .highlight_symbol(" ▎");

    let mut adjusted_state = app.vod_stream_list_state.clone();
    if adjusted_start > 0 { adjusted_state.select(Some(selected - adjusted_start)); }
    f.render_stateful_widget(list, inner_area, &mut adjusted_state);
}

pub fn render_vod_details_pane(f: &mut Frame, app: &mut App, area: Rect) {
    let inner_area = crate::ui::common::render_matrix_box(f, area, "details", SOFT_GREEN);
    let panel_w = inner_area.width.saturating_sub(1) as usize;

    let mut details_text: Vec<Line> = Vec::new();

    if let Some(vod_info) = &app.current_vod_info {
        if let Some(info) = &vod_info.info {
            // ── Title ──────────────────────────────────────────────
            // Strip IPTV provider prefix ("EN|", "MULTI-LANG|", etc.)
            let raw_title = info.get("name").and_then(|v| v.as_str()).map(|s| s.to_string())
                .or_else(|| vod_info.movie_data.as_ref().and_then(|m| m.name.clone()))
                .unwrap_or_default();

            let mut title = strip_provider_prefix(&raw_title).to_string();

            // Append year from stream name if not already present
            if !title.contains('(') && !title.contains('[') {
                if let Some(stream) = app.vod_streams.get(app.selected_vod_stream_index) {
                    let re_year = regex::Regex::new(r"[\(\[](19|20)\d{2}[\)\]]").unwrap();
                    if let Some(mat) = re_year.find(&stream.name) {
                        title.push(' ');
                        title.push_str(mat.as_str());
                    }
                }
            }

            if !title.is_empty() {
                // Wrap long titles at panel width
                details_text.push(Line::from(vec![
                    ratatui::text::Span::styled(
                        truncate_to(&title, panel_w),
                        Style::default().fg(TEXT_PRIMARY).add_modifier(Modifier::BOLD),
                    ),
                ]));
                details_text.push(Line::from(""));
            }

            // ── Rating ─────────────────────────────────────────────
            // Only show when rating > 0 (0 = no data, not a bad film)
            if let Some(rating_raw) = info.get("rating").and_then(|v| {
                match v {
                    serde_json::Value::String(s) if !s.is_empty() => Some(s.clone()),
                    serde_json::Value::Number(n) => Some(n.to_string()),
                    _ => None,
                }
            }) {
                let rating_val: f64 = rating_raw.parse().unwrap_or(0.0);
                if rating_val > 0.0 {
                    let rating_color = crate::ui::utils::get_rating_color(&rating_raw);
                    let filled = ((rating_val / 2.0).round() as usize).min(5);
                    let empty = 5usize.saturating_sub(filled);
                    let stars = format!("{}{}", "★".repeat(filled), "☆".repeat(empty));
                    details_text.push(Line::from(vec![
                        ratatui::text::Span::styled("rating  ", Style::default().fg(TEXT_SECONDARY)),
                        ratatui::text::Span::styled(
                            format!("{}/10  {}", rating_raw, stars),
                            Style::default().fg(rating_color),
                        ),
                    ]));
                }
            }

            // ── Runtime ────────────────────────────────────────────
            if let Some(runtime) = info.get("runtime").and_then(|v| {
                match v {
                    serde_json::Value::String(s) if !s.is_empty() => Some(s.clone()),
                    serde_json::Value::Number(n) => Some(n.to_string()),
                    _ => None,
                }
            }) {
                if !runtime.is_empty() && runtime != "0" {
                    details_text.push(Line::from(vec![
                        ratatui::text::Span::styled("runtime ", Style::default().fg(TEXT_SECONDARY)),
                        ratatui::text::Span::styled(format!("{} min", runtime), Style::default().fg(TEXT_PRIMARY)),
                    ]));
                }
            }

            // ── Release date ───────────────────────────────────────
            if let Some(releasedate) = info.get("releasedate").and_then(|v| v.as_str()) {
                if !releasedate.is_empty() {
                    details_text.push(Line::from(vec![
                        ratatui::text::Span::styled("released ", Style::default().fg(TEXT_SECONDARY)),
                        ratatui::text::Span::styled(releasedate, Style::default().fg(TEXT_PRIMARY)),
                    ]));
                }
            }

            // ── Genre ──────────────────────────────────────────────
            if let Some(genre) = info.get("genre").and_then(|v| v.as_str()) {
                if !genre.is_empty() {
                    details_text.push(Line::from(vec![
                        ratatui::text::Span::styled("genre   ", Style::default().fg(TEXT_SECONDARY)),
                        // Use TEXT_PRIMARY instead of MATRIX_GREEN — genre doesn't need accent color
                        ratatui::text::Span::styled(genre, Style::default().fg(TEXT_PRIMARY)),
                    ]));
                }
            }

            // ── Plot ───────────────────────────────────────────────
            if let Some(plot) = info.get("plot").and_then(|v| v.as_str())
                .or_else(|| info.get("description").and_then(|v| v.as_str()))
            {
                if !plot.is_empty() {
                    details_text.push(Line::from(""));
                    details_text.push(Line::from(
                        ratatui::text::Span::styled(plot, Style::default().fg(TEXT_SECONDARY)),
                    ));
                    details_text.push(Line::from(""));
                }
            }

            // ── Cast ───────────────────────────────────────────────
            if let Some(cast) = info.get("cast").and_then(|v| v.as_str())
                .or_else(|| info.get("actors").and_then(|v| v.as_str()))
            {
                if !cast.is_empty() {
                    // Truncate long cast lists to fit the panel
                    let cast_display = truncate_to(cast, panel_w.saturating_sub(5));
                    details_text.push(Line::from(vec![
                        ratatui::text::Span::styled("cast ", Style::default().fg(TEXT_SECONDARY)),
                        ratatui::text::Span::styled(cast_display, Style::default().fg(TEXT_PRIMARY)),
                    ]));
                }
            }
        }
    } else {
        // Empty state: centered prompt
        let empty_y = inner_area.height / 2;
        let empty_area = Rect {
            x: inner_area.x,
            y: inner_area.y + empty_y,
            width: inner_area.width,
            height: 1,
        };
        f.render_widget(
            Paragraph::new("← select a category, then pick a movie")
                .alignment(Alignment::Center)
                .style(Style::default().fg(TEXT_DIM)),
            empty_area,
        );
        return;
    }

    let paragraph = Paragraph::new(details_text)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, inner_area);
}
