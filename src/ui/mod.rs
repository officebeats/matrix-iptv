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
use crate::ui::colors::MATRIX_GREEN;
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
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),     // Content
            Constraint::Length(1), // Footer
        ])
        .split(area);

    header::render_header(f, app, chunks[0]);
    footer::render_footer(f, app, chunks[2]);

    let content_area = chunks[1];

    match app.current_screen {
        CurrentScreen::Categories | CurrentScreen::Streams => {
            // Calculate column widths
            let (cat_width, stream_width) = calculate_two_column_split(&app.categories, content_area.width);
            
            // Split horizontally: Categories (left) | Streams+Intelligence (right)
            let h_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(cat_width),
                    Constraint::Min(stream_width),
                ])
                .split(content_area);

            // Categories takes full height on left
            panes::render_categories_pane(f, app, h_chunks[0], MATRIX_GREEN);

            // Check if focused stream is a sports event
            let is_sports_event = app.streams.get(app.selected_stream_index)
                .map(|s| crate::parser::parse_stream(&s.name, app.provider_timezone.as_deref()).sports_event.is_some())
                .unwrap_or(false);

            if is_sports_event {
                // Right side: Streams (top) + Intelligence (bottom)
                // Height: 4 for double borders + 2 for content = 6
                let intel_height = 6u16;
                let right_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(10),
                        Constraint::Length(intel_height),
                    ])
                    .split(h_chunks[1]);

                panes::render_streams_pane(f, app, right_chunks[0], MATRIX_GREEN);
                panes::render_stream_details_pane(f, app, right_chunks[1], MATRIX_GREEN);
            } else {
                // No sports event: streams takes full right side
                panes::render_streams_pane(f, app, h_chunks[1], MATRIX_GREEN);
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
            // Check if focused result is a sports event
            let is_sports_event = app.global_search_results.get(app.selected_stream_index)
                .map(|s| crate::parser::parse_stream(&s.name, app.provider_timezone.as_deref()).sports_event.is_some())
                .unwrap_or(false);

            if is_sports_event {
                let intel_height = 6u16;
                let layout_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(10),
                        Constraint::Length(intel_height),
                    ])
                    .split(content_area);

                panes::render_global_search_pane(f, app, layout_chunks[0]);
                panes::render_stream_details_pane(f, app, layout_chunks[1], MATRIX_GREEN);
            } else {
                panes::render_global_search_pane(f, app, content_area);
            }
        }
        _ => {}
    }
}
