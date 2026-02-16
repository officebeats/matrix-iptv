pub mod colors;
pub mod utils;
pub mod common;
pub mod header;
pub mod footer;
pub mod panes;
pub mod popups;
pub mod loading;
pub mod form;
pub mod home;
pub mod vod;
pub mod series;
pub mod groups;
pub mod sports;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use crate::app::{App, CurrentScreen};
use crate::ui::colors::SOFT_GREEN;
use crate::ui::utils::calculate_two_column_split;

pub fn ui(f: &mut Frame, app: &mut App) {
    let area = f.area();

    // Base Screens
    match app.current_screen {
        CurrentScreen::Home => {
            home::render_home(f, app, area);
        }
        CurrentScreen::Login => {
            form::render_login(f, app, area);
        }
        CurrentScreen::Categories | CurrentScreen::Streams => {
            render_main_layout(f, app, area);
        }
        CurrentScreen::VodCategories | CurrentScreen::VodStreams => {
            render_main_layout(f, app, area);
        }
        CurrentScreen::SeriesCategories | CurrentScreen::SeriesStreams => {
            render_main_layout(f, app, area);
        }
        CurrentScreen::Settings | CurrentScreen::TimezoneSettings => {
            render_main_layout(f, app, area);
        }
        CurrentScreen::ContentTypeSelection => {
            popups::render_content_type_selection(f, app, area);
        }
        CurrentScreen::GroupManagement => {
            groups::render_group_management(f, app, area);
        }
        CurrentScreen::GroupPicker => {
            // Render the underlying screen first, then overlay the picker
            render_main_layout(f, app, area);
            groups::render_group_picker(f, app, area);
        }
        CurrentScreen::Play | CurrentScreen::GlobalSearch => {
            // Placeholder or actual play info screen
            render_main_layout(f, app, area);
        }
        CurrentScreen::UpdatePrompt => {
            popups::render_update_prompt(f, app, area);
        }
        CurrentScreen::SportsDashboard => {
            render_main_layout(f, app, area);
        }
    }

    // Overlays
    if app.loading_message.is_some() {
        loading::render_loading(f, app, area);
    }


    // Matrix Rain Overlay (Draws on top of everything except Help/Guide/Error if they need focus, but effectively screensaver covers all)
    // Actually screensaver should be on top of everything.
    if app.show_matrix_rain {
         #[cfg(not(target_arch = "wasm32"))]
         crate::matrix_rain::render_matrix_rain(f, app, area);
    }

    if app.show_guide.is_some() {
        popups::render_guide_popup(f, app, area);
    }

    if app.show_play_details {
        popups::render_play_details_popup(f, app, area);
    }

    if app.show_cast_picker {
        popups::render_cast_picker_popup(f, app, area);
    }

    if let Some(error) = &app.login_error {
        if app.current_screen != CurrentScreen::Login {
            popups::render_error_popup(f, area, error);
        }
    }
}

fn render_main_layout(f: &mut Frame, app: &mut App, area: Rect) {
    let header_height = 3; // Fixed height for bordered header
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(header_height), // Header
            Constraint::Min(0),     // Content
            Constraint::Length(2), // Footer (bordered bottom bar)
        ])
        .split(area);

    header::render_header(f, app, chunks[0]);
    footer::render_footer(f, app, chunks[2]);

    let content_area = chunks[1];

    match app.current_screen {
        CurrentScreen::Categories | CurrentScreen::Streams => {
            if app.active_pane == crate::app::Pane::Categories {
                // Full-width grid view — no streams pane until a category is selected
                panes::render_categories_pane(f, app, content_area, SOFT_GREEN);
            } else {
                // JiraTUI-inspired layout: Categories | Streams | Detail Panel
                let (cat_width, _stream_width) = calculate_two_column_split(&app.categories, content_area.width);
                
                // Show detail panel only if terminal is wide enough (>= 120 cols)
                let show_detail = content_area.width >= 120;
                
                if show_detail {
                    let detail_width = 30u16.min(content_area.width / 4); // ~25% or 30 cols max
                    let streams_width = content_area.width.saturating_sub(cat_width).saturating_sub(detail_width);
                    
                    let h_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Length(cat_width),
                            Constraint::Length(streams_width),
                            Constraint::Length(detail_width),
                        ])
                        .split(content_area);

                    panes::render_categories_pane(f, app, h_chunks[0], SOFT_GREEN);

                    // Check if focused stream is a sports event (use cache to avoid per-frame parsing)
                    let is_sports_event = app.streams.get(app.selected_stream_index)
                        .map(|s| {
                            if let Some(ref cached) = s.cached_parsed {
                                cached.sports_event.is_some() || app.get_score_for_stream(&cached.display_name).is_some()
                            } else {
                                false // Don't parse on render — assume non-sports layout if not cached
                            }
                        })
                        .unwrap_or(false);

                    if is_sports_event {
                        // Sports: Streams on top, Match Intelligence below in middle column
                        let mid_chunks = Layout::default()
                            .direction(Direction::Vertical)
                            .constraints([
                                Constraint::Min(10),
                                Constraint::Length(10),
                            ])
                            .split(h_chunks[1]);
                        panes::render_streams_pane(f, app, mid_chunks[0], SOFT_GREEN);
                        panes::render_stream_details_pane(f, app, mid_chunks[1], SOFT_GREEN);
                    } else {
                        panes::render_streams_pane(f, app, h_chunks[1], SOFT_GREEN);
                    }

                    // Right detail panel (always visible)
                    panes::render_channel_detail_panel(f, app, h_chunks[2], SOFT_GREEN);
                } else {
                    // Narrow terminal: 2-column layout (original behavior)
                    let h_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Length(cat_width),
                            Constraint::Min(20),
                        ])
                        .split(content_area);

                    panes::render_categories_pane(f, app, h_chunks[0], SOFT_GREEN);

                    let is_sports_event = app.streams.get(app.selected_stream_index)
                        .map(|s| {
                            if let Some(ref cached) = s.cached_parsed {
                                cached.sports_event.is_some() || app.get_score_for_stream(&cached.display_name).is_some()
                            } else {
                                false
                            }
                        })
                        .unwrap_or(false);

                    if is_sports_event {
                        let intel_height = 10u16;
                        let right_chunks = Layout::default()
                            .direction(Direction::Vertical)
                            .constraints([
                                Constraint::Min(10),
                                Constraint::Length(intel_height),
                            ])
                            .split(h_chunks[1]);
                        panes::render_streams_pane(f, app, right_chunks[0], SOFT_GREEN);
                        panes::render_stream_details_pane(f, app, right_chunks[1], SOFT_GREEN);
                    } else {
                        panes::render_streams_pane(f, app, h_chunks[1], SOFT_GREEN);
                    }
                }
            }
        }
        CurrentScreen::VodCategories | CurrentScreen::VodStreams => {
            vod::render_vod_view(f, app, content_area);
        }
        CurrentScreen::SeriesCategories | CurrentScreen::SeriesStreams => {
            series::render_series_view(f, app, content_area);
        }
        CurrentScreen::Settings | CurrentScreen::TimezoneSettings => {
            form::render_settings(f, app, content_area);
        }
        CurrentScreen::SportsDashboard => {
            sports::render_sports_view(f, app, content_area);
        }
        CurrentScreen::GlobalSearch => {
            // Check if focused result is a sports event OR has ESPN score data
            let is_sports_event = app.global_search_results.get(app.selected_stream_index)
                .map(|s| {
                    let parsed = crate::parser::parse_stream(&s.name, app.provider_timezone.as_deref());
                    parsed.sports_event.is_some() || app.get_score_for_stream(&parsed.display_name).is_some()
                })
                .unwrap_or(false);

            if is_sports_event {
                let intel_height = 10u16;
                let layout_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(10),
                        Constraint::Length(intel_height),
                    ])
                    .split(content_area);

                panes::render_global_search_pane(f, app, layout_chunks[0]);
                panes::render_stream_details_pane(f, app, layout_chunks[1], SOFT_GREEN);
            } else {
                panes::render_global_search_pane(f, app, content_area);
            }
        }
        _ => {}
    }
}
