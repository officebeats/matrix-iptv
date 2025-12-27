use crate::app::{App, CurrentScreen, Pane};
use crossterm::event::{MouseEvent, MouseEventKind, MouseButton};

pub fn handle_mouse_event(app: &mut App, mouse: MouseEvent) {
    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            let x = mouse.column;
            let y = mouse.row;

            match app.current_screen {
                CurrentScreen::Home => {
                    if x >= app.area_accounts.x
                        && x < app.area_accounts.x + app.area_accounts.width
                        && y > app.area_accounts.y
                        && y < app.area_accounts.y + app.area_accounts.height
                    {
                        let row = (y - app.area_accounts.y - 1) as usize;
                        if row < app.config.accounts.len() {
                            app.selected_account_index = row;
                            app.account_list_state.select(Some(row));
                        }
                    }
                }
                CurrentScreen::Categories
                | CurrentScreen::Streams
                | CurrentScreen::VodCategories
                | CurrentScreen::VodStreams => {
                    if x >= app.area_categories.x
                        && x < app.area_categories.x + app.area_categories.width
                        && y >= app.area_categories.y
                        && y < app.area_categories.y + app.area_categories.height
                    {
                        app.active_pane = Pane::Categories;
                    } else if x >= app.area_streams.x
                        && x < app.area_streams.x + app.area_streams.width
                        && y >= app.area_streams.y
                        && y < app.area_streams.y + app.area_streams.height
                    {
                        app.active_pane = Pane::Streams;
                    }
                }
                _ => {}
            }
        }
        MouseEventKind::ScrollDown => {
            if app.show_guide.is_some() {
                app.guide_scroll = app.guide_scroll.saturating_add(1);
            } else {
                match app.current_screen {
                    CurrentScreen::Home => app.next_account(),
                    CurrentScreen::Categories | CurrentScreen::Streams => {
                        match app.active_pane {
                            Pane::Categories => app.next_category(),
                            Pane::Streams => app.next_stream(),
                            _ => {}
                        }
                    }
                    CurrentScreen::VodCategories => app.next_vod_category(),
                    CurrentScreen::VodStreams => app.next_vod_stream(),
                    CurrentScreen::Settings => app.next_setting(),
                    _ => {}
                }
            }
        }
        MouseEventKind::ScrollUp => {
            if app.show_guide.is_some() {
                app.guide_scroll = app.guide_scroll.saturating_sub(1);
            } else {
                match app.current_screen {
                    CurrentScreen::Home => app.previous_account(),
                    CurrentScreen::Categories | CurrentScreen::Streams => {
                        match app.active_pane {
                            Pane::Categories => app.previous_category(),
                            Pane::Streams => app.previous_stream(),
                            _ => {}
                        }
                    }
                    CurrentScreen::VodCategories => app.previous_vod_category(),
                    CurrentScreen::VodStreams => app.previous_vod_stream(),
                    CurrentScreen::Settings => app.previous_setting(),
                    _ => {}
                }
            }
        }
        _ => {}
    }
}
