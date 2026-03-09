use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{List, ListItem, Paragraph, Wrap},
    Frame,
};
use crate::app::{App, Pane};
use crate::ui::colors::{MATRIX_GREEN, SOFT_GREEN, DARK_GREEN, HIGHLIGHT_BG, TEXT_PRIMARY, TEXT_SECONDARY, TEXT_DIM};
use crate::ui::common::stylize_channel_name;

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
    if app.active_pane == Pane::Categories {
        // Categories stage: Full screen categories (Grid or List)
        crate::ui::panes::render_categories_pane(f, app, area, SOFT_GREEN);
    } else {
        // Streams stage: Master-Detail view (Streams | Details)
        let show_detail = area.width >= 80;
        
        if show_detail {
            let detail_width = 40u16.min(area.width / 2);
            let streams_width = area.width.saturating_sub(detail_width);
            
            let h_chunks = ratatui::layout::Layout::default()
                .direction(ratatui::layout::Direction::Horizontal)
                .constraints([
                    ratatui::layout::Constraint::Length(streams_width),
                    ratatui::layout::Constraint::Length(detail_width),
                ])
                .split(area);

            render_vod_streams_pane(f, app, h_chunks[0]);
            render_vod_details_pane(f, app, h_chunks[1]);
        } else {
            render_vod_streams_pane(f, app, area);
        }
    }
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
            let parsed = if let Some(ref cached) = s.cached_parsed {
                 cached.as_ref().clone()
            } else {
                crate::parser::parse_stream(&s.name, app.provider_timezone.as_deref())
            };
            let mut spans = vec![];

            // Metadata parsing (cached)
            let name = parsed.display_name.clone();

            // Check if recently watched (progress indicator)
            let is_watched = app.config.recently_watched.iter().any(|(id, _)| id == &s.stream_id.to_string());
            let prefix = if is_watched { "✓ "} else { "" };
            let prefixed_name = format!("{}{}", prefix, name);

            // Reserve space for year and rating: " (2024)" [7] + " [8.5]" [6] = 13 chars
            let has_year = parsed.year.is_some();
            let has_rating = s.rating.map(|r| r > 0.0).unwrap_or(false);
            let mut metadata_reserve = 0;
            if has_year { metadata_reserve += 7; }
            if has_rating { metadata_reserve += 6; }
            
            let max_name_len = inner_w.saturating_sub(metadata_reserve);

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

            // Year: in light blue parentheses
            if let Some(y) = &parsed.year {
                spans.push(ratatui::text::Span::styled(
                    format!(" ({})", y),
                    Style::default().fg(ratatui::style::Color::LightBlue),
                ));
            }

            // Rating: in brackets, with color coding
            if let Some(rating_f) = s.rating {
                if rating_f > 0.0 {
                    let rating_str = format!("{:.1}", rating_f);
                    let rating_color = crate::ui::utils::get_rating_color(&rating_str);
                    spans.push(ratatui::text::Span::styled(
                        format!(" [{}]", rating_str),
                        Style::default().fg(rating_color),
                    ));
                }
            }

            ListItem::new(Line::from(spans))
        }).collect();

    // Show total count in title — position indicator is less useful than total here
    let title = "movies";
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
    let current_selection = app.vod_streams.get(app.selected_vod_stream_index);
    
    if let Some(vod_info) = &app.current_vod_info {
        if let Some(info) = &vod_info.info {
            // ── Title ──────────────────────────────────────────────
            let raw_title = info.get("name").and_then(|v| v.as_str()).map(|s| s.to_string())
                .or_else(|| vod_info.movie_data.as_ref().and_then(|m| m.name.clone()))
                .unwrap_or_else(|| current_selection.map(|s| s.name.clone()).unwrap_or_default());

            let mut title = strip_provider_prefix(&raw_title).to_string();

            // Append year from stream name or cache if not already present
            if !title.contains('(') && !title.contains('[') {
                if let Some(stream) = current_selection {
                    if let Some(ref parsed) = stream.cached_parsed {
                        if let Some(ref y) = parsed.year {
                            title.push_str(&format!(" ({})", y));
                        }
                    } else {
                        let re_year = regex::Regex::new(r"[\(\[](19|20)\d{2}[\)\]]").unwrap();
                        if let Some(mat) = re_year.find(&stream.name) {
                            title.push(' ');
                            title.push_str(mat.as_str());
                        }
                    }
                }
            }

            if !title.is_empty() {
                details_text.push(Line::from(vec![
                    ratatui::text::Span::styled(
                        truncate_to(&title, panel_w),
                        Style::default().fg(TEXT_PRIMARY).add_modifier(Modifier::BOLD),
                    ),
                ]));
                details_text.push(Line::from(""));
            }

            // ── Rating ─────────────────────────────────────────────
            if let Some(rating_raw) = info.get("rating").and_then(|v| {
                match v {
                    serde_json::Value::String(s) if !s.is_empty() => Some(s.clone()),
                    serde_json::Value::Number(n) => Some(n.to_string()),
                    _ => None,
                }
            }).or_else(|| current_selection.and_then(|s| s.rating.map(|r| r.to_string()))) 
            {
                let rating_val: f64 = rating_raw.parse().unwrap_or(0.0);
                if rating_val > 0.0 {
                    let rating_color = crate::ui::utils::get_rating_color(&rating_raw);
                    let filled = ((rating_val / 2.0).round() as usize).min(5);
                    let empty = 5usize.saturating_sub(filled);
                    let stars = format!("{}{}", "*".repeat(filled), "-".repeat(empty));
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
                    let cast_display = truncate_to(cast, panel_w.saturating_sub(5));
                    details_text.push(Line::from(vec![
                        ratatui::text::Span::styled("cast ", Style::default().fg(TEXT_SECONDARY)),
                        ratatui::text::Span::styled(cast_display, Style::default().fg(TEXT_PRIMARY)),
                    ]));
                }
            }
        }
    } else if let Some(stream) = current_selection {
        // SNAPPY FALLBACK: Show basic info from cache while loading detailed info
        let (display_name, _) = if let Some(ref parsed) = stream.cached_parsed {
            (parsed.display_name.clone(), parsed.quality)
        } else {
            let p = crate::parser::parse_stream(&stream.name, app.provider_timezone.as_deref());
            (p.display_name, p.quality)
        };

        details_text.push(Line::from(vec![
            ratatui::text::Span::styled(
                truncate_to(&display_name, panel_w),
                Style::default().fg(TEXT_PRIMARY).add_modifier(Modifier::BOLD),
            ),
        ]));
        
        if let Some(rating_f) = stream.rating {
            if rating_f > 0.0 {
                let rating_str = format!("{:.1}", rating_f);
                let rating_color = crate::ui::utils::get_rating_color(&rating_str);
                details_text.push(Line::from(vec![
                    ratatui::text::Span::styled("rating  ", Style::default().fg(TEXT_SECONDARY)),
                    ratatui::text::Span::styled(
                        format!("{}/10 (cached)", rating_str),
                        Style::default().fg(rating_color),
                    ),
                ]));
            }
        }

        details_text.push(Line::from(""));
        details_text.push(Line::from(ratatui::text::Span::styled(
            "Loading extended details...",
            Style::default().fg(TEXT_DIM).add_modifier(Modifier::ITALIC),
        )));
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

    let paragraph = Paragraph::new(details_text).wrap(Wrap { trim: true });
    f.render_widget(paragraph, inner_area);
}
