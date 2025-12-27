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
            let (cat_width, stream_width) = calculate_two_column_split(&app.categories, content_area.width);
            let content_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(cat_width),
                    Constraint::Min(stream_width),
                ])
                .split(content_area);

            panes::render_categories_pane(f, app, content_chunks[0], MATRIX_GREEN);
            panes::render_streams_pane(f, app, content_chunks[1], MATRIX_GREEN);
        }
        CurrentScreen::VodCategories | CurrentScreen::VodStreams => {
            let (cat_width, stream_width) = (30, 70); // Update with dynamic if needed
            let content_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(cat_width),
                    Constraint::Min(stream_width),
                ])
                .split(content_area);

            vod::render_vod_categories_pane(f, app, content_chunks[0], MATRIX_GREEN);
            vod::render_vod_streams_pane(f, app, content_chunks[1], MATRIX_GREEN);
        }
        CurrentScreen::SeriesCategories | CurrentScreen::SeriesStreams => {
            series::render_series_view(f, app, content_area);
        }
        CurrentScreen::Settings | CurrentScreen::TimezoneSettings => {
            form::render_settings(f, app, content_area);
        }
        CurrentScreen::GlobalSearch => {
            panes::render_global_search_pane(f, app, content_area);
        }
        _ => {}
    }
}
