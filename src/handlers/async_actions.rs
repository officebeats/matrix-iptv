use crate::app::{App, AsyncAction, CurrentScreen, Pane};
use crate::api::{Stream, get_id_str, SeriesEpisode, Category};
use crate::{preprocessing, parser};
use crate::cache::CachedCatalog;
use tokio::sync::mpsc;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use futures::join;


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
            app.last_search_query.clear(); // Reset for next search

            if let Some(account) = app.config.accounts.get(app.selected_account_index) {
                app.total_channels = account.total_channels.unwrap_or(0);
                app.total_movies = account.total_movies.unwrap_or(0);
                app.total_series = account.total_series.unwrap_or(0);
            }

            if let Some(info) = &ui {
                if app.total_channels == 0 {
                    app.total_channels = info.total_live_streams.as_ref()
                        .and_then(|id| id.as_i64()).unwrap_or(0) as usize;
                }
                if app.total_movies == 0 {
                    app.total_movies = info.total_vod_streams.as_ref()
                        .and_then(|id| id.as_i64()).unwrap_or(0) as usize;
                }
                if app.total_series == 0 {
                    app.total_series = info.total_series_streams.as_ref()
                        .and_then(|id| id.as_i64()).unwrap_or(0) as usize;
                }
            }

            // Try to load from cache first for instant UI
            let account_name = app.config.accounts.get(app.selected_account_index)
                .map(|a| a.name.clone()).unwrap_or_default();
            let current_modes = app.config.processing_modes.clone();
            let auto_refresh_hours = app.config.auto_refresh_hours;
            
            let cache_hit = CachedCatalog::load(&account_name)
                .filter(|cache| !cache.is_stale(auto_refresh_hours))
                .filter(|cache| !cache.modes_changed(&current_modes));

            if let Some(cache) = cache_hit {
                // Cache hit - populate UI instantly
                app.all_categories = cache.live_categories.into_iter().map(Arc::new).collect();
                app.categories = app.all_categories.clone();
                app.global_all_streams = cache.live_streams.into_iter().map(Arc::new).collect();
                
                app.all_vod_categories = cache.vod_categories.into_iter().map(Arc::new).collect();
                app.vod_categories = app.all_vod_categories.clone();
                app.global_all_vod_streams = cache.vod_streams.into_iter().map(Arc::new).collect();
                
                app.all_series_categories = cache.series_categories.into_iter().map(Arc::new).collect();
                app.series_categories = app.all_series_categories.clone();
                app.global_all_series_streams = cache.series_streams.into_iter().map(Arc::new).collect();
                
                app.total_channels = cache.total_channels;
                app.total_movies = cache.total_movies;
                app.total_series = cache.total_series;
                
                // Restore category counts
                app.category_channel_counts = cache.category_counts.into_iter().collect();
                
                // Navigate to content selection immediately
                app.current_screen = CurrentScreen::ContentTypeSelection;
                app.state_loading = false;
                app.cache_loaded = true;
                
                // Select first items and populate streams from cache
                if !app.categories.is_empty() {
                    app.select_category(0);
                }
                if !app.vod_categories.is_empty() {
                    app.selected_vod_category_index = 0;
                    app.vod_category_list_state.select(Some(0));
                }
                if !app.series_categories.is_empty() {
                    app.selected_series_category_index = 0;
                    app.series_category_list_state.select(Some(0));
                }
                
                // Spawn background refresh to update data silently
                app.background_refresh_active = true;
                
                if let Some(client) = &app.current_client {
                    let client = client.clone();
                    let tx = tx.clone();
                    let pms = current_modes.clone();
                    let account_name_bg = account_name.clone();
                    let cat_favs = app.config.favorites.categories.clone();
                    let vod_cat_favs = app.config.favorites.vod_categories.clone();
                    
                    // Spawn a single task that fetches all categories in parallel using join!
                    tokio::spawn(async move {
                        // Execute all category fetches in parallel
                        let (live_result, vod_result, series_result) = join!(
                            client.get_live_categories(),
                            client.get_vod_categories(),
                            client.get_series_categories()
                        );

                        // Process and send Live Categories (silent fail on background refresh)
                        if let Ok(mut cats) = live_result {
                            preprocessing::preprocess_categories(&mut cats, &cat_favs, &pms, true, false, &account_name_bg);
                            let _ = tx.send(AsyncAction::CategoriesLoaded(cats)).await;
                        }

                        // Process and send VOD Categories
                        if let Ok(mut cats) = vod_result {
                            preprocessing::preprocess_categories(&mut cats, &vod_cat_favs, &pms, false, true, &account_name_bg);
                            let _ = tx.send(AsyncAction::VodCategoriesLoaded(cats)).await;
                        }

                        // Process and send Series Categories
                        if let Ok(mut cats) = series_result {
                            preprocessing::preprocess_categories(&mut cats, &cat_favs, &pms, false, false, &account_name_bg);
                            let _ = tx.send(AsyncAction::SeriesCategoriesLoaded(cats)).await;
                        }
                    });
                }
                
                // Update last_refreshed timestamp
                let ts_now = chrono::Utc::now().timestamp();
                if let Some(account) = app.config.accounts.get_mut(app.selected_account_index) {
                    account.last_refreshed = Some(ts_now);
                    let _ = app.config.save();
                }
            } else {
                // Cache miss or stale - normal loading flow
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
                    let cat_favs = app.config.favorites.categories.clone();
                    let vod_cat_favs = app.config.favorites.vod_categories.clone();

                    // Spawn a single task that fetches all categories in parallel using join!
                    tokio::spawn(async move {
                        // Execute all category fetches in parallel
                        let (live_result, vod_result, series_result) = join!(
                            client.get_live_categories(),
                            client.get_vod_categories(),
                            client.get_series_categories()
                        );

                        // Process and send Live Categories
                        match live_result {
                            Ok(mut cats) => {
                                preprocessing::preprocess_categories(&mut cats, &cat_favs, &pms, true, false, &account_name);
                                let _ = tx.send(AsyncAction::CategoriesLoaded(cats)).await;
                            }
                            Err(e) => {
                                let _ = tx.send(AsyncAction::Error(format!("Live Categories Error: {}", e))).await;
                            }
                        }

                        // Process and send VOD Categories
                        match vod_result {
                            Ok(mut cats) => {
                                preprocessing::preprocess_categories(&mut cats, &vod_cat_favs, &pms, false, true, &account_name);
                                let _ = tx.send(AsyncAction::VodCategoriesLoaded(cats)).await;
                            }
                            Err(e) => {
                                let _ = tx.send(AsyncAction::Error(format!("VOD Categories Error: {}", e))).await;
                            }
                        }

                        // Process and send Series Categories
                        match series_result {
                            Ok(mut cats) => {
                                preprocessing::preprocess_categories(&mut cats, &cat_favs, &pms, false, false, &account_name);
                                let _ = tx.send(AsyncAction::SeriesCategoriesLoaded(cats)).await;
                            }
                            Err(e) => {
                                let _ = tx.send(AsyncAction::Error(format!("Series Categories Error: {}", e))).await;
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
                // Use select_category to also populate streams from global cache if available
                app.select_category(0);
            }

            // Navigate to ContentTypeSelection immediately — channel scan
            // only starts when user explicitly picks "Live Channels"
            if app.current_screen == CurrentScreen::Home || app.current_screen == CurrentScreen::Login {
                app.current_screen = CurrentScreen::ContentTypeSelection;
            }
            app.state_loading = false;
            app.loading_message = None;
            
            // Check if background refresh is complete (all categories loaded)
            if app.background_refresh_active
                && !app.all_categories.is_empty()
                && !app.all_vod_categories.is_empty()
                && !app.all_series_categories.is_empty() {
                app.background_refresh_active = false;
            }
        }
        AsyncAction::StreamsLoaded(streams, cat_id) => {
            let wrapped: Vec<Arc<Stream>> = streams.into_iter().map(Arc::new).collect();
            if cat_id == "ALL" {
                app.global_all_streams = wrapped.clone();
            }
            app.all_streams = wrapped.clone();
            app.current_screen = CurrentScreen::Streams;
            app.active_pane = Pane::Streams;
            
            // Use update_search to apply Merica/filter logic to the view
            app.update_search();
            
            app.state_loading = false;
            app.loading_message = None;
        }
        AsyncAction::VodCategoriesLoaded(cats) => {
            let wrapped: Vec<Arc<Category>> = cats.into_iter().map(Arc::new).collect();
            app.all_vod_categories = wrapped.clone();
            app.vod_categories = wrapped;
            if !app.vod_categories.is_empty() {
                app.selected_vod_category_index = 0;
                app.vod_category_list_state.select(Some(0));
                // Don't navigate while live scan is running — TotalChannelsLoaded handles it
            }
            
            // Check if background refresh is complete (all categories loaded)
            if app.background_refresh_active
                && !app.all_categories.is_empty()
                && !app.all_vod_categories.is_empty()
                && !app.all_series_categories.is_empty() {
                app.background_refresh_active = false;
            }
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
                // Don't navigate while live scan is running — TotalChannelsLoaded handles it
            }
            
            // Check if background refresh is complete (all categories loaded)
            if app.background_refresh_active
                && !app.all_categories.is_empty()
                && !app.all_vod_categories.is_empty()
                && !app.all_series_categories.is_empty() {
                app.background_refresh_active = false;
            }
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
            // Track recently watched
            if let Some(stream) = app.get_selected_stream() {
                let id = crate::api::get_id_str(&stream.stream_id);
                let name = stream.name.clone();
                app.record_recently_watched(id, name);
            }
        }
        AsyncAction::PlayerFailed(e) => {
            app.state_loading = false;
            app.loading_message = None;
            app.login_error = Some(e);
        }
        AsyncAction::LoadingMessage(msg) => {
            if msg.is_empty() {
                app.loading_message = None;
            } else {
                app.loading_message = Some(msg.clone());
                // Add to verbose log
                if app.loading_log.len() >= 25 {
                    app.loading_log.pop_front();
                }
                app.loading_log.push_back(msg);
            }
        }
        AsyncAction::TotalChannelsLoaded(mut streams) => {
            let pms = app.config.processing_modes.clone();
            let favs = app.config.favorites.streams.clone();
            let account_name = app.config.accounts.get(app.selected_account_index)
                .map(|a| a.name.clone()).unwrap_or_default();
            
            preprocessing::preprocess_streams(&mut streams, &favs, &pms, true, &account_name);

            let count = streams.len();
            app.total_channels = count;
            app.global_all_streams = streams.into_iter().map(Arc::new).collect();
            let tz = app.provider_timezone.clone();
            App::pre_cache_parsed(&mut app.global_all_streams, tz.as_deref());
            if let Some(account) = app.config.accounts.get_mut(app.selected_account_index) {
                account.total_channels = Some(count);
                let _ = app.config.save();
            }

            // Build category → channel count map for UI display
            app.build_category_counts();

            // Save to cache for instant startup on next launch
            let account_name = app.config.accounts.get(app.selected_account_index)
                .map(|a| a.name.clone()).unwrap_or_default();
            let account_url = app.config.accounts.get(app.selected_account_index)
                .map(|a| a.base_url.clone()).unwrap_or_default();
            let cache = CachedCatalog {
                version: 1,
                cached_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                account_name: account_name.clone(),
                account_url,
                live_categories: app.all_categories.iter().map(|c| (**c).clone()).collect(),
                live_streams: app.global_all_streams.iter().map(|s| (**s).clone()).collect(),
                vod_categories: app.all_vod_categories.iter().map(|c| (**c).clone()).collect(),
                vod_streams: app.global_all_vod_streams.iter().map(|s| (**s).clone()).collect(),
                series_categories: app.all_series_categories.iter().map(|c| (**c).clone()).collect(),
                series_streams: app.global_all_series_streams.iter().map(|s| (**s).clone()).collect(),
                total_channels: app.total_channels,
                total_movies: app.total_movies,
                total_series: app.total_series,
                category_counts: app.category_channel_counts.clone().into_iter().collect(),
                processing_modes: app.config.processing_modes.clone(),
            };
            let _ = cache.save();

            // Refresh view context-aware:
            // Instead of dumping ALL streams into the view (which overwrites specific category views),
            // re-select the current category to trigger the new auto-filtering logic.
            // This ensures if the user navigated to "Sports" while loading, they see "Sports", not "All".
            let current_idx = app.selected_category_index;
            app.select_category(current_idx);

            // If user triggered this load from the Categories screen on "ALL", switch to Streams
            let on_categories = app.current_screen == crate::app::CurrentScreen::Categories;
            let selected_all = app.categories.get(app.selected_category_index)
                .map(|c| c.category_id == "ALL").unwrap_or(false);
            if on_categories && selected_all && app.state_loading {
                app.current_screen = crate::app::CurrentScreen::Streams;
                app.active_pane = crate::app::Pane::Streams;
                app.category_grid_view = false;
                app.selected_stream_index = 0;
                app.stream_list_state.select(Some(0));
            }
            
            app.state_loading = false;
            app.loading_message = None;
            app.loading_progress = None;
        }
        AsyncAction::TotalMoviesLoaded(mut streams) => {
            let pms = app.config.processing_modes.clone();
            let favs = app.config.favorites.vod_streams.clone(); // Use separate VOD favs if available or generic
            let account_name = app.config.accounts.get(app.selected_account_index)
                .map(|a| a.name.clone()).unwrap_or_default();

            preprocessing::preprocess_streams(&mut streams, &favs, &pms, false, &account_name);

            let count = streams.len();
            app.total_movies = count;
            app.global_all_vod_streams = streams.into_iter().map(Arc::new).collect();
            let tz = app.provider_timezone.clone();
            App::pre_cache_parsed(&mut app.global_all_vod_streams, tz.as_deref());
            if let Some(account) = app.config.accounts.get_mut(app.selected_account_index) {
                account.total_movies = Some(count);
                let _ = app.config.save();
            }

            // Save to cache for instant startup on next launch
            let account_name = app.config.accounts.get(app.selected_account_index)
                .map(|a| a.name.clone()).unwrap_or_default();
            let account_url = app.config.accounts.get(app.selected_account_index)
                .map(|a| a.base_url.clone()).unwrap_or_default();
            let cache = CachedCatalog {
                version: 1,
                cached_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                account_name: account_name.clone(),
                account_url,
                live_categories: app.all_categories.iter().map(|c| (**c).clone()).collect(),
                live_streams: app.global_all_streams.iter().map(|s| (**s).clone()).collect(),
                vod_categories: app.all_vod_categories.iter().map(|c| (**c).clone()).collect(),
                vod_streams: app.global_all_vod_streams.iter().map(|s| (**s).clone()).collect(),
                series_categories: app.all_series_categories.iter().map(|c| (**c).clone()).collect(),
                series_streams: app.global_all_series_streams.iter().map(|s| (**s).clone()).collect(),
                total_channels: app.total_channels,
                total_movies: app.total_movies,
                total_series: app.total_series,
                category_counts: app.category_channel_counts.clone().into_iter().collect(),
                processing_modes: app.config.processing_modes.clone(),
            };
            let _ = cache.save();

            if app.search_mode { app.update_search(); }
            app.state_loading = false;
            app.loading_message = None;
        }
        AsyncAction::TotalSeriesLoaded(mut series) => {
            let pms = app.config.processing_modes.clone();
            let favs = app.config.favorites.vod_streams.clone(); // Series use VOD favorites match 
            let account_name = app.config.accounts.get(app.selected_account_index)
                .map(|a| a.name.clone()).unwrap_or_default();

            // Treat series like VOD for filtering purposes (is_live = false)
            preprocessing::preprocess_streams(&mut series, &favs, &pms, false, &account_name);

            let count = series.len();
            app.total_series = count;
            app.global_all_series_streams = series.into_iter().map(Arc::new).collect();
            let tz = app.provider_timezone.clone();
            App::pre_cache_parsed(&mut app.global_all_series_streams, tz.as_deref());
            if let Some(account) = app.config.accounts.get_mut(app.selected_account_index) {
                account.total_series = Some(count);
                let _ = app.config.save();
            }

            // Save to cache for instant startup on next launch
            let account_name = app.config.accounts.get(app.selected_account_index)
                .map(|a| a.name.clone()).unwrap_or_default();
            let account_url = app.config.accounts.get(app.selected_account_index)
                .map(|a| a.base_url.clone()).unwrap_or_default();
            let cache = CachedCatalog {
                version: 1,
                cached_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                account_name: account_name.clone(),
                account_url,
                live_categories: app.all_categories.iter().map(|c| (**c).clone()).collect(),
                live_streams: app.global_all_streams.iter().map(|s| (**s).clone()).collect(),
                vod_categories: app.all_vod_categories.iter().map(|c| (**c).clone()).collect(),
                vod_streams: app.global_all_vod_streams.iter().map(|s| (**s).clone()).collect(),
                series_categories: app.all_series_categories.iter().map(|c| (**c).clone()).collect(),
                series_streams: app.global_all_series_streams.iter().map(|s| (**s).clone()).collect(),
                total_channels: app.total_channels,
                total_movies: app.total_movies,
                total_series: app.total_series,
                category_counts: app.category_channel_counts.clone().into_iter().collect(),
                processing_modes: app.config.processing_modes.clone(),
            };
            let _ = cache.save();

            if app.search_mode { app.update_search(); }
            app.state_loading = false;
            app.loading_message = None;
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
            let cat_favs = app.config.favorites.categories.clone();
            let vod_cat_favs = app.config.favorites.vod_categories.clone();

            // Spawn a single task that fetches all categories in parallel using join!
            tokio::spawn(async move {
                // Execute all category fetches in parallel
                let (live_result, vod_result, series_result) = join!(
                    client.get_live_categories(),
                    client.get_vod_categories(),
                    client.get_series_categories()
                );

                // Process and send Live Categories
                match live_result {
                    Ok(mut cats) => {
                        preprocessing::preprocess_categories(&mut cats, &cat_favs, &pms, true, false, &account_name);
                        let _ = tx.send(AsyncAction::CategoriesLoaded(cats)).await;
                    }
                    Err(e) => {
                        let _ = tx.send(AsyncAction::Error(format!("Live Categories Error: {}", e))).await;
                    }
                }

                // Process and send VOD Categories
                match vod_result {
                    Ok(mut cats) => {
                        preprocessing::preprocess_categories(&mut cats, &vod_cat_favs, &pms, false, true, &account_name);
                        let _ = tx.send(AsyncAction::VodCategoriesLoaded(cats)).await;
                    }
                    Err(e) => {
                        let _ = tx.send(AsyncAction::Error(format!("VOD Categories Error: {}", e))).await;
                    }
                }

                // Process and send Series Categories
                match series_result {
                    Ok(mut cats) => {
                        preprocessing::preprocess_categories(&mut cats, &cat_favs, &pms, false, false, &account_name);
                        let _ = tx.send(AsyncAction::SeriesCategoriesLoaded(cats)).await;
                    }
                    Err(e) => {
                        let _ = tx.send(AsyncAction::Error(format!("Series Category Error: {}", e))).await;
                    }
                }
            });
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
            app.loading_progress = None;
        }
        AsyncAction::ScanProgress { current, total, eta_secs } => {
            app.loading_progress = Some(crate::errors::LoadingProgress {
                stage: crate::errors::LoadingStage::FetchingStreams { category: "All".to_string() },
                current,
                total,
                eta: Some(std::time::Duration::from_secs(eta_secs)),
            });
        }
    }
}

/// Spawn background Live channel parallel scan (called lazily when user navigates to Live Channels)
/// Uses semaphore-limited concurrency (15 max) with progress reporting and ETA
pub fn spawn_live_scan(app: &App, tx: &mpsc::Sender<AsyncAction>) {
    let client = match &app.current_client {
        Some(c) => c.clone(),
        None => return,
    };
    let tx = tx.clone();
    let stream_favs = app.config.favorites.streams.clone();
    let account_name = app.config.accounts.get(app.selected_account_index)
        .map(|a| a.name.clone()).unwrap_or_default();
    let pms = app.config.processing_modes.clone();
    
    // Clone categories for fallback scenario
    let cat_info: Vec<(String, String)> = app.all_categories.iter()
        .map(|c| (c.category_id.clone(), c.category_name.clone()))
        .collect();

    tokio::spawn(async move {
        // FAST PATH: Try to fetch ALL streams in one go.
        // This is 1 HTTP request vs 500+ requests. Drastically reduces ISP noise.
        let _ = tx.send(AsyncAction::LoadingMessage("Fetching all channels (single request)...".to_string())).await;
        
        match client.get_live_streams("ALL").await {
            Ok(mut all_streams) if !all_streams.is_empty() => {
                let _ = tx.send(AsyncAction::LoadingMessage(format!(
                    "Received {} channels. Processing...", all_streams.len()
                ))).await;
                
                preprocessing::preprocess_streams(&mut all_streams, &stream_favs, &pms, true, &account_name);
                let _ = tx.send(AsyncAction::TotalChannelsLoaded(all_streams)).await;
                return; 
            }
            Ok(_) => {
                let _ = tx.send(AsyncAction::LoadingMessage("Server returned empty list for ALL. Falling back to scan...".to_string())).await;
            }
            Err(e) => {
                let _ = tx.send(AsyncAction::LoadingMessage(format!("Bulk fetch failed: {}. Falling back to scan...", e))).await;
            }
        }

        // SLOW PATH: Fallback to scanning categories if "ALL" is not supported.
        // We strictly limit concurrency to 1 (Sequential) to avoid ISP "excessive connection" alerts.
        let scan_start = std::time::Instant::now();
        let total_cats = cat_info.len();
        
        // ISP Friendly: Sequential loop (one at a time) + Delay
        let sem = std::sync::Arc::new(tokio::sync::Semaphore::new(3));
        let mut all_streams = Vec::new();

        for (i, (cat_id, cat_name)) in cat_info.into_iter().enumerate() {
            let _permit = sem.acquire().await.unwrap();
            
            // Artificial delay to be kind to the network
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;

            let completed = i + 1;
            let pct = (completed * 100) / total_cats;
            let bar_filled = pct / 5;
            let bar_empty = 20 - bar_filled;
            let bar = format!("{}{}", "█".repeat(bar_filled), "░".repeat(bar_empty));
            
            let _ = tx.send(AsyncAction::LoadingMessage(format!(
                "Scanning [{}/{}] {}% [{}] · {}", 
                completed, total_cats, pct, bar, cat_name
            ))).await;

            if let Ok(streams) = client.get_live_streams(&cat_id).await {
                all_streams.extend(streams);
            }
        }

        {
            use std::collections::HashSet;
            let before = all_streams.len();
            let mut seen = HashSet::with_capacity(all_streams.len());
            all_streams.retain(|s| {
                let id = crate::api::get_id_str(&s.stream_id);
                seen.insert(id)
            });
            let _ = tx.send(AsyncAction::LoadingMessage(format!(
                "Deduplicating... {} → {} channels", before, all_streams.len()
            ))).await;
        }

        let _ = tx.send(AsyncAction::LoadingMessage(format!(
            "Processing {} channels...", all_streams.len()
        ))).await;
        preprocessing::preprocess_streams(&mut all_streams, &stream_favs, &pms, true, &account_name);
        let _ = tx.send(AsyncAction::TotalChannelsLoaded(all_streams)).await;
    });
}

/// Spawn background VOD parallel scan (called lazily when user navigates to Movies)
/// Uses semaphore-limited concurrency (15 max) with progress reporting
pub fn spawn_vod_scan(app: &App, tx: &mpsc::Sender<AsyncAction>) {
    let client = match &app.current_client {
        Some(c) => c.clone(),
        None => return,
    };
    let tx = tx.clone();
    let vod_favs = app.config.favorites.vod_streams.clone();
    let account_name = app.config.accounts.get(app.selected_account_index)
        .map(|a| a.name.clone()).unwrap_or_default();
    let pms = app.config.processing_modes.clone();

    tokio::spawn(async move {
        let _ = tx.send(AsyncAction::LoadingMessage("Loading movie categories...".to_string())).await;
        let cats = match client.get_vod_categories().await {
            Ok(cats) => cats,
            Err(_) => {
                let _ = tx.send(AsyncAction::LoadingMessage("Fetching all movies (fallback)...".to_string())).await;
                if let Ok(mut streams) = client.get_vod_streams_all().await {
                    preprocessing::preprocess_streams(&mut streams, &vod_favs, &pms, false, &account_name);
                    let _ = tx.send(AsyncAction::TotalMoviesLoaded(streams)).await;
                }
                return;
            }
        };

        let total_cats = cats.len();
        let _ = tx.send(AsyncAction::LoadingMessage(format!("Scanning {} movie categories...", total_cats))).await;

        // ISP Friendly: Reduced concurrency from 15 -> 3
        let sem = std::sync::Arc::new(tokio::sync::Semaphore::new(3));
        let mut handles: Vec<(String, tokio::task::JoinHandle<Vec<Stream>>)> = Vec::with_capacity(cats.len());
        for cat in &cats {
            let c = client.clone();
            let cat_id = cat.category_id.clone();
            let cat_name = cat.category_name.clone();
            let permit = sem.clone();
            handles.push((cat_name, tokio::spawn(async move {
                let _permit = permit.acquire().await.unwrap();
                // Rate Limiting: 50ms jitter
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                c.get_vod_streams(&cat_id).await.unwrap_or_default()
            })));
        }

        let mut all_streams = Vec::new();
        for (i, (cat_name, handle)) in handles.into_iter().enumerate() {
            if let Ok(streams) = handle.await {
                all_streams.extend(streams);
            }
            let completed = i + 1;
            let pct = (completed * 100) / total_cats;
            let bar_filled = pct / 5;
            let bar_empty = 20 - bar_filled;
            let bar = format!("{}{}", "█".repeat(bar_filled), "░".repeat(bar_empty));
            let _ = tx.send(AsyncAction::LoadingMessage(format!(
                "Loading movies [{}/{}] {}% [{}] · {} ({} found)",
                completed, total_cats, pct, bar, cat_name, all_streams.len()
            ))).await;
        }

        {
            use std::collections::HashSet;
            let mut seen = HashSet::with_capacity(all_streams.len());
            all_streams.retain(|s| {
                let id = crate::api::get_id_str(&s.stream_id);
                seen.insert(id)
            });
        }

        let _ = tx.send(AsyncAction::LoadingMessage(format!(
            "Processing {} movies...", all_streams.len()
        ))).await;
        preprocessing::preprocess_streams(&mut all_streams, &vod_favs, &pms, false, &account_name);
        let _ = tx.send(AsyncAction::TotalMoviesLoaded(all_streams)).await;
    });
}

/// Spawn background Series parallel scan (called lazily when user navigates to Series)
/// Uses semaphore-limited concurrency (15 max) with progress reporting
pub fn spawn_series_scan(app: &App, tx: &mpsc::Sender<AsyncAction>) {
    let client = match &app.current_client {
        Some(c) => c.clone(),
        None => return,
    };
    let tx = tx.clone();
    let series_favs = app.config.favorites.vod_streams.clone();
    let account_name = app.config.accounts.get(app.selected_account_index)
        .map(|a| a.name.clone()).unwrap_or_default();
    let pms = app.config.processing_modes.clone();

    tokio::spawn(async move {
        let _ = tx.send(AsyncAction::LoadingMessage("Loading series categories...".to_string())).await;
        let cats = match client.get_series_categories().await {
            Ok(cats) => cats,
            Err(_) => {
                let _ = tx.send(AsyncAction::LoadingMessage("Fetching all series (fallback)...".to_string())).await;
                if let Ok(mut streams) = client.get_series_all().await {
                    preprocessing::preprocess_streams(&mut streams, &series_favs, &pms, false, &account_name);
                    let _ = tx.send(AsyncAction::TotalSeriesLoaded(streams)).await;
                }
                return;
            }
        };

        let total_cats = cats.len();
        let _ = tx.send(AsyncAction::LoadingMessage(format!("Scanning {} series categories...", total_cats))).await;

        // ISP Friendly: Reduced concurrency from 15 -> 3
        let sem = std::sync::Arc::new(tokio::sync::Semaphore::new(3));
        let mut handles: Vec<(String, tokio::task::JoinHandle<Vec<Stream>>)> = Vec::with_capacity(cats.len());
        for cat in &cats {
            let c = client.clone();
            let cat_id = cat.category_id.clone();
            let cat_name = cat.category_name.clone();
            let permit = sem.clone();
            handles.push((cat_name, tokio::spawn(async move {
                let _permit = permit.acquire().await.unwrap();
                // Rate Limiting: 50ms jitter
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                c.get_series_streams(&cat_id).await.unwrap_or_default()
            })));
        }

        let mut all_streams = Vec::new();
        for (i, (cat_name, handle)) in handles.into_iter().enumerate() {
            if let Ok(streams) = handle.await {
                all_streams.extend(streams);
            }
            let completed = i + 1;
            let pct = (completed * 100) / total_cats;
            let bar_filled = pct / 5;
            let bar_empty = 20 - bar_filled;
            let bar = format!("{}{}", "█".repeat(bar_filled), "░".repeat(bar_empty));
            let _ = tx.send(AsyncAction::LoadingMessage(format!(
                "Loading series [{}/{}] {}% [{}] · {} ({} found)",
                completed, total_cats, pct, bar, cat_name, all_streams.len()
            ))).await;
        }

        {
            use std::collections::HashSet;
            let mut seen = HashSet::with_capacity(all_streams.len());
            all_streams.retain(|s| {
                let id = crate::api::get_id_str(&s.stream_id);
                seen.insert(id)
            });
        }

        let _ = tx.send(AsyncAction::LoadingMessage(format!(
            "Processing {} series...", all_streams.len()
        ))).await;
        preprocessing::preprocess_streams(&mut all_streams, &series_favs, &pms, false, &account_name);
        let _ = tx.send(AsyncAction::TotalSeriesLoaded(all_streams)).await;
    });
}
