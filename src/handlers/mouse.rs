use crate::api::get_id_str;
use crate::app::{App, AsyncAction, CurrentScreen, Pane};
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use tokio::sync::mpsc;

pub fn handle_mouse_event(app: &mut App, mouse: MouseEvent, tx: &mpsc::Sender<AsyncAction>) {
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
                            app.session.selected_account_index = row;
                            app.account_list_state.select(Some(row));
                        }
                    }
                }
                CurrentScreen::Categories
                | CurrentScreen::Streams
                | CurrentScreen::VodCategories
                | CurrentScreen::VodStreams
                | CurrentScreen::SeriesCategories
                | CurrentScreen::SeriesStreams
                | CurrentScreen::GlobalSearch => {
                    if x >= app.area_categories.x
                        && x < app.area_categories.x + app.area_categories.width
                        && y >= app.area_categories.y
                        && y < app.area_categories.y + app.area_categories.height
                    {
                        app.active_pane = Pane::Categories;

                        let list_y = y.saturating_sub(app.area_categories.y + 1) as usize;

                        let current_offset = match app.current_screen {
                            CurrentScreen::Categories | CurrentScreen::Streams => {
                                app.category_list_state.offset()
                            }
                            CurrentScreen::VodCategories | CurrentScreen::VodStreams => {
                                app.vod_category_list_state.offset()
                            }
                            CurrentScreen::SeriesCategories | CurrentScreen::SeriesStreams => {
                                app.series_category_list_state.offset()
                            }
                            _ => 0,
                        };

                        let selected_idx = current_offset + list_y;
                        let account_name = app
                            .config
                            .accounts
                            .get(app.session.selected_account_index)
                            .map(|a| a.name.clone())
                            .unwrap_or_default();

                        match app.current_screen {
                            CurrentScreen::Categories | CurrentScreen::Streams => {
                                if selected_idx < app.categories.len() {
                                    app.selected_category_index = selected_idx;
                                    app.category_list_state.select(Some(selected_idx));

                                    // Trigger load for streams
                                    if let Some(client) = &app.session.current_client {
                                        let cat_id =
                                            app.categories[selected_idx].category_id.clone();
                                        let client = client.clone();
                                        let _tx = tx.clone();
                                        let favs = app.config.favorites.streams.clone();
                                        let pms = app.config.processing_modes.clone();
                                        let acc_name_cloned = account_name.clone();
                                        let tx_cloned = tx.clone();
                                        tokio::spawn(async move {
                                            match client
                                                .get_live_streams(&cat_id, Some(tx_cloned.clone()))
                                                .await
                                            {
                                                Ok(mut streams) => {
                                                    crate::preprocessing::preprocess_streams(
                                                        &mut streams,
                                                        &favs,
                                                        &pms,
                                                        true,
                                                        &acc_name_cloned,
                                                        Some(tx_cloned.clone()),
                                                    );
                                                    let _ = tx_cloned
                                                        .send(AsyncAction::StreamsLoaded(
                                                            streams, cat_id,
                                                        ))
                                                        .await;
                                                }
                                                Err(e) => {
                                                    let _ = tx_cloned
                                                        .send(AsyncAction::Error(e.to_string()))
                                                        .await;
                                                }
                                            }
                                        });
                                    }
                                    app.current_screen = CurrentScreen::Streams;
                                }
                            }
                            CurrentScreen::VodCategories | CurrentScreen::VodStreams => {
                                if selected_idx < app.vod_categories.len() {
                                    app.selected_vod_category_index = selected_idx;
                                    app.vod_category_list_state.select(Some(selected_idx));

                                    // Trigger load for VOD
                                    if let Some(client) = &app.session.current_client {
                                        let cat_id =
                                            app.vod_categories[selected_idx].category_id.clone();
                                        app.session.state_loading = true;
                                        app.active_pane = Pane::Streams;
                                        let client = client.clone();
                                        let tx = tx.clone();
                                        let pms = app.config.processing_modes.clone();
                                        let favs = app.config.favorites.vod_streams.clone();
                                        let acc_name_cloned = account_name.clone();
                                        let tx_cloned = tx.clone();
                                        tokio::spawn(async move {
                                            let result = if cat_id == "ALL" {
                                                client.get_vod_streams_all().await
                                            } else {
                                                client.get_vod_streams(&cat_id).await
                                            };
                                            match result {
                                                Ok(mut streams) => {
                                                    crate::preprocessing::preprocess_streams(
                                                        &mut streams,
                                                        &favs,
                                                        &pms,
                                                        false,
                                                        &acc_name_cloned,
                                                        Some(tx_cloned.clone()),
                                                    );
                                                    let _ = tx_cloned
                                                        .send(AsyncAction::VodStreamsLoaded(
                                                            streams, cat_id,
                                                        ))
                                                        .await;
                                                }
                                                Err(e) => {
                                                    let _ = tx_cloned
                                                        .send(AsyncAction::Error(e.to_string()))
                                                        .await;
                                                }
                                            }
                                        });
                                    }
                                    app.current_screen = CurrentScreen::VodStreams;
                                }
                            }
                            CurrentScreen::SeriesCategories | CurrentScreen::SeriesStreams => {
                                if selected_idx < app.series_categories.len() {
                                    app.selected_series_category_index = selected_idx;
                                    app.series_category_list_state.select(Some(selected_idx));

                                    // Trigger load for Series
                                    if let Some(client) = &app.session.current_client {
                                        let cat_id =
                                            app.series_categories[selected_idx].category_id.clone();
                                        let client = client.clone();
                                        let tx = tx.clone();
                                        let pms = app.config.processing_modes.clone();
                                        let favs = app.config.favorites.vod_streams.clone(); // Series use vod favorites
                                        app.session.state_loading = true;
                                        app.active_pane = Pane::Streams;
                                        let acc_name_cloned = account_name.clone();
                                        let tx_cloned = tx.clone();
                                        tokio::spawn(async move {
                                            match client.get_series_streams(&cat_id).await {
                                                Ok(mut streams) => {
                                                    crate::preprocessing::preprocess_streams(
                                                        &mut streams,
                                                        &favs,
                                                        &pms,
                                                        false,
                                                        &acc_name_cloned,
                                                        Some(tx_cloned.clone()),
                                                    );
                                                    let _ = tx_cloned
                                                        .send(AsyncAction::SeriesStreamsLoaded(
                                                            streams, cat_id,
                                                        ))
                                                        .await;
                                                }
                                                Err(e) => {
                                                    let _ = tx_cloned
                                                        .send(AsyncAction::Error(e.to_string()))
                                                        .await;
                                                }
                                            }
                                        });
                                    }
                                    app.current_screen = CurrentScreen::SeriesStreams;
                                }
                            }
                            _ => {}
                        }
                    } else if x >= app.area_streams.x
                        && x < app.area_streams.x + app.area_streams.width
                        && y >= app.area_streams.y
                        && y < app.area_streams.y + app.area_streams.height
                    {
                        app.active_pane = Pane::Streams;

                        let list_y = y.saturating_sub(app.area_streams.y + 1) as usize;

                        let current_offset = match app.current_screen {
                            CurrentScreen::Categories | CurrentScreen::Streams => {
                                app.stream_list_state.offset()
                            }
                            CurrentScreen::VodCategories | CurrentScreen::VodStreams => {
                                app.vod_stream_list_state.offset()
                            }
                            CurrentScreen::SeriesCategories | CurrentScreen::SeriesStreams => {
                                app.series_stream_list_state.offset()
                            }
                            CurrentScreen::GlobalSearch => app.global_search_list_state.offset(),
                            _ => 0,
                        };

                        let selected_idx = current_offset + list_y;

                        match app.current_screen {
                            CurrentScreen::Categories | CurrentScreen::Streams => {
                                if selected_idx < app.streams.len() {
                                    app.selected_stream_index = selected_idx;
                                    app.stream_list_state.select(Some(selected_idx));

                                    // Trigger Stream Player
                                    if let Some(client) = &app.session.current_client {
                                        let stream = &app.streams[selected_idx];
                                        let id = get_id_str(&stream.stream_id);
                                        let url = client.get_stream_url(&id, "ts");
                                        let _tx = tx.clone();
                                        app.session.state_loading = true;
                                        app.ui.player_error = None;
                                        app.session.loading_message =
                                            Some(format!("Preparing: {}...", stream.name));

                                        // The player playback action usually needs handle_key_event's player ref or
                                        // we just let the main loop or handle_key_event do playback proper.
                                        // In TUI, best if we just delegate it: create a new popup/play pending state
                                        app.pending_play_url = Some(url);
                                        app.pending_play_title = Some(stream.name.clone());
                                        // Force enter input to process playback properly if needed
                                    }
                                }
                            }
                            CurrentScreen::VodCategories | CurrentScreen::VodStreams => {
                                if selected_idx < app.vod_streams.len() {
                                    app.selected_vod_stream_index = selected_idx;
                                    app.vod_stream_list_state.select(Some(selected_idx));

                                    // Trigger VOD detail playback
                                    if let Some(client) = &app.session.current_client {
                                        let stream = &app.vod_streams[selected_idx];
                                        let id = get_id_str(&stream.stream_id);
                                        let extension =
                                            stream.container_extension.as_deref().unwrap_or("mp4");
                                        let url = client.get_vod_url(&id, extension);
                                        app.pending_play_url = Some(url);
                                        app.pending_play_title = Some(stream.name.clone());
                                        app.show_play_details = true;
                                    }
                                }
                            }
                            CurrentScreen::SeriesCategories | CurrentScreen::SeriesStreams => {
                                if selected_idx < app.series_streams.len() {
                                    app.selected_series_stream_index = selected_idx;
                                    app.series_stream_list_state.select(Some(selected_idx));

                                    // Trigger Series episode loading
                                    if let Some(client) = &app.session.current_client {
                                        let stream = &app.series_streams[selected_idx];
                                        let id = get_id_str(&stream.stream_id);
                                        app.session.state_loading = true;
                                        app.active_pane = Pane::Episodes;
                                        let tx_cloned = tx.clone();
                                        let client = client.clone();
                                        tokio::spawn(async move {
                                            match client.get_series_info(&id).await {
                                                Ok(info) => {
                                                    let _ = tx_cloned
                                                        .send(AsyncAction::SeriesInfoLoaded(info))
                                                        .await;
                                                }
                                                Err(e) => {
                                                    let _ = tx_cloned
                                                        .send(AsyncAction::Error(e.to_string()))
                                                        .await;
                                                }
                                            }
                                        });
                                    }
                                }
                            }
                            CurrentScreen::GlobalSearch => {
                                if selected_idx < app.global_search_results.len() {
                                    app.global_search_list_state.select(Some(selected_idx));
                                    // Same pending logic
                                    if let Some(client) = &app.session.current_client {
                                        let stream = &app.global_search_results[selected_idx];
                                        let id = get_id_str(&stream.stream_id);
                                        let extension =
                                            stream.container_extension.as_deref().unwrap_or("ts");
                                        let url = match stream.stream_type.as_str() {
                                            "movie" => client.get_vod_url(&id, extension),
                                            "series" => client.get_series_url(&id, extension),
                                            _ => client.get_stream_url(&id, "ts"), // Force TS for live streams
                                        };

                                        if stream.stream_type == "movie"
                                            || stream.stream_type == "series"
                                        {
                                            app.pending_play_url = Some(url);
                                            app.pending_play_title = Some(stream.name.clone());
                                            app.show_play_details = true;
                                        } else {
                                            app.pending_play_url = Some(url);
                                            app.pending_play_title = Some(stream.name.clone());
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    } else if x >= app.area_episodes.x
                        && x < app.area_episodes.x + app.area_episodes.width
                        && y >= app.area_episodes.y
                        && y < app.area_episodes.y + app.area_episodes.height
                        && app.active_pane == Pane::Episodes
                    {
                        // Series episode selection click support
                        let list_y = y.saturating_sub(app.area_episodes.y + 1) as usize;
                        let current_offset = app.series_episode_list_state.offset();
                        let selected_idx = current_offset + list_y;

                        if selected_idx < app.series_episodes.len() {
                            app.selected_series_episode_index = selected_idx;
                            app.series_episode_list_state.select(Some(selected_idx));

                            let episode = &app.series_episodes[app.selected_series_episode_index];
                            if let Some(client) = &app.session.current_client {
                                let id = episode
                                    .id
                                    .as_ref()
                                    .map(|v| get_id_str(v))
                                    .unwrap_or_default();
                                if !id.is_empty() {
                                    let ext =
                                        episode.container_extension.as_deref().unwrap_or("mp4");
                                    let url = client.get_series_url(&id, ext);
                                    app.pending_play_url = Some(url);
                                    app.pending_play_title = episode.title.clone();
                                    app.show_play_details = true;
                                }
                            }
                        }
                    }
                }
                CurrentScreen::SportsDashboard => {
                    if x >= app.area_categories.x
                        && x < app.area_categories.x + app.area_categories.width
                        && y >= app.area_categories.y
                        && y < app.area_categories.y + app.area_categories.height
                    {
                        app.active_pane = Pane::Categories;
                        let list_y = y.saturating_sub(app.area_categories.y + 1) as usize;
                        let current_offset = app.sports_category_list_state.offset();
                        let selected_idx: usize = current_offset + list_y;
                        if selected_idx < app.sports_categories.len() {
                            app.sports_category_list_state.select(Some(selected_idx));
                            app.sports_matches.clear();
                            app.current_sports_streams.clear();

                            let category = app.sports_categories[selected_idx].clone();
                            app.session.state_loading = true;
                            let tx = tx.clone();

                            let tx_cloned = tx.clone();
                            tokio::spawn(async move {
                                if let Ok(matches) =
                                    crate::sports::fetch_streamed_matches(&category).await
                                {
                                    let _ = tx_cloned
                                        .send(crate::app::AsyncAction::SportsMatchesLoaded(matches))
                                        .await;
                                } else {
                                    let _ = tx_cloned
                                        .send(crate::app::AsyncAction::Error(format!(
                                            "Failed to load sports"
                                        )))
                                        .await;
                                }
                            });
                        }
                    } else if x >= app.area_streams.x
                        && x < app.area_streams.x + app.area_streams.width
                        && y >= app.area_streams.y
                        && y < app.area_streams.y + app.area_streams.height
                    {
                        app.active_pane = Pane::Streams;
                        let list_y = y.saturating_sub(app.area_streams.y + 1) as usize;
                        let current_offset = app.sports_list_state.offset();
                        let selected_idx: usize = current_offset + list_y;

                        if selected_idx < app.sports_matches.len() {
                            app.sports_list_state.select(Some(selected_idx));
                            app.active_pane = Pane::Episodes; // Move right to sources
                        }
                    } else if x >= app.area_episodes.x
                        && x < app.area_episodes.x + app.area_episodes.width
                        && y >= app.area_episodes.y
                        && y < app.area_episodes.y + app.area_episodes.height
                        && app.active_pane == Pane::Episodes
                    {
                        // Sports stream source selection
                        let list_y = y.saturating_sub(app.area_episodes.y + 1) as usize;
                        let selected_idx: usize = list_y; // Assume no scrolling offset for sources

                        if selected_idx < app.current_sports_streams.len() {
                            let stream = &app.current_sports_streams[selected_idx];
                            let stream_url = &stream.embed_url;
                            let match_title = if let Some(m_idx) = app.sports_list_state.selected()
                            {
                                if m_idx < app.sports_matches.len() {
                                    app.sports_matches[m_idx].title.clone()
                                } else {
                                    "Sports".to_string()
                                }
                            } else {
                                "Sports".to_string()
                            };

                            let stream_title = format!("{} ({})", match_title, stream.source);

                            app.pending_play_url = Some(stream_url.clone());
                            app.pending_play_title = Some(stream_title);
                        }
                    }
                }
                _ => {}
            }
        }
        MouseEventKind::ScrollDown => {
            if app.show_guide.is_some() {
                app.guide_scroll = app.guide_scroll.saturating_add(3);
            } else {
                match app.current_screen {
                    CurrentScreen::Home => app.next_account(),
                    CurrentScreen::Categories | CurrentScreen::Streams => match app.active_pane {
                        Pane::Categories => app.half_page_down_category(),
                        Pane::Streams => app.half_page_down_stream(),
                        _ => {}
                    },
                    CurrentScreen::VodCategories | CurrentScreen::VodStreams => {
                        match app.active_pane {
                            Pane::Categories => app.half_page_down_category(),
                            Pane::Streams => app.half_page_down_vod_stream(),
                            _ => {}
                        }
                    }
                    CurrentScreen::SeriesCategories | CurrentScreen::SeriesStreams => {
                        match app.active_pane {
                            Pane::Categories => app.half_page_down_category(),
                            Pane::Streams => app.half_page_down_series_stream(),
                            Pane::Episodes => {
                                let half = app.page_size_for_pane(Pane::Episodes) / 2;
                                crate::app::App::jump_list(
                                    app.series_episodes.len(),
                                    &mut app.selected_series_episode_index,
                                    &mut app.series_episode_list_state,
                                    half.max(1),
                                    true,
                                );
                            }
                        }
                    }
                    CurrentScreen::SportsDashboard => {
                        match app.active_pane {
                            Pane::Categories => {
                                app.selected_sports_category_index = app
                                    .selected_sports_category_index
                                    .saturating_add(1)
                                    .min(app.sports_categories.len().saturating_sub(1));
                                app.sports_category_list_state
                                    .select(Some(app.selected_sports_category_index));
                            }
                            Pane::Streams => {
                                if let Some(mut selected) = app.sports_list_state.selected() {
                                    selected = selected
                                        .saturating_add(1)
                                        .min(app.sports_matches.len().saturating_sub(1));
                                    app.sports_list_state.select(Some(selected));
                                }
                            }
                            Pane::Episodes => {} // No scrolling for stream array yet
                        }
                    }
                    CurrentScreen::Settings => app.next_setting(),
                    CurrentScreen::GlobalSearch => app.next_global_search_result(),
                    _ => {}
                }
            }
        }
        MouseEventKind::ScrollUp => {
            if app.show_guide.is_some() {
                app.guide_scroll = app.guide_scroll.saturating_sub(3);
            } else {
                match app.current_screen {
                    CurrentScreen::Home => app.previous_account(),
                    CurrentScreen::Categories | CurrentScreen::Streams => match app.active_pane {
                        Pane::Categories => app.half_page_up_category(),
                        Pane::Streams => app.half_page_up_stream(),
                        _ => {}
                    },
                    CurrentScreen::VodCategories | CurrentScreen::VodStreams => {
                        match app.active_pane {
                            Pane::Categories => app.half_page_up_category(),
                            Pane::Streams => app.half_page_up_vod_stream(),
                            _ => {}
                        }
                    }
                    CurrentScreen::SeriesCategories | CurrentScreen::SeriesStreams => {
                        match app.active_pane {
                            Pane::Categories => app.half_page_up_category(),
                            Pane::Streams => app.half_page_up_series_stream(),
                            Pane::Episodes => {
                                let half = app.page_size_for_pane(Pane::Episodes) / 2;
                                crate::app::App::jump_list(
                                    app.series_episodes.len(),
                                    &mut app.selected_series_episode_index,
                                    &mut app.series_episode_list_state,
                                    half.max(1),
                                    false,
                                );
                            }
                        }
                    }
                    CurrentScreen::SportsDashboard => match app.active_pane {
                        Pane::Categories => {
                            app.selected_sports_category_index =
                                app.selected_sports_category_index.saturating_sub(1);
                            app.sports_category_list_state
                                .select(Some(app.selected_sports_category_index));
                        }
                        Pane::Streams => {
                            if let Some(mut selected) = app.sports_list_state.selected() {
                                selected = selected.saturating_sub(1);
                                app.sports_list_state.select(Some(selected));
                            }
                        }
                        Pane::Episodes => {}
                    },
                    CurrentScreen::Settings => app.previous_setting(),
                    CurrentScreen::GlobalSearch => app.previous_global_search_result(),
                    _ => {}
                }
            }
        }
        _ => {}
    }
}
