use crate::app::{App, AsyncAction, CurrentScreen, Pane};
use crate::api::{Stream, get_id_str, SeriesEpisode, Category};
use crate::{preprocessing, parser};
use tokio::sync::mpsc;
use std::sync::Arc;


pub async fn handle_async_action(
    app: &mut App,
    action: AsyncAction,
    tx: &mpsc::Sender<AsyncAction>,
) {
    match action {
        AsyncAction::LoginSuccess(client, ui, si) => {
            app.current_client = Some(client);
            app.account_info = ui.clone();
            app.server_info = si.clone();
            app.provider_timezone = si.and_then(|s| s.timezone);
            
            app.search_mode = false;
            app.search_state.query.clear();

            if let Some(account) = app.config.accounts.get(app.selected_account_index) {
                app.total_channels = account.total_channels.unwrap_or(0);
                app.total_movies = account.total_movies.unwrap_or(0);
                app.total_series = account.total_series.unwrap_or(0);
            }

            if let Some(info) = &ui {
                if app.total_channels == 0 {
                    app.total_channels = match &info.total_live_streams {
                        Some(serde_json::Value::Number(n)) => n.as_u64().unwrap_or(0) as usize,
                        Some(serde_json::Value::String(s)) => s.parse::<usize>().unwrap_or(0),
                        _ => 0,
                    };
                }
                if app.total_movies == 0 {
                    app.total_movies = match &info.total_vod_streams {
                        Some(serde_json::Value::Number(n)) => n.as_u64().unwrap_or(0) as usize,
                        Some(serde_json::Value::String(s)) => s.parse::<usize>().unwrap_or(0),
                        _ => 0,
                    };
                }
                if app.total_series == 0 {
                    app.total_series = match &info.total_series_streams {
                        Some(serde_json::Value::Number(n)) => n.as_u64().unwrap_or(0) as usize,
                        Some(serde_json::Value::String(s)) => s.parse::<usize>().unwrap_or(0),
                        _ => 0,
                    };
                }
            }

            app.state_loading = true;

            // Determine if we need a full refresh based on auto_refresh_hours config
            let should_full_refresh = {
                let last = app.config.accounts.get(app.selected_account_index)
                    .and_then(|a| a.last_refreshed).unwrap_or(0);
                let now = chrono::Utc::now().timestamp();
                let threshold_hours = app.config.auto_refresh_hours as i64;
                // Refresh if: threshold is 0 (always), no last_refreshed, or stale
                threshold_hours == 0 || last == 0 || (now - last) > (threshold_hours * 3600)
            };

            // Update last_refreshed timestamp only if we're doing a full refresh
            if should_full_refresh {
                let ts_now = chrono::Utc::now().timestamp();
                if let Some(account) = app.config.accounts.get_mut(app.selected_account_index) {
                    account.last_refreshed = Some(ts_now);
                    let _ = app.config.save();
                }
            }

            if let Some(client) = &app.current_client {
                let client = client.clone();
                let tx = tx.clone();
                let pms = app.config.processing_modes.clone();
                let account_name = app.config.accounts.get(app.selected_account_index)
                                    .map(|a| a.name.clone()).unwrap_or_default();

                // Live Categories
                let c1 = client.clone();
                let t1 = tx.clone();
                let cat_favs = app.config.favorites.categories.clone();
                let account_name_live = account_name.clone();
                let pms1 = pms.clone();
                tokio::spawn(async move {
                    match c1.get_live_categories().await {
                        Ok(mut cats) => {
                            preprocessing::preprocess_categories(&mut cats, &cat_favs, &pms1, true, false, &account_name_live);
                            let _ = t1.send(AsyncAction::CategoriesLoaded(cats)).await;
                        }
                        Err(e) => {
                            let _ = t1.send(AsyncAction::Error(format!("Live Categories Error: {}", e))).await;
                        }
                    }
                });

                // VOD Categories
                let c_vod = client.clone();
                let t_vod = tx.clone();
                let vod_cat_favs = app.config.favorites.vod_categories.clone();
                let account_name_vod_c = account_name.clone();
                let pms_v = pms.clone();
                tokio::spawn(async move {
                    match c_vod.get_vod_categories().await {
                        Ok(mut cats) => {
                            preprocessing::preprocess_categories(&mut cats, &vod_cat_favs, &pms_v, false, true, &account_name_vod_c);
                            let _ = t_vod.send(AsyncAction::VodCategoriesLoaded(cats)).await;
                        }
                        Err(e) => {
                            let _ = t_vod.send(AsyncAction::Error(format!("VOD Categories Error: {}", e))).await;
                        }
                    }
                });

                // Series Categories (always fetch - small payload)
                let c5 = client.clone();
                let t5 = tx.clone();
                let series_cat_favs = app.config.favorites.categories.clone();
                let account_name_ser_c = account_name.clone();
                let pms5 = pms.clone();
                tokio::spawn(async move {
                    match c5.get_series_categories().await {
                        Ok(mut cats) => {
                            preprocessing::preprocess_categories(&mut cats, &series_cat_favs, &pms5, false, false, &account_name_ser_c);
                            let _ = t5.send(AsyncAction::SeriesCategoriesLoaded(cats)).await;
                        }
                        Err(e) => {
                            let _ = t5.send(AsyncAction::Error(format!("Series Categories Error: {}", e))).await;
                        }
                    }
                });

                if should_full_refresh {
                    let c2 = client.clone();
                    let t2 = tx.clone();
                    let stream_favs = app.config.favorites.streams.clone();
                    let account_name_live_s_c = account_name.clone();
                    let pms2 = pms.clone();
                    tokio::spawn(async move {
                        // Strategy: Fetch categories first, then fetch all category streams in parallel
                        // This is dramatically faster than a single get_live_streams("ALL") call
                        // which often times out or fails on large providers
                        let cats = match c2.get_live_categories().await {
                            Ok(cats) => cats,
                            Err(_) => {
                                // Fallback to monolithic ALL call if categories fail
                                match c2.get_live_streams("ALL").await {
                                    Ok(mut streams) => {
                                        preprocessing::preprocess_streams(&mut streams, &stream_favs, &pms2, true, &account_name_live_s_c);
                                        let _ = t2.send(AsyncAction::TotalChannelsLoaded(streams)).await;
                                    }
                                    Err(e) => {
                                        let _ = t2.send(AsyncAction::LoadingMessage(format!("Scan Warning: {}", e))).await;
                                    }
                                }
                                return;
                            }
                        };

                        // Parallel fetch: spawn a future for each category
                        let mut handles = Vec::with_capacity(cats.len());
                        for cat in &cats {
                            let c = c2.clone();
                            let cat_id = cat.category_id.clone();
                            handles.push(tokio::spawn(async move {
                                c.get_live_streams(&cat_id).await.unwrap_or_default()
                            }));
                        }

                        // Collect results
                        let mut all_streams = Vec::new();
                        for handle in handles {
                            if let Ok(streams) = handle.await {
                                all_streams.extend(streams);
                            }
                        }

                        // Deduplicate by stream ID
                        {
                            use std::collections::HashSet;
                            let mut seen = HashSet::with_capacity(all_streams.len());
                            all_streams.retain(|s| {
                                let id = crate::api::get_id_str(&s.stream_id);
                                seen.insert(id)
                            });
                        }

                        preprocessing::preprocess_streams(&mut all_streams, &stream_favs, &pms2, true, &account_name_live_s_c);
                        let _ = t2.send(AsyncAction::TotalChannelsLoaded(all_streams)).await;
                    });

                    // VOD Full Scan - Delayed even more
                    let c4 = client.clone();
                    let t4 = tx.clone();
                    let vod_favs = app.config.favorites.vod_streams.clone();
                    let account_name_vod_s_c = account_name.clone();
                    let pms4 = pms.clone();
                    tokio::spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
                        match c4.get_vod_streams_all().await {
                            Ok(mut streams) => {
                                preprocessing::preprocess_streams(&mut streams, &vod_favs, &pms4, false, &account_name_vod_s_c);
                                let _ = t4.send(AsyncAction::TotalMoviesLoaded(streams)).await;
                            }
                            Err(e) => {
                                let _ = t4.send(AsyncAction::LoadingMessage(format!("Scan Warning: {}", e))).await;
                            }
                        }
                    });

                    // Series Full Scan - Delayed even more
                    let c_series = client.clone();
                    let t_series = tx.clone();
                    let series_favs = app.config.favorites.vod_streams.clone(); 
                    let account_name_ser_s_c = account_name.clone();
                    let pms_ser = pms.clone();
                    tokio::spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(2500)).await;
                        match c_series.get_series_all().await {
                            Ok(mut series) => {
                                preprocessing::preprocess_streams(&mut series, &series_favs, &pms_ser, false, &account_name_ser_s_c);
                                let _ = t_series.send(AsyncAction::TotalSeriesLoaded(series)).await;
                            }
                            Err(e) => {
                                let _ = t_series.send(AsyncAction::LoadingMessage(format!("Scan Warning: {}", e))).await;
                            }
                        }
                    });
                }
            }
        }
        AsyncAction::LoginFailed(e) => {
            app.login_error = Some(e);
            app.state_loading = false;
            app.loading_message = None;
        }
        AsyncAction::CategoriesLoaded(cats) => {
            let wrapped: Vec<Arc<Category>> = cats.into_iter().map(Arc::new).collect();
            app.all_categories = wrapped.clone();
            app.categories = wrapped;
            if !app.categories.is_empty() {
                app.selected_category_index = 0;
                app.category_list_state.select(Some(0));
                if app.current_screen == CurrentScreen::Home || app.current_screen == CurrentScreen::Login {
                    app.current_screen = CurrentScreen::ContentTypeSelection;
                }
            }
            app.state_loading = false;
            app.loading_message = None;
        }
        AsyncAction::StreamsLoaded(streams, cat_id) => {
            let wrapped: Vec<Arc<Stream>> = streams.into_iter().map(Arc::new).collect();
            if cat_id == "ALL" {
                app.global_all_streams = wrapped.clone();
            }
            app.all_streams = wrapped.clone();
            app.streams = wrapped;
            app.current_screen = CurrentScreen::Streams;
            app.active_pane = Pane::Streams;
            app.state_loading = false;
            app.loading_message = None;
            app.selected_stream_index = 0;
            app.stream_list_state.select(Some(0));
        }
        AsyncAction::VodCategoriesLoaded(cats) => {
            let wrapped: Vec<Arc<Category>> = cats.into_iter().map(Arc::new).collect();
            app.all_vod_categories = wrapped.clone();
            app.vod_categories = wrapped;
            if !app.vod_categories.is_empty() {
                app.selected_vod_category_index = 0;
                app.vod_category_list_state.select(Some(0));
                if app.current_screen == CurrentScreen::Home || app.current_screen == CurrentScreen::Login {
                    app.current_screen = CurrentScreen::ContentTypeSelection;
                }
            }
            app.state_loading = false;
            app.loading_message = None;
        }
        AsyncAction::VodStreamsLoaded(streams, cat_id) => {
            let wrapped: Vec<Arc<Stream>> = streams.into_iter().map(Arc::new).collect();
            if cat_id == "ALL" {
                app.global_all_vod_streams = wrapped.clone();
            }
            app.all_vod_streams = wrapped.clone();
            app.vod_streams = wrapped;
            app.current_screen = CurrentScreen::VodStreams;
            app.active_pane = Pane::Streams;
            app.state_loading = false;
            app.loading_message = None;
            app.selected_vod_stream_index = 0;
            app.vod_stream_list_state.select(Some(0));
        }
        AsyncAction::SeriesCategoriesLoaded(cats) => {
            let wrapped: Vec<Arc<Category>> = cats.into_iter().map(Arc::new).collect();
            app.all_series_categories = wrapped.clone();
            app.series_categories = wrapped;
            if !app.series_categories.is_empty() {
                app.selected_series_category_index = 0;
                app.series_category_list_state.select(Some(0));
                if app.current_screen == CurrentScreen::Home || app.current_screen == CurrentScreen::Login {
                    app.current_screen = CurrentScreen::ContentTypeSelection;
                }
            }
            app.state_loading = false;
            app.loading_message = None;
        }
        AsyncAction::SeriesStreamsLoaded(streams, cat_id) => {
            let wrapped: Vec<Arc<Stream>> = streams.into_iter().map(Arc::new).collect();
            if cat_id == "ALL" {
                app.global_all_series_streams = wrapped.clone();
            }
            app.all_series_streams = wrapped.clone();
            app.series_streams = wrapped;
            app.current_screen = CurrentScreen::SeriesStreams;
            app.active_pane = Pane::Streams;
            app.state_loading = false;
            app.loading_message = None;
            app.selected_series_stream_index = 0;
            app.series_stream_list_state.select(Some(0));
        }
        AsyncAction::PlayerStarted => {
            app.state_loading = false;
            app.loading_message = None;
        }
        AsyncAction::PlayerFailed(e) => {
            app.state_loading = false;
            app.loading_message = None;
            app.login_error = Some(e);
        }
        AsyncAction::LoadingMessage(msg) => {
            app.loading_message = Some(msg);
        }
        AsyncAction::TotalChannelsLoaded(streams) => {
            let count = streams.len();
            app.total_channels = count;
            app.global_all_streams = streams.into_iter().map(Arc::new).collect();
            if let Some(account) = app.config.accounts.get_mut(app.selected_account_index) {
                account.total_channels = Some(count);
                let _ = app.config.save();
            }
            // If user was actively waiting for "All Channels", navigate to streams view
            if app.state_loading {
                app.all_streams = app.global_all_streams.clone();
                app.streams = app.all_streams.clone();
                app.current_screen = CurrentScreen::Streams;
                app.active_pane = Pane::Streams;
                app.selected_stream_index = 0;
                app.stream_list_state.select(Some(0));
                app.state_loading = false;
                app.loading_message = None;
            }
            if app.search_mode { app.update_search(); }
        }
        AsyncAction::TotalMoviesLoaded(streams) => {
            let count = streams.len();
            app.total_movies = count;
            app.global_all_vod_streams = streams.into_iter().map(Arc::new).collect();
            if let Some(account) = app.config.accounts.get_mut(app.selected_account_index) {
                account.total_movies = Some(count);
                let _ = app.config.save();
            }
            if app.search_mode { app.update_search(); }
        }
        AsyncAction::TotalSeriesLoaded(series) => {
            let count = series.len();
            app.total_series = count;
            app.global_all_series_streams = series.into_iter().map(Arc::new).collect();
            if let Some(account) = app.config.accounts.get_mut(app.selected_account_index) {
                account.total_series = Some(count);
                let _ = app.config.save();
            }
            if app.search_mode { app.update_search(); }
        }
        AsyncAction::PlaylistRefreshed(client, ui, si) => {
            app.current_client = Some(client.clone());
            app.account_info = ui.clone();
            app.server_info = si.clone();
            app.state_loading = true; // Stay loading while we reload data
            
            if let Some(account) = app.config.accounts.get_mut(app.selected_account_index) {
                account.last_refreshed = Some(chrono::Utc::now().timestamp());
                let _ = app.config.save();
            }

            let client = client.clone();
            let tx = tx.clone();
            let pms = app.config.processing_modes.clone();
            let account_name = app.config.accounts.get(app.selected_account_index)
                                .map(|a| a.name.clone()).unwrap_or_default();

            // 1. Live Categories
            let c1 = client.clone();
            let t1 = tx.clone();
            let cat_favs = app.config.favorites.categories.clone();
            let account_name_live = account_name.clone();
            let pms1 = pms.clone();
            tokio::spawn(async move {
                match c1.get_live_categories().await {
                    Ok(mut cats) => {
                        preprocessing::preprocess_categories(&mut cats, &cat_favs, &pms1, true, false, &account_name_live);
                        let _ = t1.send(AsyncAction::CategoriesLoaded(cats)).await;
                    }
                    Err(e) => { let _ = t1.send(AsyncAction::Error(format!("Live Categories Error: {}", e))).await; }
                }
            });

            // 2. VOD Categories
            let c_vod = client.clone();
            let t_vod = tx.clone();
            let vod_cat_favs = app.config.favorites.vod_categories.clone();
            let account_name_vod_c = account_name.clone();
            let pms_v = pms.clone();
            tokio::spawn(async move {
                match c_vod.get_vod_categories().await {
                    Ok(mut cats) => {
                        preprocessing::preprocess_categories(&mut cats, &vod_cat_favs, &pms_v, false, true, &account_name_vod_c);
                        let _ = t_vod.send(AsyncAction::VodCategoriesLoaded(cats)).await;
                    }
                    Err(e) => { let _ = t_vod.send(AsyncAction::Error(format!("VOD Categories Error: {}", e))).await; }
                }
            });

            // 3. Series Categories
            let c5 = client.clone();
            let t5 = tx.clone();
            let series_cat_favs = app.config.favorites.categories.clone(); 
            let account_name_ser_c = account_name.clone();
            let pms5 = pms.clone();
            tokio::spawn(async move {
                match c5.get_series_categories().await {
                    Ok(mut cats) => {
                        preprocessing::preprocess_categories(&mut cats, &series_cat_favs, &pms5, false, false, &account_name_ser_c);
                        let _ = t5.send(AsyncAction::SeriesCategoriesLoaded(cats)).await;
                    }
                    Err(e) => { let _ = t5.send(AsyncAction::Error(format!("Series Category Error: {}", e))).await; }
                }
            });

            // 4. Background Full Scans (Delayed) - DISABLED per user request
            /*
            let c2 = client.clone();
            let t2 = tx.clone();
            let stream_favs = app.config.favorites.streams.clone();
            let account_name_live_s_c = account_name.clone();
            let pms2 = pms.clone();
            tokio::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                if let Ok(mut streams) = c2.get_live_streams("ALL").await {
                    preprocessing::preprocess_streams(&mut streams, &stream_favs, &pms2, true, &account_name_live_s_c);
                    let _ = t2.send(AsyncAction::TotalChannelsLoaded(streams)).await;
                }
            });

            let c4 = client.clone();
            let t4 = tx.clone();
            let vod_favs = app.config.favorites.vod_streams.clone();
            let account_name_vod_s_c = account_name.clone();
            let pms4 = pms.clone();
            tokio::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
                if let Ok(mut streams) = c4.get_vod_streams_all().await {
                    preprocessing::preprocess_streams(&mut streams, &vod_favs, &pms4, false, &account_name_vod_s_c);
                    let _ = t4.send(AsyncAction::TotalMoviesLoaded(streams)).await;
                }
            });
            */
        }
        AsyncAction::SeriesInfoLoaded(info) => {
            app.current_series_info = Some(info.clone());
            app.state_loading = false;
            let mut episodes = Vec::new();
            if let serde_json::Value::Object(episodes_map) = &info.episodes {
                for (_season_key, season_episodes) in episodes_map {
                    if let serde_json::Value::Array(ep_array) = season_episodes {
                        for ep_val in ep_array {
                            if let Ok(mut episode) = serde_json::from_value::<SeriesEpisode>(ep_val.clone()) {
                                if app.config.playlist_mode.is_merica_variant() {
                                    if let Some(ref title) = episode.title {
                                        episode.title = Some(parser::clean_american_name(title));
                                    }
                                }
                                episodes.push(episode);
                            }
                        }
                    }
                }
            }
            episodes.sort_by(|a, b| {
                match a.season.cmp(&b.season) {
                    std::cmp::Ordering::Equal => a.episode_num.cmp(&b.episode_num),
                    other => other,
                }
            });
            app.series_episodes = episodes;
            app.selected_series_episode_index = 0;
            if !app.series_episodes.is_empty() {
                app.series_episode_list_state.select(Some(0));
            }
            app.state_loading = false;
        }
        AsyncAction::VodInfoLoaded(info) => {
            app.current_vod_info = Some(info);
            app.state_loading = false;
        }
        AsyncAction::EpgLoaded(stream_id, program_title) => {
            app.epg_cache.insert(stream_id.clone(), program_title.clone());
            let update_stream = |s: &mut Arc<Stream>| {
                if get_id_str(&s.stream_id) == stream_id {
                    // We must use Arc::make_mut to modify the inner data if we have unique ownership
                    // or clone if shared. In TUI usually shared.
                    // To avoid complex borrow checker issues with Arc in hot loop, 
                    // we can just re-create the Arc if needed, or if we want performance:
                    if let Some(inner) = Arc::get_mut(s) {
                         inner.stream_display_name = Some(program_title.clone());
                         inner.cached_parsed = None;
                    } else {
                         let mut new_s = (**s).clone();
                         new_s.stream_display_name = Some(program_title.clone());
                         new_s.cached_parsed = None;
                         *s = Arc::new(new_s);
                    }
                }
            };
            app.streams.iter_mut().for_each(update_stream);
            app.global_all_streams.iter_mut().for_each(update_stream);
            app.all_streams.iter_mut().for_each(update_stream);
        }
        AsyncAction::StreamHealthLoaded(stream_id, latency) => {
            let update_health = |s: &mut Arc<Stream>| {
                if get_id_str(&s.stream_id) == stream_id {
                    if let Some(inner) = Arc::get_mut(s) {
                        inner.latency_ms = Some(latency);
                    } else {
                        let mut new_s = (**s).clone();
                        new_s.latency_ms = Some(latency);
                        *s = Arc::new(new_s);
                    }
                }
            };
            app.streams.iter_mut().for_each(update_health);
            app.global_all_streams.iter_mut().for_each(update_health);
            app.all_streams.iter_mut().for_each(update_health);
            app.global_search_results.iter_mut().for_each(update_health);
        }
        AsyncAction::UpdateAvailable(v) => {
            app.new_version_available = Some(v);
            app.current_screen = CurrentScreen::UpdatePrompt;
        }
        AsyncAction::NoUpdateFound => {
            app.state_loading = false;
            app.loading_message = None;
            app.login_error = Some("System Protocol: You are running the latest version.".to_string());
        }
        AsyncAction::SportsMatchesLoaded(matches) => {
            app.sports_matches = matches;
            app.state_loading = false;
            app.loading_message = None;
            app.sports_list_state.select(Some(0));
            // Trigger stream fetch for the first match if it exists
            if !app.sports_matches.is_empty() {
                app.sports_details_loading = true;
            }
        }
        AsyncAction::SportsStreamsLoaded(streams) => {
            app.current_sports_streams = streams;
            app.sports_details_loading = false;
        }
        AsyncAction::ScoresLoaded(scores) => {
            app.live_scores = scores;
        }
        // Chromecast Casting
        AsyncAction::CastDevicesDiscovered(devices) => {
            app.cast_devices = devices;
            app.cast_discovering = false;
            if !app.cast_devices.is_empty() {
                app.selected_cast_device_index = 0;
                app.cast_device_list_state.select(Some(0));
            }
        }
        AsyncAction::CastStarted(device_name) => {
            app.state_loading = false;
            app.loading_message = Some(format!("▶ Casting to {}", device_name));
            app.show_cast_picker = false;
        }
        AsyncAction::CastFailed(e) => {
            app.cast_discovering = false;
            app.show_cast_picker = false;
            app.player_error = Some(format!("Cast failed: {}", e));
        }
        AsyncAction::Error(e) => {
            app.login_error = Some(e);
            app.state_loading = false;
            app.loading_message = None;
        }
    }
}
