use crate::api::{Category, IptvClient, ServerInfo, Stream, UserInfo};
use crate::config::AppConfig;
use crate::state::{
    CategoryManagementState, ContentState, GroupManagementState, LoginFormState, MatrixRainState,
    SearchState, SeriesState, SessionState, SportsState, UiState, VodState,
};
use rayon::prelude::*;
use std::collections::VecDeque;
use std::sync::Arc;
// Parser imports removed as processing moved to background tasks in main.rs
use ratatui::layout::Rect;
use ratatui::widgets::ListState;
use tui_input::Input;

// Casting types - conditionally use real or stub implementation
#[cfg(all(not(target_arch = "wasm32"), feature = "chromecast"))]
pub use crate::cast::CastDevice;

#[cfg(not(all(not(target_arch = "wasm32"), feature = "chromecast")))]
#[derive(Debug, Clone, PartialEq)]
pub struct CastDevice {
    pub name: String,
    pub ip: String,
    pub port: u16,
    pub model: Option<String>,
}

#[derive(Debug, Clone)]
pub enum AsyncAction {
    LoginSuccess(IptvClient, Option<UserInfo>, Option<ServerInfo>),
    LoginFailed(String),
    CategoriesLoaded(Vec<Category>),
    StreamsLoaded(Vec<Stream>, String),
    VodCategoriesLoaded(Vec<Category>),
    VodStreamsLoaded(Vec<Stream>, String),
    SeriesCategoriesLoaded(Vec<Category>),
    SeriesStreamsLoaded(Vec<Stream>, String),
    SeriesInfoLoaded(crate::api::SeriesInfo),
    VodInfoLoaded(crate::api::VodInfo),
    PlayerStarted,
    PlayerFailed(String),
    LoadingMessage(String),
    TotalChannelsLoaded(Vec<Stream>),
    PartialChannelsLoaded(Vec<Stream>),
    TotalMoviesLoaded(Vec<Stream>),
    TotalSeriesLoaded(Vec<Stream>),
    FinalizeChannelsLoaded {
        streams: Vec<Arc<Stream>>,
        counts: std::collections::HashMap<String, usize>,
        by_cat: std::collections::HashMap<String, Vec<Arc<Stream>>>,
    },
    FinalizeMoviesLoaded {
        streams: Vec<Arc<Stream>>,
        by_cat: std::collections::HashMap<String, Vec<Arc<Stream>>>,
    },
    FinalizeSeriesLoaded {
        streams: Vec<Arc<Stream>>,
        by_cat: std::collections::HashMap<String, Vec<Arc<Stream>>>,
    },
    PlaylistRefreshed(IptvClient, Option<UserInfo>, Option<ServerInfo>),
    EpgLoaded(String, String),             // stream_id, program_title
    EpgBatchLoaded(Vec<(String, String)>), // Vec of (stream_id, program_title)
    StreamHealthLoaded(String, u64),       // stream_id, latency_ms
    UpdateAvailable(String),               // new_version
    NoUpdateFound,
    SportsMatchesLoaded(Vec<crate::sports::StreamedMatch>),
    SportsStreamsLoaded(Vec<crate::sports::StreamedStream>),
    ScoresLoaded(Vec<crate::scores::ScoreGame>),
    ScanProgress {
        current: usize,
        total: usize,
        eta_secs: u64,
    },
    // Chromecast casting
    CastDevicesDiscovered(Vec<CastDevice>),
    CastStarted(String), // Device name
    CastFailed(String),  // Error message
    Error(String),

    // Lazy Category Loading (Phase 4)
    LoadLiveStreams(String),   // category_id
    LoadVodStreams(String),    // category_id
    LoadSeriesStreams(String), // category_id
}

#[derive(PartialEq, Debug, Clone)]
pub enum CurrentScreen {
    Home,                 // List of accounts
    Login,                // Add new account form
    Categories,           // List of channel categories
    Streams,              // List of streams in a category
    VodCategories,        // List of VOD categories
    VodStreams,           // List of VODs in a category
    SeriesCategories,     // List of Series categories
    SeriesStreams,        // List of Series (shows as streams/list)
    Settings,             // App settings
    TimezoneSettings,     // Edit Timezone
    Play,                 // (Optional) Info screen before playing
    ContentTypeSelection, // New intermediate screen
    GlobalSearch,         // Ctrl+Space Global Search across all content
    GroupManagement,      // Manage custom groups (create/edit/delete)
    GroupPicker,          // Pick a group to add stream to
    UpdatePrompt,         // Prompt for app update
    SportsDashboard,      // Integrated Live Sports from Streamed.pk
}

#[derive(PartialEq, Debug)]
pub enum InputMode {
    Normal,
    Editing,
}

#[derive(PartialEq)]
pub enum LoginField {
    Name,
    Url,
    Username,
    Password,
    EpgUrl,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Pane {
    Categories,
    Streams,
    Episodes, // Third column for series episodes
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Guide {
    WhatIsApp,
    HowToGetPlaylists,
    WhatIsIptv,
}

pub struct App {
    pub config: AppConfig,
    pub current_screen: CurrentScreen,
    pub input_mode: InputMode,
    pub should_quit: bool,
    pub needs_clear: bool,

    // Home / Accounts
    pub account_list_state: ListState,

    // Login Form
    pub login_field_focus: LoginField,
    pub input_name: Input,
    pub input_url: Input,
    pub input_username: Input,
    pub input_password: Input,
    pub input_epg_url: Input,
    pub input_server_timezone: Input,
    pub login_error: Option<String>,

    // Categories
    pub all_categories: Vec<Arc<Category>>,
    pub categories: Vec<Arc<Category>>,
    pub selected_category_index: usize,
    pub category_list_state: ratatui::widgets::TableState,

    // Streams
    pub all_streams: Vec<Arc<Stream>>,
    pub streams: Vec<Arc<Stream>>,
    pub selected_stream_index: usize,
    pub stream_list_state: ListState,

    // VOD Categories
    pub all_vod_categories: Vec<Arc<Category>>,
    pub vod_categories: Vec<Arc<Category>>,
    pub selected_vod_category_index: usize,
    pub vod_category_list_state: ratatui::widgets::TableState,

    // VOD Streams
    pub all_vod_streams: Vec<Arc<Stream>>,
    pub vod_streams: Vec<Arc<Stream>>,
    pub selected_vod_stream_index: usize,
    pub vod_stream_list_state: ListState,

    // Series Data
    pub all_series_categories: Vec<Arc<Category>>,
    pub series_categories: Vec<Arc<Category>>,
    pub selected_series_category_index: usize,
    pub series_category_list_state: ratatui::widgets::TableState,

    pub all_series_streams: Vec<Arc<Stream>>, // Series are treated as 'Streams' for listing
    pub series_streams: Vec<Arc<Stream>>,
    pub selected_series_stream_index: usize,
    pub series_stream_list_state: ListState,

    // Series Episodes (for 3-column view)
    pub series_episodes: Vec<crate::api::SeriesEpisode>,
    pub selected_series_episode_index: usize,
    pub series_episode_list_state: ListState,
    pub current_series_info: Option<crate::api::SeriesInfo>,
    pub current_vod_info: Option<crate::api::VodInfo>,

    // Global caches for "ALL" categories
    pub global_all_streams: Vec<Arc<Stream>>,
    pub global_streams_by_cat: std::collections::HashMap<String, Vec<Arc<Stream>>>,
    pub global_all_vod_streams: Vec<Arc<Stream>>,
    pub global_vod_streams_by_cat: std::collections::HashMap<String, Vec<Arc<Stream>>>,
    pub global_all_series_streams: Vec<Arc<Stream>>,
    pub global_series_streams_by_cat: std::collections::HashMap<String, Vec<Arc<Stream>>>,

    // Settings
    pub playlist_mode_list_state: ListState,
    pub settings_options: Vec<String>,
    pub settings_descriptions: Vec<String>,
    pub selected_settings_index: usize,
    pub settings_list_state: ListState,
    pub selected_content_type_index: usize,

    // Timezone selection
    pub input_timezone: Input,
    pub timezone_list: Vec<String>,
    pub timezone_list_state: ListState,

    pub last_search_query: String,

    // DNS selection
    pub dns_list_state: ListState,

    // Video Mode selection
    pub video_mode_list_state: ListState,

    // Player Engine selection
    pub player_engine_list_state: ListState,

    // Auto-Refresh selection
    pub auto_refresh_list_state: ListState,

    // Editing
    pub editing_account_index: Option<usize>,

    // 2-Pane Navigation
    pub active_pane: Pane,
    pub category_grid_view: bool, // true = grid tiles, false = list
    pub grid_cols: usize,         // set by renderer, read by input handler

    // Search/Filter
    pub search_state: SearchState,
    pub search_mode: bool,
    // Track last query for incremental narrowing

    // Help
    pub show_help: bool,
    pub show_guide: Option<Guide>,
    pub guide_scroll: u16,

    // Sub-states
    pub settings_state: SettingsState,
    pub previous_screen: Option<CurrentScreen>,
    pub show_save_confirmation: bool,

    // About content
    pub about_text: String,
    pub about_scroll: u16,
    pub loading_log: std::collections::VecDeque<String>,
    pub needs_stream_refresh: bool,
    #[cfg(not(target_arch = "wasm32"))]
    pub last_category_move: Option<std::time::Instant>,
    pub is_navigating_categories: bool,

    // Layout tracking for mouse support
    pub area_categories: Rect,
    pub area_streams: Rect,
    pub area_episodes: Rect,
    pub area_accounts: Rect,

    // FTUE (First Time User Experience)
    pub show_matrix_rain: bool,
    #[cfg(not(target_arch = "wasm32"))]
    pub matrix_rain_start_time: Option<std::time::Instant>,
    #[cfg(target_arch = "wasm32")]
    pub matrix_rain_start_time: Option<f64>,
    pub matrix_rain_screensaver_mode: bool, // true = screensaver (no logo), false = startup (with logo)
    pub show_welcome_popup: bool,
    pub matrix_rain_columns: Vec<MatrixColumn>,
    pub matrix_rain_logo_hits: Vec<bool>, // Tracks which logo pixels have been "activated"

    // Screen Transition Effects (tachyonfx)
    #[cfg(not(target_arch = "wasm32"))]
    pub transition_effect: Option<tachyonfx::Effect>,
    pub transition_last_screen: Option<CurrentScreen>,
    #[cfg(not(target_arch = "wasm32"))]
    pub frame_instant: std::time::Instant,

    // EPG Enrichment
    pub epg_cache: std::collections::HashMap<String, String>,
    pub last_focused_stream_id: Option<String>,
    #[cfg(not(target_arch = "wasm32"))]
    pub focus_timestamp: Option<std::time::Instant>,
    #[cfg(target_arch = "wasm32")]
    pub focus_timestamp: Option<f64>,

    // Global Search
    pub global_search_results: Vec<Arc<Stream>>,
    pub global_search_list_state: ListState,

    // Group Management
    pub selected_group_index: usize,
    pub group_list_state: ListState,
    pub pending_stream_for_group: Option<String>, // Stream ID waiting to be added to a group
    pub group_name_input: String,                 // For creating/renaming groups
    pub pending_play_url: Option<String>,
    pub pending_play_title: Option<String>,
    pub show_play_details: bool,
    pub new_version_available: Option<String>,

    // Sports Dashboard (Streamed.pk)
    pub sports_matches: Vec<crate::sports::StreamedMatch>,
    pub sports_list_state: ListState,
    pub sports_categories: Vec<String>,
    pub sports_category_list_state: ListState,
    pub selected_sports_category_index: usize,
    pub current_sports_streams: Vec<crate::sports::StreamedStream>,
    pub sports_details_loading: bool,

    // Live Scores (ESPN)
    pub live_scores: Vec<crate::scores::ScoreGame>,

    // Chromecast Casting
    #[cfg(all(not(target_arch = "wasm32"), feature = "chromecast"))]
    pub cast_manager: crate::cast::CastManager,
    pub cast_devices: Vec<CastDevice>,
    pub cast_device_list_state: ListState,
    pub show_cast_picker: bool,
    pub cast_discovering: bool,
    pub selected_cast_device_index: usize,

    // UX improvements
    #[cfg(not(target_arch = "wasm32"))]
    pub scan_start_time: Option<std::time::Instant>, // For loading ETA
    #[cfg(not(target_arch = "wasm32"))]
    pub last_search_update: Option<std::time::Instant>, // 150ms debounce gate
    pub category_channel_counts: std::collections::HashMap<String, usize>, // Counts per category_id

    // --- Decomposed State Structs (Phase 6) ---
    /// Session state for provider connection
    pub session: SessionState,
    /// Live channels content state
    pub live: ContentState,
    /// VOD movies content state
    pub vod: VodState,
    /// Series content state
    pub series: SeriesState,
    /// Login form state
    pub login_form: LoginFormState,
    /// UI navigation state
    pub ui: UiState,
    /// Sports state
    pub sports: SportsState,
    /// Matrix rain animation state
    pub matrix_rain: MatrixRainState,
    /// Group management state
    pub groups: GroupManagementState,
    /// Category management state
    pub category_mgmt: CategoryManagementState,
    pub pending_lazy_loads: std::collections::VecDeque<AsyncAction>,
}

#[derive(Clone)]
pub struct MatrixColumn {
    pub x: u16,
    pub y: u16,
    pub length: u16,
    pub speed: u16,
    pub chars: Vec<char>,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum SettingsState {
    Main,
    ManageAccounts,
    DnsSelection,
    VideoModeSelection,
    PlayerEngineSelection,
    PlaylistModeSelection,
    AutoRefreshSelection,
    CategoryManagement,
    About,
}

impl App {
    pub fn new() -> App {
        let config = AppConfig::load().unwrap_or_default();
        let mut account_list_state = ListState::default();
        if !config.accounts.is_empty() {
            account_list_state.select(Some(0));
        }

        // Load ABOUT.md if exists
        let about_text = std::fs::read_to_string("ABOUT.md")
            .unwrap_or_else(|_| "Vibe IPTV CLI\n\nBuilt by Ernesto \"Beats\"".to_string());

        let _show_ftue = config.accounts.is_empty();
        // Always show matrix rain animation on startup (3 seconds)
        #[cfg(not(target_arch = "wasm32"))]
        let matrix_rain_start = Some(std::time::Instant::now());
        #[cfg(target_arch = "wasm32")]
        let matrix_rain_start = web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now());

        let cached_tz = config.get_user_timezone();

        let mut app = App {
            config,
            current_screen: CurrentScreen::Home,
            input_mode: InputMode::Normal,
            should_quit: false,
            needs_clear: false,
            session: SessionState {
                cached_user_timezone: cached_tz,
                selected_account_index: 0,
                max_category_name_len: 10,
                ..Default::default()
            },

            epg_cache: std::collections::HashMap::new(),
            last_focused_stream_id: None,
            focus_timestamp: None,

            editing_account_index: None,

            account_list_state,

            login_field_focus: LoginField::Name,
            input_name: Input::default(),
            input_url: Input::default(),
            input_username: Input::default(),
            input_password: Input::default(),
            input_epg_url: Input::default(),
            input_server_timezone: Input::default(),
            login_error: None,

            all_categories: vec![],
            categories: vec![],
            selected_category_index: 0,
            category_list_state: ratatui::widgets::TableState::default(),

            all_streams: vec![],
            streams: vec![],
            selected_stream_index: 0,
            stream_list_state: ListState::default(),

            all_vod_categories: vec![],
            vod_categories: Vec::new(),
            selected_vod_category_index: 0,
            vod_category_list_state: ratatui::widgets::TableState::default(),

            // VOD Streams
            all_vod_streams: Vec::new(),
            vod_streams: Vec::new(),
            selected_vod_stream_index: 0,
            vod_stream_list_state: ListState::default(),

            // Series Data
            all_series_categories: Vec::new(),
            series_categories: Vec::new(),
            selected_series_category_index: 0,
            series_category_list_state: ratatui::widgets::TableState::default(),
            all_series_streams: vec![],
            series_streams: vec![],
            selected_series_stream_index: 0,
            series_stream_list_state: ListState::default(),

            // Series Episodes Init
            series_episodes: vec![],
            selected_series_episode_index: 0,
            series_episode_list_state: ListState::default(),
            current_series_info: None,
            current_vod_info: None,

            // Global Caches
            global_all_streams: vec![],
            global_streams_by_cat: std::collections::HashMap::new(),
            global_all_vod_streams: vec![],
            global_vod_streams_by_cat: std::collections::HashMap::new(),
            global_all_series_streams: vec![],
            global_series_streams_by_cat: std::collections::HashMap::new(),

            global_search_results: vec![],
            global_search_list_state: ListState::default(),

            settings_options: vec![],
            settings_descriptions: vec![],
            selected_settings_index: 0,
            settings_list_state: ListState::default(),
            selected_content_type_index: 0,
            input_timezone: Input::default(),
            timezone_list: vec![
                "UTC".to_string(),
                "America/New_York".to_string(),
                "America/Chicago".to_string(),
                "America/Denver".to_string(),
                "America/Los_Angeles".to_string(),
                "America/Toronto".to_string(),
                "America/Vancouver".to_string(),
                "Europe/London".to_string(),
                "Europe/Paris".to_string(),
                "Europe/Berlin".to_string(),
                "Europe/Amsterdam".to_string(),
                "Asia/Tokyo".to_string(),
                "Asia/Shanghai".to_string(),
                "Asia/Singapore".to_string(),
                "Australia/Sydney".to_string(),
                "Pacific/Auckland".to_string(),
            ],
            timezone_list_state: ListState::default(),
            dns_list_state: ListState::default(),
            video_mode_list_state: ListState::default(),
            player_engine_list_state: ListState::default(),
            auto_refresh_list_state: ListState::default(),

            active_pane: Pane::Categories,
            category_grid_view: false,
            grid_cols: 3, // default, overwritten by renderer
            search_state: SearchState::new(),
            search_mode: false,
            last_search_query: String::new(),
            show_help: false,
            show_guide: None,
            guide_scroll: 0,
            settings_state: SettingsState::Main,
            previous_screen: None,
            show_save_confirmation: false,
            needs_stream_refresh: false,
            last_category_move: None,
            is_navigating_categories: false,
            about_text,
            about_scroll: 0,
            area_categories: Rect::default(),
            area_streams: Rect::default(),
            area_episodes: Rect::default(),
            area_accounts: Rect::default(),

            // Matrix rain: Always show on startup for 3 seconds
            show_matrix_rain: true,
            matrix_rain_start_time: matrix_rain_start,
            matrix_rain_screensaver_mode: false, // Startup mode (with logo)
            show_welcome_popup: false,
            matrix_rain_columns: vec![],
            matrix_rain_logo_hits: vec![false; 101 * 6], // 101 wide x 6 high logo

            // Screen Transition Effects
            #[cfg(not(target_arch = "wasm32"))]
            transition_effect: None,
            transition_last_screen: None,
            #[cfg(not(target_arch = "wasm32"))]
            frame_instant: std::time::Instant::now(),

            playlist_mode_list_state: ListState::default(),

            // Group Management
            selected_group_index: 0,
            group_list_state: ListState::default(),
            pending_stream_for_group: None,
            group_name_input: String::new(),
            pending_play_url: None,
            pending_play_title: None,
            show_play_details: false,
            new_version_available: None,

            // Sports Dashboard
            sports_matches: Vec::new(),
            sports_list_state: ListState::default(),
            sports_categories: vec![
                "live".to_string(),
                "all-today".to_string(),
                "football".to_string(),
                "basketball".to_string(),
                "f1".to_string(),
                "ufc".to_string(),
                "tennis".to_string(),
                "baseball".to_string(),
                "hockey".to_string(),
            ],
            sports_category_list_state: ListState::default(),
            selected_sports_category_index: 0,
            current_sports_streams: Vec::new(),
            sports_details_loading: false,

            // Live Scores
            live_scores: Vec::new(),

            // Chromecast Casting
            #[cfg(all(not(target_arch = "wasm32"), feature = "chromecast"))]
            cast_manager: crate::cast::CastManager::new(),
            cast_devices: Vec::new(),
            cast_device_list_state: ListState::default(),
            show_cast_picker: false,
            cast_discovering: false,
            selected_cast_device_index: 0,

            // UX improvements
            #[cfg(not(target_arch = "wasm32"))]
            scan_start_time: None,
            #[cfg(not(target_arch = "wasm32"))]
            last_search_update: None,
            category_channel_counts: std::collections::HashMap::new(),

            // --- Decomposed State Structs ---
            // Initialized above
            live: ContentState::new(),
            vod: VodState::new(),
            series: SeriesState::new(),
            login_form: LoginFormState::new(),
            ui: UiState::new(),
            sports: SportsState::new(),
            matrix_rain: MatrixRainState::new(),
            groups: GroupManagementState::new(),
            category_mgmt: CategoryManagementState::new(),
            loading_log: VecDeque::with_capacity(30),
            pending_lazy_loads: std::collections::VecDeque::new(),
        };

        app.refresh_settings_options();
        app.apply_category_filters(); // Apply initial filters
        app
    }

    pub fn toggle_category_visibility(&mut self, category_id: String) {
        if let Some(acc) = self
            .config
            .accounts
            .get_mut(self.session.selected_account_index)
        {
            if acc.hidden_categories.contains(&category_id) {
                acc.hidden_categories.remove(&category_id);
            } else {
                acc.hidden_categories.insert(category_id);
            }
            let _ = self.config.save();
        }
        self.apply_category_filters();
    }

    pub fn cycle_category_sort_order(&mut self) {
        if let Some(acc) = self
            .config
            .accounts
            .get_mut(self.session.selected_account_index)
        {
            acc.category_sort_order = acc.category_sort_order.next();
            let _ = self.config.save();
        }
        self.apply_category_filters();
    }

    pub fn apply_category_filters(&mut self) {
        let acc = match self
            .config
            .accounts
            .get(self.session.selected_account_index)
        {
            Some(a) => a,
            None => return,
        };

        let sort_order = acc.category_sort_order;
        let hidden = &acc.hidden_categories;

        let pms = &self.config.processing_modes;
        let use_merica = pms.contains(&crate::config::ProcessingMode::Merica);
        let use_all_english = pms.contains(&crate::config::ProcessingMode::AllEnglish);

        // Helper for filtering/sorting
        let process = |cats: &[Arc<Category>],
                       hidden: &std::collections::HashSet<String>,
                       order: crate::config::CategorySortOrder,
                       is_vod: bool|
         -> Vec<Arc<Category>> {
            let mut filtered: Vec<_> = cats
                .iter()
                .filter(|c| {
                    if c.category_id == "ALL" || c.category_id == "FAVORITES" {
                        return true;
                    }
                    if hidden.contains(&c.category_id) {
                        return false;
                    }

                    // Apply 'Merica/English filtering
                    if is_vod {
                        if (use_merica || use_all_english)
                            && !crate::parser::is_english_vod(&c.category_name)
                        {
                            return false;
                        }
                    } else if use_merica {
                        if !crate::parser::is_american_live(&c.category_name) {
                            return false;
                        }
                    } else if use_all_english {
                        if !crate::parser::is_english_live(&c.category_name) {
                            return false;
                        }
                    }

                    true
                })
                .cloned()
                .collect();

            match order {
                crate::config::CategorySortOrder::Alphabetical => {
                    filtered.sort_by(|a, b| {
                        if a.category_id == "ALL" || a.category_id == "FAVORITES" {
                            return std::cmp::Ordering::Less;
                        }
                        if b.category_id == "ALL" || b.category_id == "FAVORITES" {
                            return std::cmp::Ordering::Greater;
                        }
                        a.category_name
                            .to_lowercase()
                            .cmp(&b.category_name.to_lowercase())
                    });
                }
                crate::config::CategorySortOrder::ZtoA => {
                    filtered.sort_by(|a, b| {
                        if a.category_id == "ALL" || a.category_id == "FAVORITES" {
                            return std::cmp::Ordering::Less;
                        }
                        if b.category_id == "ALL" || b.category_id == "FAVORITES" {
                            return std::cmp::Ordering::Greater;
                        }
                        b.category_name
                            .to_lowercase()
                            .cmp(&a.category_name.to_lowercase())
                    });
                }
                _ => {} // Server order
            }
            filtered
        };

        self.categories = process(&self.all_categories, hidden, sort_order, false);
        self.vod_categories = process(&self.all_vod_categories, hidden, sort_order, true);
        self.series_categories = process(&self.all_series_categories, hidden, sort_order, true);
    }

    pub fn on_channels_loaded(&mut self, streams: Vec<Stream>, reset_selection: bool) {
        let wrapped: Vec<Arc<Stream>> = streams.into_iter().map(Arc::new).collect();

        if reset_selection {
            self.global_all_streams = wrapped.clone();
            self.all_streams = wrapped;
            self.current_screen = CurrentScreen::Streams;
            self.active_pane = Pane::Streams;
            self.update_search();
            self.session.state_loading = false;
            self.session.loading_message = None;
        } else {
            // Partial update during pipelined ingestion:
            // Append and update counts
            for s in wrapped {
                if let Some(ref cid) = s.category_id {
                    *self.category_channel_counts.entry(cid.clone()).or_insert(0) += 1;
                }
                self.global_all_streams.push(s);
            }

            // Sync with current view if we are on the streams screen
            if self.current_screen == CurrentScreen::Streams
                || self.current_screen == CurrentScreen::Categories
            {
                self.update_search();
            }
        }
    }

    pub fn get_score_for_stream(&self, stream_name: &str) -> Option<&crate::scores::ScoreGame> {
        // Strip leading emojis/icons from team names (preprocessing adds 🏀, etc.)
        fn strip_emoji_prefix(s: &str) -> String {
            s.chars()
                .skip_while(|c| !c.is_ascii_alphanumeric() && !c.is_ascii_whitespace())
                .collect::<String>()
                .trim()
                .to_lowercase()
        }

        // Strategy 1: Try to parse as "Team1 vs Team2" matchup
        if let Some(event) = crate::sports::parse_sports_event(stream_name) {
            let t1 = strip_emoji_prefix(&event.team1);
            let t2 = strip_emoji_prefix(&event.team2);

            // Strict match: Both teams must be in the game
            if let Some(game) = self.live_scores.iter().find(|game| {
                let g_home = game.home_team.to_lowercase();
                let g_away = game.away_team.to_lowercase();

                let t1_in_game = g_home.contains(&t1)
                    || g_away.contains(&t1)
                    || t1.contains(&g_home)
                    || t1.contains(&g_away);
                let t2_in_game = g_home.contains(&t2)
                    || g_away.contains(&t2)
                    || t2.contains(&g_home)
                    || t2.contains(&g_away);

                t1_in_game && t2_in_game
            }) {
                return Some(game);
            }
        }

        // Strategy 2: Fallback - match by single team name in stream
        // Useful for channel names like "SPECTRUM SPORTSNET LAKERS" or "YES NETWORK"
        let stream_lower = stream_name.to_lowercase();

        // Common team name keywords to search for
        self.live_scores.iter().find(|game| {
            // Extract team short names (last word, e.g., "Lakers" from "Los Angeles Lakers")
            let home_short = game
                .home_team
                .split_whitespace()
                .last()
                .unwrap_or("")
                .to_lowercase();
            let away_short = game
                .away_team
                .split_whitespace()
                .last()
                .unwrap_or("")
                .to_lowercase();

            // Check if stream contains either team's short name AS A FULL WORD
            // Use regex or splitting to avoid partial matches (e.g. "magic" in "magical")
            let has_word = |name: &str, target: &str| {
                if target.len() < 4 {
                    return false;
                }
                // simple word boundary check
                let mut padded = String::with_capacity(name.len() + 2);
                padded.push(' ');
                padded.push_str(name);
                padded.push(' ');

                let mut target_pad = String::with_capacity(target.len() + 2);
                target_pad.push(' ');
                target_pad.push_str(target);
                target_pad.push(' ');

                padded.contains(&target_pad)
            };

            has_word(&stream_lower, &home_short) || has_word(&stream_lower, &away_short)
        })
    }

    pub fn refresh_settings_options(&mut self) {
        self.settings_options = vec![
            "Manage Playlists".to_string(),
            format!(
                "Set Timezone (Current: {})",
                self.config.get_user_timezone()
            ),
            format!(
                "Playlist Filters: {}",
                if self.config.processing_modes.is_empty() {
                    "None".to_string()
                } else {
                    self.config
                        .processing_modes
                        .iter()
                        .map(|m| match m {
                            crate::config::ProcessingMode::Merica => "'merica",
                            crate::config::ProcessingMode::Sports => "Sports",
                            crate::config::ProcessingMode::AllEnglish => "All English",
                        })
                        .collect::<Vec<_>>()
                        .join(" + ")
                }
            ),
            format!("DNS Provider: {}", self.config.dns_provider.display_name()),
            format!(
                "Video Mode: {}",
                if self.config.use_default_mpv {
                    "MPV Default"
                } else {
                    "Enhanced"
                }
            ),
            format!(
                "Player Engine: {}",
                self.config.preferred_player.display_name()
            ),
            format!(
                "Smooth Motion: {}",
                if self.config.smooth_motion {
                    "ON"
                } else {
                    "OFF"
                }
            ),
            format!(
                "Auto-Refresh: {}",
                if self.config.auto_refresh_hours == 0 {
                    "Disabled".to_string()
                } else {
                    format!("Every {}h", self.config.auto_refresh_hours)
                }
            ),
            "Matrix Rain Screensaver".to_string(),
            "Check for Updates".to_string(),
            "Manage Category Visibility".to_string(),
            "About".to_string(),
        ];

        // Descriptions for each setting (same order as settings_options)
        self.settings_descriptions = vec![
            "Add, edit, or remove IPTV playlist connections.".to_string(),
            "Set your local timezone for accurate program scheduling.".to_string(),
            "Playlist Mode: Change how playlists are processed and displayed. e.g. 'merica mode for US sports, Sports mode for global athletics, etc.".to_string(),
            "Choose DNS provider for network requests. Quad9 recommended for privacy.".to_string(),
            "Enhanced = Interpolation/Upscaling (MPV only). MPV Default = No enhancements.".to_string(),
            "Switch between MPV (High Performance) and VLC (High Stability) playback engines.".to_string(),
            "Enables motion interpolation to double the perceived frame-rate (works on MPV and VLC).".to_string(),
            "How often to automatically refresh playlist data when logging in. Set to 0 to disable.".to_string(),
            "Launch the iconic Matrix digital rain animation.".to_string(),
            "Check if a newer version of Matrix IPTV is available for download.".to_string(),
            "Hide or show specific playlist categories for a cleaner experience.".to_string(),
            "View application info, version, and credits.".to_string(),
        ];

        if self.settings_list_state.selected().is_none() {
            self.settings_list_state.select(Some(0));
        }
    }

    pub fn get_selected_account(&self) -> Option<&crate::config::Account> {
        self.config
            .accounts
            .get(self.session.selected_account_index)
    }

    pub fn get_selected_category(&self) -> Option<&Arc<Category>> {
        self.categories.get(self.selected_category_index)
    }

    pub fn get_selected_stream(&self) -> Option<&Arc<Stream>> {
        self.streams.get(self.selected_stream_index)
    }

    pub fn next_timezone(&mut self) {
        let len = self.timezone_list.len();
        if len > 0 {
            let i = match self.timezone_list_state.selected() {
                Some(i) => (i + 1) % len,
                None => 0,
            };
            self.timezone_list_state.select(Some(i));
        }
    }

    pub fn previous_timezone(&mut self) {
        let len = self.timezone_list.len();
        if len > 0 {
            let i = match self.timezone_list_state.selected() {
                Some(i) => {
                    if i == 0 {
                        len - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.timezone_list_state.select(Some(i));
        }
    }

    pub fn toggle_input_mode(&mut self) {
        self.input_mode = match self.input_mode {
            InputMode::Normal => InputMode::Editing,
            InputMode::Editing => InputMode::Normal,
        };
    }

    fn navigate_list(
        len: usize,
        current_index: &mut usize,
        list_state: &mut ListState,
        forward: bool,
    ) {
        if len == 0 {
            return;
        }
        let i = match list_state.selected() {
            Some(i) => {
                if forward {
                    (i + 1) % len
                } else if i == 0 {
                    len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        *current_index = i;
        list_state.select(Some(i));
    }

    pub fn jump_list(
        len: usize,
        current_index: &mut usize,
        list_state: &mut ListState,
        amount: usize,
        forward: bool,
    ) {
        if len == 0 {
            return;
        }
        let current = list_state.selected().unwrap_or(0);
        let new_idx = if forward {
            (current + amount).min(len - 1)
        } else {
            current.saturating_sub(amount)
        };
        *current_index = new_idx;
        list_state.select(Some(new_idx));
    }

    fn jump_list_top(len: usize, current_index: &mut usize, list_state: &mut ListState) {
        if len == 0 {
            return;
        }
        *current_index = 0;
        list_state.select(Some(0));
    }

    fn jump_list_bottom(len: usize, current_index: &mut usize, list_state: &mut ListState) {
        if len == 0 {
            return;
        }
        *current_index = len - 1;
        list_state.select(Some(len - 1));
    }

    pub fn page_size_for_pane(&self, pane: Pane) -> usize {
        let area = match pane {
            Pane::Categories => self.area_categories,
            Pane::Streams => self.area_streams,
            Pane::Episodes => self.area_episodes,
        };
        (area.height.saturating_sub(2) as usize).max(1)
    }

    pub fn next_account(&mut self) {
        let prev = self.session.selected_account_index;
        Self::navigate_list(
            self.config.accounts.len(),
            &mut self.session.selected_account_index,
            &mut self.account_list_state,
            true,
        );
        if self.session.selected_account_index != prev {
            // Switched to a different account — invalidate cached session
            // so re-entry will authenticate with the new account's credentials.
            self.session.current_client = None;
        }
    }

    pub fn previous_account(&mut self) {
        let prev = self.session.selected_account_index;
        Self::navigate_list(
            self.config.accounts.len(),
            &mut self.session.selected_account_index,
            &mut self.account_list_state,
            false,
        );
        if self.session.selected_account_index != prev {
            self.session.current_client = None;
        }
    }

    pub fn next_category(&mut self) {
        self.move_category_y(true);
    }

    pub fn previous_category(&mut self) {
        self.move_category_y(false);
    }

    pub fn get_current_category_indices(&self) -> (usize, usize) {
        match self.current_screen {
            CurrentScreen::VodCategories | CurrentScreen::VodStreams => {
                (self.vod_categories.len(), self.selected_vod_category_index)
            }
            CurrentScreen::SeriesCategories | CurrentScreen::SeriesStreams => (
                self.series_categories.len(),
                self.selected_series_category_index,
            ),
            _ => (self.categories.len(), self.selected_category_index),
        }
    }

    fn apply_category_move(&mut self, new_index: usize) {
        match self.current_screen {
            CurrentScreen::VodCategories | CurrentScreen::VodStreams => {
                self.select_vod_category(new_index)
            }
            CurrentScreen::SeriesCategories | CurrentScreen::SeriesStreams => {
                self.select_series_category(new_index)
            }
            _ => self.select_category(new_index),
        }
    }

    pub fn move_category_x(&mut self, is_right: bool) {
        let (len, selected) = self.get_current_category_indices();
        if len == 0 {
            return;
        }

        let cols = self.grid_cols.max(1);
        let rows = (len + cols - 1) / cols;
        let full_cols = if len % cols == 0 { cols } else { len % cols };

        let mut c = 0;
        let mut r = selected;
        for col in 0..cols {
            let col_size = if col < full_cols { rows } else { rows - 1 };
            if r < col_size {
                c = col;
                break;
            }
            r -= col_size;
        }

        let next_c = if is_right {
            if c + 1 < cols {
                c + 1
            } else {
                c
            }
        } else {
            c.saturating_sub(1)
        };

        let next_col_size = if next_c < full_cols { rows } else { rows - 1 };
        let next_r = if r >= next_col_size {
            next_col_size.saturating_sub(1)
        } else {
            r
        };

        let mut next = 0;
        for col in 0..next_c {
            next += if col < full_cols { rows } else { rows - 1 };
        }

        self.apply_category_move(next + next_r);
    }

    pub fn move_category_y(&mut self, is_down: bool) {
        let (len, selected) = self.get_current_category_indices();
        if len == 0 {
            return;
        }

        let cols = self.grid_cols.max(1);
        let rows = (len + cols - 1) / cols;
        let full_cols = if len % cols == 0 { cols } else { len % cols };

        let mut c = 0;
        let mut r = selected;
        for col in 0..cols {
            let col_size = if col < full_cols { rows } else { rows - 1 };
            if r < col_size {
                c = col;
                break;
            }
            r -= col_size;
        }

        let col_size = if c < full_cols { rows } else { rows - 1 };

        if is_down {
            if r + 1 < col_size {
                r += 1;
            }
        } else {
            r = r.saturating_sub(1);
        }

        let mut next = 0;
        for col in 0..c {
            next += if col < full_cols { rows } else { rows - 1 };
        }

        self.apply_category_move(next + r);
    }

    pub fn page_down_category(&mut self) {
        let page = self.page_size_for_pane(Pane::Categories);
        let (len, cur) = self.get_current_category_indices();
        let new_idx = (cur + page).min(len.saturating_sub(1));
        self.apply_category_move(new_idx);
    }

    pub fn page_up_category(&mut self) {
        let page = self.page_size_for_pane(Pane::Categories);
        let (len, cur) = self.get_current_category_indices();
        if len == 0 {
            return;
        }
        let new_idx = cur.saturating_sub(page);
        self.apply_category_move(new_idx);
    }

    pub fn half_page_down_category(&mut self) {
        let half = self.page_size_for_pane(Pane::Categories) / 2;
        let (len, cur) = self.get_current_category_indices();
        let new_idx = (cur + half.max(1)).min(len.saturating_sub(1));
        self.apply_category_move(new_idx);
    }

    pub fn half_page_up_category(&mut self) {
        let half = self.page_size_for_pane(Pane::Categories) / 2;
        let (len, cur) = self.get_current_category_indices();
        if len == 0 {
            return;
        }
        let new_idx = cur.saturating_sub(half.max(1));
        self.apply_category_move(new_idx);
    }

    pub fn jump_to_category(&mut self, index: usize) {
        if index < self.categories.len() {
            self.select_category(index);
        }
    }

    pub fn jump_to_category_bottom(&mut self) {
        let (len, _) = self.get_current_category_indices();
        if len > 0 {
            self.apply_category_move(len - 1);
        }
    }

    pub fn jump_to_category_top(&mut self) {
        let (len, _) = self.get_current_category_indices();
        if len > 0 {
            self.apply_category_move(0);
        }
    }

    pub fn select_category(&mut self, index: usize) {
        if index >= self.categories.len() {
            return;
        }

        // Track navigation for debounce
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.last_category_move = Some(std::time::Instant::now());
            self.is_navigating_categories = true;
        }

        self.selected_category_index = index;
        self.category_list_state
            .select(Some(index / self.grid_cols.max(1)));

        self.selected_stream_index = 0;
        self.stream_list_state.select(Some(0));

        self.needs_stream_refresh = true;
    }

    pub fn select_vod_category(&mut self, index: usize) {
        if index >= self.vod_categories.len() {
            return;
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.last_category_move = Some(std::time::Instant::now());
            self.is_navigating_categories = true;
        }

        self.selected_vod_category_index = index;
        self.vod_category_list_state.select(Some(index));

        self.needs_stream_refresh = true;
    }

    pub fn select_series_category(&mut self, index: usize) {
        if index >= self.series_categories.len() {
            return;
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.last_category_move = Some(std::time::Instant::now());
            self.is_navigating_categories = true;
        }

        self.selected_series_category_index = index;
        self.series_category_list_state.select(Some(index));

        self.needs_stream_refresh = true;
    }

    pub fn refresh_streams_from_cache(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        if self.is_navigating_categories {
            if let Some(last) = self.last_category_move {
                if last.elapsed() < std::time::Duration::from_millis(70) {
                    self.needs_stream_refresh = true; // Stay dirty to try again next frame
                    return;
                }
            }
            self.is_navigating_categories = false;
        }

        if !self.categories.is_empty() && self.selected_category_index < self.categories.len() {
            let cat_id = self.categories[self.selected_category_index]
                .category_id
                .clone();
            if cat_id == "ALL" {
                self.all_streams = self.global_all_streams.clone();
            } else if let Some(streams) = self.global_streams_by_cat.get(&cat_id) {
                self.all_streams = streams.clone();
            } else if !self.session.state_loading && self.current_screen == CurrentScreen::Streams {
                // Trigger lazy load for Live
                if !self
                    .pending_lazy_loads
                    .iter()
                    .any(|a| matches!(a, AsyncAction::LoadLiveStreams(ref id) if id == &cat_id))
                {
                    self.pending_lazy_loads
                        .push_back(AsyncAction::LoadLiveStreams(cat_id));
                    self.session.state_loading = true;
                    self.session.loading_message =
                        Some("System Protocol: Uploading Category Streamset...".to_string());
                }
            }
        }

        if !self.vod_categories.is_empty()
            && self.selected_vod_category_index < self.vod_categories.len()
        {
            let cat_id = self.vod_categories[self.selected_vod_category_index]
                .category_id
                .clone();
            if cat_id == "ALL" {
                self.all_vod_streams = self.global_all_vod_streams.clone();
            } else if let Some(streams) = self.global_vod_streams_by_cat.get(&cat_id) {
                self.all_vod_streams = streams.clone();
            } else if !self.session.state_loading
                && self.current_screen == CurrentScreen::VodStreams
            {
                // Trigger lazy load for VOD
                if !self
                    .pending_lazy_loads
                    .iter()
                    .any(|a| matches!(a, AsyncAction::LoadVodStreams(ref id) if id == &cat_id))
                {
                    self.pending_lazy_loads
                        .push_back(AsyncAction::LoadVodStreams(cat_id));
                    self.session.state_loading = true;
                }
            }
        }

        if !self.series_categories.is_empty()
            && self.selected_series_category_index < self.series_categories.len()
        {
            let cat_id = self.series_categories[self.selected_series_category_index]
                .category_id
                .clone();
            if cat_id == "ALL" {
                self.all_series_streams = self.global_all_series_streams.clone();
            } else if let Some(streams) = self.global_series_streams_by_cat.get(&cat_id) {
                self.all_series_streams = streams.clone();
            } else if !self.session.state_loading
                && self.current_screen == CurrentScreen::SeriesStreams
            {
                // Trigger lazy load for Series
                if !self
                    .pending_lazy_loads
                    .iter()
                    .any(|a| matches!(a, AsyncAction::LoadSeriesStreams(ref id) if id == &cat_id))
                {
                    self.pending_lazy_loads
                        .push_back(AsyncAction::LoadSeriesStreams(cat_id));
                    self.session.state_loading = true;
                }
            }
        }

        self.update_search();
    }

    pub fn next_stream(&mut self) {
        Self::navigate_list(
            self.streams.len(),
            &mut self.selected_stream_index,
            &mut self.stream_list_state,
            true,
        );
    }

    pub fn page_down_stream(&mut self) {
        let page = self.page_size_for_pane(Pane::Streams);
        Self::jump_list(
            self.streams.len(),
            &mut self.selected_stream_index,
            &mut self.stream_list_state,
            page,
            true,
        );
    }

    pub fn page_up_stream(&mut self) {
        let page = self.page_size_for_pane(Pane::Streams);
        Self::jump_list(
            self.streams.len(),
            &mut self.selected_stream_index,
            &mut self.stream_list_state,
            page,
            false,
        );
    }

    pub fn half_page_down_stream(&mut self) {
        let half = self.page_size_for_pane(Pane::Streams) / 2;
        Self::jump_list(
            self.streams.len(),
            &mut self.selected_stream_index,
            &mut self.stream_list_state,
            half.max(1),
            true,
        );
    }

    pub fn half_page_up_stream(&mut self) {
        let half = self.page_size_for_pane(Pane::Streams) / 2;
        Self::jump_list(
            self.streams.len(),
            &mut self.selected_stream_index,
            &mut self.stream_list_state,
            half.max(1),
            false,
        );
    }

    pub fn jump_to_stream(&mut self, index: usize) {
        if index < self.streams.len() {
            self.selected_stream_index = index;
            self.stream_list_state.select(Some(index));
        }
    }

    pub fn jump_to_bottom(&mut self) {
        Self::jump_list_bottom(
            self.streams.len(),
            &mut self.selected_stream_index,
            &mut self.stream_list_state,
        );
    }

    pub fn jump_to_top(&mut self) {
        Self::jump_list_top(
            self.streams.len(),
            &mut self.selected_stream_index,
            &mut self.stream_list_state,
        );
    }

    pub fn previous_stream(&mut self) {
        Self::navigate_list(
            self.streams.len(),
            &mut self.selected_stream_index,
            &mut self.stream_list_state,
            false,
        );
    }

    pub fn next_vod_category(&mut self) {
        let new_idx = (self.selected_vod_category_index + 1) % self.vod_categories.len().max(1);
        self.select_vod_category(new_idx);
    }

    pub fn previous_vod_category(&mut self) {
        let len = self.vod_categories.len();
        let new_idx = (self.selected_vod_category_index + len - 1) % len.max(1);
        self.select_vod_category(new_idx);
    }

    pub fn next_vod_stream(&mut self) {
        Self::navigate_list(
            self.vod_streams.len(),
            &mut self.selected_vod_stream_index,
            &mut self.vod_stream_list_state,
            true,
        );
    }

    pub fn page_down_vod_stream(&mut self) {
        let page = self.page_size_for_pane(Pane::Streams);
        Self::jump_list(
            self.vod_streams.len(),
            &mut self.selected_vod_stream_index,
            &mut self.vod_stream_list_state,
            page,
            true,
        );
    }

    pub fn page_up_vod_stream(&mut self) {
        let page = self.page_size_for_pane(Pane::Streams);
        Self::jump_list(
            self.vod_streams.len(),
            &mut self.selected_vod_stream_index,
            &mut self.vod_stream_list_state,
            page,
            false,
        );
    }

    pub fn half_page_down_vod_stream(&mut self) {
        let half = self.page_size_for_pane(Pane::Streams) / 2;
        Self::jump_list(
            self.vod_streams.len(),
            &mut self.selected_vod_stream_index,
            &mut self.vod_stream_list_state,
            half.max(1),
            true,
        );
    }

    pub fn half_page_up_vod_stream(&mut self) {
        let half = self.page_size_for_pane(Pane::Streams) / 2;
        Self::jump_list(
            self.vod_streams.len(),
            &mut self.selected_vod_stream_index,
            &mut self.vod_stream_list_state,
            half.max(1),
            false,
        );
    }

    pub fn jump_to_vod_stream(&mut self, index: usize) {
        if index < self.vod_streams.len() {
            self.selected_vod_stream_index = index;
            self.vod_stream_list_state.select(Some(index));
        }
    }

    pub fn jump_to_vod_bottom(&mut self) {
        Self::jump_list_bottom(
            self.vod_streams.len(),
            &mut self.selected_vod_stream_index,
            &mut self.vod_stream_list_state,
        );
    }

    pub fn jump_to_vod_top(&mut self) {
        Self::jump_list_top(
            self.vod_streams.len(),
            &mut self.selected_vod_stream_index,
            &mut self.vod_stream_list_state,
        );
    }

    pub fn previous_vod_stream(&mut self) {
        Self::navigate_list(
            self.vod_streams.len(),
            &mut self.selected_vod_stream_index,
            &mut self.vod_stream_list_state,
            false,
        );
    }

    pub fn next_setting(&mut self) {
        Self::navigate_list(
            self.settings_options.len(),
            &mut self.selected_settings_index,
            &mut self.settings_list_state,
            true,
        );
    }

    pub fn previous_setting(&mut self) {
        Self::navigate_list(
            self.settings_options.len(),
            &mut self.selected_settings_index,
            &mut self.settings_list_state,
            false,
        );
    }

    pub fn next_global_search_result(&mut self) {
        Self::navigate_list(
            self.global_search_results.len(),
            &mut self.selected_stream_index,
            &mut self.global_search_list_state,
            true,
        );
    }

    pub fn page_down_global_search(&mut self) {
        let page = self.page_size_for_pane(Pane::Streams);
        Self::jump_list(
            self.global_search_results.len(),
            &mut self.selected_stream_index,
            &mut self.global_search_list_state,
            page,
            true,
        );
    }

    pub fn page_up_global_search(&mut self) {
        let page = self.page_size_for_pane(Pane::Streams);
        Self::jump_list(
            self.global_search_results.len(),
            &mut self.selected_stream_index,
            &mut self.global_search_list_state,
            page,
            false,
        );
    }

    pub fn jump_to_global_search_result(&mut self, index: usize) {
        if index < self.global_search_results.len() {
            self.selected_stream_index = index;
            self.global_search_list_state.select(Some(index));
        }
    }

    pub fn jump_to_global_search_bottom(&mut self) {
        Self::jump_list_bottom(
            self.global_search_results.len(),
            &mut self.selected_stream_index,
            &mut self.global_search_list_state,
        );
    }

    pub fn jump_to_global_search_top(&mut self) {
        Self::jump_list_top(
            self.global_search_results.len(),
            &mut self.selected_stream_index,
            &mut self.global_search_list_state,
        );
    }

    pub fn previous_global_search_result(&mut self) {
        Self::navigate_list(
            self.global_search_results.len(),
            &mut self.selected_stream_index,
            &mut self.global_search_list_state,
            false,
        );
    }

    /// Pre-populate cached_parsed for all visible streams to avoid per-frame parsing.
    /// Uses Arc::make_mut() which only clones if refcount > 1 (copy-on-write).
    pub fn pre_cache_parsed(
        streams: &mut [Arc<Stream>],
        provider_tz: Option<&str>,
        tx: Option<tokio::sync::mpsc::Sender<crate::app::AsyncAction>>,
        msg_prefix: &str,
    ) {
        let total = streams.len();
        for (i, s) in streams.iter_mut().enumerate() {
            if i % 2000 == 0 {
                if let Some(ref sender) = tx {
                    let _ = sender.blocking_send(crate::app::AsyncAction::LoadingMessage(format!(
                        "{} {} / {}...",
                        msg_prefix, i, total
                    )));
                }
            }
            if s.cached_parsed.is_none() {
                let inner = Arc::make_mut(s);
                inner.cached_parsed = Some(Box::new(crate::parser::parse_stream(
                    &inner.name,
                    provider_tz,
                )));
            }
        }
    }

    /// Build a map of category_id → channel count from global_all_streams,
    /// and pre-compute O(1) filtering HashMaps to enable instantaneous zero-latency navigation.
    pub fn build_category_indices(&mut self) {
        self.category_channel_counts.clear();
        self.global_streams_by_cat.clear();
        self.global_vod_streams_by_cat.clear();
        self.global_series_streams_by_cat.clear();

        // Live Streams
        for s in &self.global_all_streams {
            if let Some(ref cid) = s.category_id {
                *self.category_channel_counts.entry(cid.clone()).or_insert(0) += 1;
                self.global_streams_by_cat
                    .entry(cid.clone())
                    .or_default()
                    .push(s.clone());
            }
        }

        // VOD Streams
        for s in &self.global_all_vod_streams {
            if let Some(ref cid) = s.category_id {
                self.global_vod_streams_by_cat
                    .entry(cid.clone())
                    .or_default()
                    .push(s.clone());
            }
        }

        // Series Streams
        for s in &self.global_all_series_streams {
            if let Some(ref cid) = s.category_id {
                self.global_series_streams_by_cat
                    .entry(cid.clone())
                    .or_default()
                    .push(s.clone());
            }
        }
    }

    /// Record a recently watched channel. Deduplicates by stream_id, caps at 20.
    pub fn record_recently_watched(&mut self, stream_id: String, stream_name: String) {
        // Remove existing entry for this stream_id (dedup)
        self.config
            .recently_watched
            .retain(|(id, _)| id != &stream_id);
        // Push to front
        self.config
            .recently_watched
            .insert(0, (stream_id, stream_name));
        // Cap at 20
        self.config.recently_watched.truncate(20);
        let _ = self.config.save();
    }

    /// Update search with debouncing and fuzzy matching
    /// Phase 3: Incremental Search Narrowing - only re-filter when query changes meaningfully
    pub fn update_search(&mut self) {
        // 150ms debounce gate — only during active search to avoid
        // skipping handler-triggered population calls (TotalChannelsLoaded etc.)
        #[cfg(not(target_arch = "wasm32"))]
        if self.search_mode {
            if let Some(last) = self.last_search_update {
                if last.elapsed() < std::time::Duration::from_millis(150) {
                    return;
                }
            }
            self.last_search_update = Some(std::time::Instant::now());
        }

        let query = self.search_state.query.to_lowercase();
        let query_changed = query != self.last_search_query;
        // Update the last search query
        self.last_search_query = query.clone();

        // Minimum character threshold: skip filtering for very short queries
        // unless clearing (empty query should reset the view)
        const MIN_QUERY_LENGTH: usize = 2;
        if !query.is_empty() && query.len() < MIN_QUERY_LENGTH {
            return; // Wait for more characters before filtering
        }
        let is_merica = self.config.playlist_mode.is_merica_variant();

        match self.current_screen {
            CurrentScreen::Categories | CurrentScreen::Streams => {
                // Detect context change and clear stack
                if self.search_state.last_screen.as_ref() != Some(&self.current_screen)
                    || self.search_state.last_pane.as_ref() != Some(&self.active_pane)
                {
                    self.search_state.narrow_stack.clear();
                }
                self.search_state.last_screen = Some(self.current_screen.clone());
                self.search_state.last_pane = Some(self.active_pane);

                match self.active_pane {
                    Pane::Categories => {
                        // Categories usually small enough for full re-filter - Parallelize ONLY if massive
                        if self.all_categories.len() > 1000 {
                            self.categories = self
                                .all_categories
                                .par_iter()
                                .filter(|c| {
                                    c.search_name.contains(&query) && (!is_merica || c.is_american)
                                })
                                .cloned()
                                .collect();
                        } else {
                            self.categories = self
                                .all_categories
                                .iter()
                                .filter(|c| {
                                    c.search_name.contains(&query) && (!is_merica || c.is_american)
                                })
                                .cloned()
                                .collect();
                        }

                        if query_changed {
                            self.selected_category_index = 0;
                            self.category_list_state
                                .select(if self.categories.is_empty() {
                                    None
                                } else {
                                    Some(0)
                                });
                        }

                        // Cross-pane stream search (All Channels)
                        if !query.is_empty() && !self.global_all_streams.is_empty() {
                            let mut stream_results: Vec<Arc<Stream>> = self
                                .global_all_streams
                                .par_iter()
                                .filter(|s| {
                                    if is_merica && !s.is_american {
                                        return false;
                                    }
                                    if s.search_name.contains(&query) {
                                        return true;
                                    }
                                    query.len() >= 4 && s.fuzzy_match(&query, 60)
                                })
                                .cloned()
                                .collect();
                            stream_results.sort_by_cached_key(|s| !s.search_name.contains(&query));
                            self.streams = stream_results.into_iter().take(1000).collect();
                        } else if query.is_empty() && !self.all_streams.is_empty() {
                            // FAST PATH: Direct cloning/limiting for base view
                            if !is_merica {
                                self.streams =
                                    self.all_streams.iter().take(1000).cloned().collect();
                            } else {
                                self.streams = self
                                    .all_streams
                                    .iter()
                                    .filter(|s| s.is_american)
                                    .take(1000)
                                    .cloned()
                                    .collect();
                            }
                        }

                        if query_changed {
                            self.selected_stream_index = 0;
                            self.stream_list_state.select(if self.streams.is_empty() {
                                None
                            } else {
                                Some(0)
                            });
                        }
                    }
                    Pane::Streams => {
                        if query.is_empty() {
                            if !is_merica {
                                self.streams =
                                    self.all_streams.iter().take(1000).cloned().collect();
                            } else {
                                self.streams = self
                                    .all_streams
                                    .iter()
                                    .filter(|s| s.is_american)
                                    .take(1000)
                                    .cloned()
                                    .collect();
                            }
                            self.search_state.narrow_stack.clear();
                        } else {
                            // Incremental Narrowing Logic
                            let (base_list, is_narrowing) = if query
                                .starts_with(&self.last_search_query)
                                && !self.last_search_query.is_empty()
                                && !self.streams.is_empty()
                            {
                                // Narrowing: filter current result set
                                (&self.streams, true)
                            } else {
                                // Widening or Jump: search stack or full list
                                while let Some((len, _)) = self.search_state.narrow_stack.last() {
                                    if *len >= query.len() {
                                        self.search_state.narrow_stack.pop();
                                    } else {
                                        break;
                                    }
                                }

                                if let Some((_, cached_results)) =
                                    self.search_state.narrow_stack.last()
                                {
                                    (cached_results, false)
                                } else {
                                    (&self.all_streams, false)
                                }
                            };

                            let mut results: Vec<Arc<Stream>> = base_list
                                .par_iter()
                                .filter(|s| {
                                    if is_merica && !s.is_american {
                                        return false;
                                    }
                                    if s.search_name.contains(&query) {
                                        return true;
                                    }
                                    query.len() >= 4 && s.fuzzy_match(&query, 60)
                                })
                                .cloned()
                                .collect();

                            // Result classification (Sort exact matches higher)
                            results.sort_by_cached_key(|s| !s.search_name.contains(&query));

                            self.streams = results.into_iter().take(1000).collect();

                            // Push to stack if not narrowing (or if we want to store progress)
                            if !is_narrowing || self.search_state.narrow_stack.is_empty() {
                                self.search_state
                                    .narrow_stack
                                    .push((query.len(), self.streams.clone()));
                            }
                        }

                        App::pre_cache_parsed(
                            &mut self.streams,
                            self.session.provider_timezone.as_deref(),
                            None,
                            "",
                        );
                        self.selected_stream_index = 0;
                        if !self.streams.is_empty() {
                            self.stream_list_state.select(Some(0));
                        } else {
                            self.stream_list_state.select(None);
                        }
                    }
                    Pane::Episodes => {}
                }
            }
            CurrentScreen::VodCategories | CurrentScreen::VodStreams => match self.active_pane {
                Pane::Categories => {
                    if self.all_vod_categories.len() > 1000 {
                        self.vod_categories = self
                            .all_vod_categories
                            .par_iter()
                            .filter(|c| {
                                c.search_name.contains(&query) && (!is_merica || c.is_english)
                            })
                            .cloned()
                            .collect();
                    } else {
                        self.vod_categories = self
                            .all_vod_categories
                            .iter()
                            .filter(|c| {
                                c.search_name.contains(&query) && (!is_merica || c.is_english)
                            })
                            .cloned()
                            .collect();
                    }
                    if query_changed {
                        self.selected_vod_category_index = 0;
                        self.vod_category_list_state
                            .select(if self.vod_categories.is_empty() {
                                None
                            } else {
                                Some(0)
                            });
                    }
                }
                Pane::Streams => {
                    if query.is_empty() {
                        if !is_merica {
                            self.vod_streams =
                                self.all_vod_streams.iter().take(1000).cloned().collect();
                        } else {
                            self.vod_streams = self
                                .all_vod_streams
                                .iter()
                                .filter(|s| s.is_english)
                                .take(1000)
                                .cloned()
                                .collect();
                        }
                        self.search_state.narrow_stack.clear();
                    } else {
                        let (base_list, is_narrowing) = if query
                            .starts_with(&self.last_search_query)
                            && !self.last_search_query.is_empty()
                            && !self.vod_streams.is_empty()
                        {
                            (&self.vod_streams, true)
                        } else {
                            while let Some((len, _)) = self.search_state.narrow_stack.last() {
                                if *len >= query.len() {
                                    self.search_state.narrow_stack.pop();
                                } else {
                                    break;
                                }
                            }
                            if let Some((_, cached)) = self.search_state.narrow_stack.last() {
                                (cached, false)
                            } else {
                                (&self.all_vod_streams, false)
                            }
                        };

                        let mut results: Vec<Arc<Stream>> = base_list
                            .par_iter()
                            .filter(|s| {
                                if is_merica && !s.is_english {
                                    return false;
                                }
                                if s.search_name.contains(&query) {
                                    return true;
                                }
                                query.len() >= 4 && s.fuzzy_match(&query, 60)
                            })
                            .cloned()
                            .collect();

                        results.sort_by_cached_key(|s| !s.search_name.contains(&query));
                        self.vod_streams = results.into_iter().take(1000).collect();

                        if !is_narrowing || self.search_state.narrow_stack.is_empty() {
                            self.search_state
                                .narrow_stack
                                .push((query.len(), self.vod_streams.clone()));
                        }
                    }

                    App::pre_cache_parsed(
                        &mut self.vod_streams,
                        self.session.provider_timezone.as_deref(),
                        None,
                        "",
                    );
                    if query_changed {
                        self.selected_vod_stream_index = 0;
                        self.vod_stream_list_state
                            .select(if self.vod_streams.is_empty() {
                                None
                            } else {
                                Some(0)
                            });
                    }
                }
                Pane::Episodes => {}
            },
            CurrentScreen::SeriesCategories | CurrentScreen::SeriesStreams => {
                match self.active_pane {
                    Pane::Categories => {
                        if self.all_series_categories.len() > 1000 {
                            self.series_categories = self
                                .all_series_categories
                                .par_iter()
                                .filter(|c| {
                                    c.search_name.contains(&query) && (!is_merica || c.is_english)
                                })
                                .cloned()
                                .collect();
                        } else {
                            self.series_categories = self
                                .all_series_categories
                                .iter()
                                .filter(|c| {
                                    c.search_name.contains(&query) && (!is_merica || c.is_english)
                                })
                                .cloned()
                                .collect();
                        }
                        if query_changed {
                            self.selected_series_category_index = 0;
                            self.series_category_list_state.select(
                                if self.series_categories.is_empty() {
                                    None
                                } else {
                                    Some(0)
                                },
                            );
                        }
                    }
                    Pane::Streams => {
                        if query.is_empty() {
                            if !is_merica {
                                self.series_streams =
                                    self.all_series_streams.iter().take(1000).cloned().collect();
                            } else {
                                self.series_streams = self
                                    .all_series_streams
                                    .iter()
                                    .filter(|s| s.is_english)
                                    .take(1000)
                                    .cloned()
                                    .collect();
                            }
                            self.search_state.narrow_stack.clear();
                        } else {
                            let (base_list, is_narrowing) = if query
                                .starts_with(&self.last_search_query)
                                && !self.last_search_query.is_empty()
                                && !self.series_streams.is_empty()
                            {
                                (&self.series_streams, true)
                            } else {
                                while let Some((len, _)) = self.search_state.narrow_stack.last() {
                                    if *len >= query.len() {
                                        self.search_state.narrow_stack.pop();
                                    } else {
                                        break;
                                    }
                                }
                                if let Some((_, cached)) = self.search_state.narrow_stack.last() {
                                    (cached, false)
                                } else {
                                    (&self.all_series_streams, false)
                                }
                            };

                            let mut results: Vec<Arc<Stream>> = base_list
                                .par_iter()
                                .filter(|s| {
                                    if is_merica && !s.is_english {
                                        return false;
                                    }
                                    if s.search_name.contains(&query) {
                                        return true;
                                    }
                                    query.len() >= 4 && s.fuzzy_match(&query, 60)
                                })
                                .cloned()
                                .collect();

                            results.sort_by_cached_key(|s| !s.search_name.contains(&query));
                            self.series_streams = results.into_iter().take(1000).collect();

                            if !is_narrowing || self.search_state.narrow_stack.is_empty() {
                                self.search_state
                                    .narrow_stack
                                    .push((query.len(), self.series_streams.clone()));
                            }
                        }

                        App::pre_cache_parsed(
                            &mut self.series_streams,
                            self.session.provider_timezone.as_deref(),
                            None,
                            "",
                        );
                        if query_changed {
                            self.selected_series_stream_index = 0;
                            self.series_stream_list_state.select(
                                if self.series_streams.is_empty() {
                                    None
                                } else {
                                    Some(0)
                                },
                            );
                        }
                    }
                    Pane::Episodes => {}
                }
            }
            CurrentScreen::GlobalSearch => {
                let results = if query.is_empty() {
                    Vec::new()
                } else {
                    // Multi-pass prioritized search
                    let use_fuzzy = query.len() >= 3;
                    let mut hits: Vec<Arc<Stream>> = self
                        .global_all_streams
                        .par_iter()
                        .filter(|s| {
                            s.search_name.contains(&query)
                                || (use_fuzzy && s.fuzzy_match(&query, 70))
                        })
                        .chain(self.global_all_vod_streams.par_iter().filter(|s| {
                            s.search_name.contains(&query)
                                || (use_fuzzy && s.fuzzy_match(&query, 70))
                        }))
                        .chain(self.global_all_series_streams.par_iter().filter(|s| {
                            s.search_name.contains(&query)
                                || (use_fuzzy && s.fuzzy_match(&query, 70))
                        }))
                        .cloned()
                        .collect();

                    // Prioritize exact substring matches
                    hits.sort_by_cached_key(|s| !s.search_name.contains(&query));
                    hits.into_iter().take(100).collect()
                };

                self.global_search_results = results;
                App::pre_cache_parsed(
                    &mut self.global_search_results,
                    self.session.provider_timezone.as_deref(),
                    None,
                    "",
                );
                self.selected_stream_index = 0;
                if !self.global_search_results.is_empty() {
                    self.global_search_list_state.select(Some(0));
                } else {
                    self.global_search_list_state.select(None);
                }
            }
            _ => {}
        }

        // Recalculate max_category_name_len for current category list
        self.session.max_category_name_len = self
            .categories
            .iter()
            .map(|c| {
                crate::parser::parse_category(&c.category_name)
                    .display_name
                    .len()
            })
            .max()
            .unwrap_or(10);
    }

    // Series Navigation Helpers
    pub fn next_series_category(&mut self) {
        let new_idx =
            (self.selected_series_category_index + 1) % self.series_categories.len().max(1);
        self.select_series_category(new_idx);
    }

    pub fn previous_series_category(&mut self) {
        let len = self.series_categories.len();
        let new_idx = (self.selected_series_category_index + len - 1) % len.max(1);
        self.select_series_category(new_idx);
    }

    pub fn next_series_stream(&mut self) {
        Self::navigate_list(
            self.series_streams.len(),
            &mut self.selected_series_stream_index,
            &mut self.series_stream_list_state,
            true,
        );
    }

    pub fn page_down_series_stream(&mut self) {
        let page = self.page_size_for_pane(Pane::Streams);
        Self::jump_list(
            self.series_streams.len(),
            &mut self.selected_series_stream_index,
            &mut self.series_stream_list_state,
            page,
            true,
        );
    }

    pub fn page_up_series_stream(&mut self) {
        let page = self.page_size_for_pane(Pane::Streams);
        Self::jump_list(
            self.series_streams.len(),
            &mut self.selected_series_stream_index,
            &mut self.series_stream_list_state,
            page,
            false,
        );
    }

    pub fn half_page_down_series_stream(&mut self) {
        let half = self.page_size_for_pane(Pane::Streams) / 2;
        Self::jump_list(
            self.series_streams.len(),
            &mut self.selected_series_stream_index,
            &mut self.series_stream_list_state,
            half.max(1),
            true,
        );
    }

    pub fn half_page_up_series_stream(&mut self) {
        let half = self.page_size_for_pane(Pane::Streams) / 2;
        Self::jump_list(
            self.series_streams.len(),
            &mut self.selected_series_stream_index,
            &mut self.series_stream_list_state,
            half.max(1),
            false,
        );
    }

    pub fn page_down_series_episode(&mut self) {
        let page = self.page_size_for_pane(Pane::Episodes);
        Self::jump_list(
            self.series_episodes.len(),
            &mut self.selected_series_episode_index,
            &mut self.series_episode_list_state,
            page,
            true,
        );
    }

    pub fn page_up_series_episode(&mut self) {
        let page = self.page_size_for_pane(Pane::Episodes);
        Self::jump_list(
            self.series_episodes.len(),
            &mut self.selected_series_episode_index,
            &mut self.series_episode_list_state,
            page,
            false,
        );
    }

    pub fn jump_to_series_stream(&mut self, index: usize) {
        if index < self.series_streams.len() {
            self.selected_series_stream_index = index;
            self.series_stream_list_state.select(Some(index));
        }
    }

    pub fn jump_to_series_bottom(&mut self) {
        Self::jump_list_bottom(
            self.series_streams.len(),
            &mut self.selected_series_stream_index,
            &mut self.series_stream_list_state,
        );
    }

    pub fn jump_to_series_top(&mut self) {
        Self::jump_list_top(
            self.series_streams.len(),
            &mut self.selected_series_stream_index,
            &mut self.series_stream_list_state,
        );
    }

    pub fn previous_series_stream(&mut self) {
        Self::navigate_list(
            self.series_streams.len(),
            &mut self.selected_series_stream_index,
            &mut self.series_stream_list_state,
            false,
        );
    }

    pub fn next_series_episode(&mut self) {
        Self::navigate_list(
            self.series_episodes.len(),
            &mut self.selected_series_episode_index,
            &mut self.series_episode_list_state,
            true,
        );
    }

    pub fn previous_series_episode(&mut self) {
        Self::navigate_list(
            self.series_episodes.len(),
            &mut self.selected_series_episode_index,
            &mut self.series_episode_list_state,
            false,
        );
    }

    pub fn save_account(&mut self) {
        use crate::config::Account;

        let name = self.input_name.value().to_string();
        let url = self.input_url.value().to_string();
        let user = self.input_username.value().to_string();
        let pass = self.input_password.value().to_string();
        let epg = self.input_epg_url.value().to_string();
        let epg_opt = if epg.is_empty() { None } else { Some(epg) };

        let tz_str = self.input_server_timezone.value().to_string();
        let tz_opt = if tz_str.is_empty() {
            None
        } else {
            Some(tz_str)
        };

        if !name.is_empty() && !url.is_empty() {
            // Sanitize URL
            let mut final_url = url.trim().to_string();
            if !final_url.starts_with("http://") && !final_url.starts_with("https://") {
                final_url = format!("http://{}", final_url);
            }
            if final_url.ends_with('/') {
                final_url.pop();
            }

            let acc = Account {
                name,
                base_url: final_url.clone(),
                username: user.clone(),
                password: pass.clone(),
                account_type: if Self::is_m3u_url(&final_url, &user, &pass) {
                    crate::config::AccountType::M3uUrl
                } else {
                    crate::config::AccountType::Xtream
                },
                epg_url: epg_opt,
                last_refreshed: None,
                total_channels: None,
                total_movies: None,
                total_series: None,
                server_timezone: tz_opt,
                hidden_categories: std::collections::HashSet::new(),
                category_sort_order: crate::config::CategorySortOrder::Default,
            };

            if let Some(idx) = self.editing_account_index {
                self.config.update_account(idx, acc);
            } else {
                self.config.add_account(acc);
            }
        }

        // Reset inputs
        self.input_name = Input::default();
        self.input_url = Input::default();
        self.input_username = Input::default();
        self.input_password = Input::default();
        self.input_epg_url = Input::default();
        self.input_server_timezone = Input::default();
        self.editing_account_index = None;
        self.login_error = None;
    }

    /// Detect if a URL is an M3U playlist URL rather than an Xtream Codes server
    pub fn is_m3u_url(url: &str, username: &str, password: &str) -> bool {
        let url_lower = url.to_lowercase();

        // If both username and password are empty, it's likely M3U
        if username.is_empty() && password.is_empty() {
            return true;
        }

        // Check for M3U file extensions
        if url_lower.ends_with(".m3u") || url_lower.ends_with(".m3u8") {
            return true;
        }

        // Check URL patterns: strip query string for extension check
        if let Some(path) = url_lower.split('?').next() {
            if path.ends_with(".m3u") || path.ends_with(".m3u8") {
                return true;
            }
        }

        // Check for common M3U URL patterns in query string
        if url_lower.contains("type=m3u")
            || url_lower.contains("output=m3u")
            || url_lower.contains("type=m3u_plus")
            || url_lower.contains("output=ts")
        {
            return true;
        }

        // Check for get.php pattern (common M3U CDN pattern)
        if url_lower.contains("/get.php")
            && (url_lower.contains("type=m3u") || url_lower.contains("output="))
        {
            return true;
        }

        false
    }

    /// Handles a key event and returns an optional AsyncAction to be spawned.
    /// This allows testing the logic without running the full TUI.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> Option<AsyncAction> {
        use crossterm::event::KeyCode;

        // Global
        if self.input_mode == InputMode::Normal {
            if let KeyCode::Char('q') = key.code {
                self.should_quit = true;
                return None;
            }
        }

        match self.current_screen {
            CurrentScreen::Home => {
                match key.code {
                    KeyCode::Char('x') => {
                        // Logic for Series Entry
                        if !self.config.accounts.is_empty() {
                            // We can't easily spawn the async task here without the client credential details
                            // For testing purposes, we might just return a "signal" or set loading state.
                            // In a real refactor, we would return an Action enum like `Action::FetchSeries(account_index)`
                            // key-handling logic should primarily update synchronous state.
                            self.session.state_loading = true;
                            // For testing: we can assert that state_loading became true.
                        }
                    }
                    // ... other Home keys
                    _ => {}
                }
            }
            CurrentScreen::ContentTypeSelection => match key.code {
                KeyCode::Char('1') => {
                    self.current_screen = CurrentScreen::Categories;
                    self.active_pane = Pane::Categories;
                    self.search_mode = false;
                    self.search_state.query.clear();
                }
                KeyCode::Char('2') => {
                    self.current_screen = CurrentScreen::VodCategories;
                    self.active_pane = Pane::Categories;
                    self.search_mode = false;
                    self.search_state.query.clear();
                }
                KeyCode::Char('3') => {
                    self.current_screen = CurrentScreen::SeriesCategories;
                    self.active_pane = Pane::Categories;
                    self.search_mode = false;
                    self.search_state.query.clear();
                }
                KeyCode::Esc | KeyCode::Backspace => {
                    self.current_screen = CurrentScreen::Home;
                }
                _ => {}
            },
            CurrentScreen::Categories | CurrentScreen::Streams => {
                if self.search_mode {
                    match key.code {
                        KeyCode::Esc => {
                            self.search_mode = false;
                            self.search_state.query.clear();
                        }
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Esc | KeyCode::Backspace => {
                            self.current_screen = CurrentScreen::ContentTypeSelection;
                            self.search_mode = false;
                            self.search_state.query.clear();
                        }
                        KeyCode::Char('j') | KeyCode::Down => {
                            if self.active_pane == Pane::Categories {
                                self.move_category_y(true);
                            } else {
                                self.next_stream();
                            }
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            if self.active_pane == Pane::Categories {
                                self.move_category_y(false);
                            } else {
                                self.previous_stream();
                            }
                        }
                        KeyCode::Char('l') | KeyCode::Right => {
                            if self.active_pane == Pane::Categories {
                                self.move_category_x(true);
                            }
                        }
                        KeyCode::Char('h') | KeyCode::Left => {
                            if self.active_pane == Pane::Categories {
                                self.move_category_x(false);
                            }
                        }
                        KeyCode::Char('g') => {
                            if self.active_pane == Pane::Categories {
                                self.jump_to_category_bottom();
                            } else {
                                self.jump_to_bottom();
                            }
                        }
                        KeyCode::Char('G') => {
                            if self.active_pane == Pane::Categories {
                                self.jump_to_category_top();
                            } else {
                                self.jump_to_top();
                            }
                        }
                        KeyCode::Char('0') => {
                            if self.active_pane == Pane::Categories {
                                self.jump_to_category_top();
                            } else {
                                self.jump_to_top();
                            }
                        }
                        KeyCode::Char('1') => {
                            if self.active_pane == Pane::Categories {
                                self.jump_to_category(0);
                            } else {
                                self.jump_to_stream(0);
                            }
                        }
                        KeyCode::Char('2') => {
                            if self.active_pane == Pane::Categories {
                                self.jump_to_category(1);
                            } else {
                                self.jump_to_stream(1);
                            }
                        }
                        KeyCode::Char('3') => {
                            if self.active_pane == Pane::Categories {
                                self.jump_to_category(2);
                            } else {
                                self.jump_to_stream(2);
                            }
                        }
                        KeyCode::Char('4') => {
                            if self.active_pane == Pane::Categories {
                                self.jump_to_category(3);
                            } else {
                                self.jump_to_stream(3);
                            }
                        }
                        KeyCode::Char('5') => {
                            if self.active_pane == Pane::Categories {
                                self.jump_to_category(4);
                            } else {
                                self.jump_to_stream(4);
                            }
                        }
                        KeyCode::Char('6') => {
                            if self.active_pane == Pane::Categories {
                                self.jump_to_category(5);
                            } else {
                                self.jump_to_stream(5);
                            }
                        }
                        KeyCode::Char('7') => {
                            if self.active_pane == Pane::Categories {
                                self.jump_to_category(6);
                            } else {
                                self.jump_to_stream(6);
                            }
                        }
                        KeyCode::Char('8') => {
                            if self.active_pane == Pane::Categories {
                                self.jump_to_category(7);
                            } else {
                                self.jump_to_stream(7);
                            }
                        }
                        KeyCode::Char('9') => {
                            if self.active_pane == Pane::Categories {
                                self.jump_to_category(8);
                            } else {
                                self.jump_to_stream(8);
                            }
                        }
                        _ => {}
                    }
                }
            }
            CurrentScreen::SeriesCategories => match key.code {
                KeyCode::Esc | KeyCode::Backspace => {
                    self.series_categories.clear();
                    self.all_series_categories.clear();
                    self.selected_series_category_index = 0;
                    self.series_category_list_state.select(None);
                    self.current_screen = CurrentScreen::Home;
                    self.search_mode = false;
                    self.search_state.query.clear();
                }
                KeyCode::Char('j') | KeyCode::Down => self.next_series_category(),
                KeyCode::Char('k') | KeyCode::Up => self.previous_series_category(),
                KeyCode::Char('g') => {
                    if !self.series_categories.is_empty() {
                        self.selected_series_category_index = self.series_categories.len() - 1;
                        self.series_category_list_state
                            .select(Some(self.series_categories.len() - 1));
                    }
                }
                KeyCode::Char('G') => {
                    if !self.series_categories.is_empty() {
                        self.selected_series_category_index = 0;
                        self.series_category_list_state.select(Some(0));
                    }
                }
                KeyCode::Char('0') => {
                    if !self.series_categories.is_empty() {
                        self.selected_series_category_index = 0;
                        self.series_category_list_state.select(Some(0));
                    }
                }
                KeyCode::Char('1') => {
                    if self.series_categories.len() > 0 {
                        self.selected_series_category_index = 0;
                        self.series_category_list_state.select(Some(0));
                    }
                }
                KeyCode::Char('2') => {
                    if self.series_categories.len() > 1 {
                        self.selected_series_category_index = 1;
                        self.series_category_list_state.select(Some(1));
                    }
                }
                KeyCode::Char('3') => {
                    if self.series_categories.len() > 2 {
                        self.selected_series_category_index = 2;
                        self.series_category_list_state.select(Some(2));
                    }
                }
                KeyCode::Char('4') => {
                    if self.series_categories.len() > 3 {
                        self.selected_series_category_index = 3;
                        self.series_category_list_state.select(Some(3));
                    }
                }
                KeyCode::Char('5') => {
                    if self.series_categories.len() > 4 {
                        self.selected_series_category_index = 4;
                        self.series_category_list_state.select(Some(4));
                    }
                }
                KeyCode::Char('6') => {
                    if self.series_categories.len() > 5 {
                        self.selected_series_category_index = 5;
                        self.series_category_list_state.select(Some(5));
                    }
                }
                KeyCode::Char('7') => {
                    if self.series_categories.len() > 6 {
                        self.selected_series_category_index = 6;
                        self.series_category_list_state.select(Some(6));
                    }
                }
                KeyCode::Char('8') => {
                    if self.series_categories.len() > 7 {
                        self.selected_series_category_index = 7;
                        self.series_category_list_state.select(Some(7));
                    }
                }
                KeyCode::Char('9') => {
                    if self.series_categories.len() > 8 {
                        self.selected_series_category_index = 8;
                        self.series_category_list_state.select(Some(8));
                    }
                }
                _ => {}
            },
            CurrentScreen::SeriesStreams => match key.code {
                KeyCode::Esc | KeyCode::Backspace | KeyCode::Left => {
                    self.series_streams.clear();
                    self.all_series_streams.clear();
                    self.selected_series_stream_index = 0;
                    self.series_stream_list_state.select(None);
                    self.current_screen = CurrentScreen::SeriesCategories;
                    self.search_mode = false;
                    self.search_state.query.clear();
                }
                KeyCode::Char('j') | KeyCode::Down => self.next_series_stream(),
                KeyCode::Char('k') | KeyCode::Up => self.previous_series_stream(),
                KeyCode::Char('g') => self.jump_to_series_bottom(),
                KeyCode::Char('G') => self.jump_to_series_top(),
                KeyCode::Char('0') => self.jump_to_series_top(),
                KeyCode::Char('1') => self.jump_to_series_stream(0),
                KeyCode::Char('2') => self.jump_to_series_stream(1),
                KeyCode::Char('3') => self.jump_to_series_stream(2),
                KeyCode::Char('4') => self.jump_to_series_stream(3),
                KeyCode::Char('5') => self.jump_to_series_stream(4),
                KeyCode::Char('6') => self.jump_to_series_stream(5),
                KeyCode::Char('7') => self.jump_to_series_stream(6),
                KeyCode::Char('8') => self.jump_to_series_stream(7),
                KeyCode::Char('9') => self.jump_to_series_stream(8),
                _ => {}
            },
            _ => {}
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flex_id::FlexId;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    fn make_key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        }
    }

    #[test]
    fn test_series_navigation_flow() {
        let mut app = App::new();

        // 1. Initial State
        assert_eq!(app.current_screen, CurrentScreen::Home);

        // 2. Simulate 'x' press on Home
        // Note: Actual async firing is mocked by just seeing if state_loading triggers
        // or if we were to act on the return value.
        // In our partial refactor, 'x' sets loading = true.
        app.handle_key_event(make_key(KeyCode::Char('x')));

        // If accounts are empty (default), nothing happens or it tries to init.
        // We need to inject a dummy account to test the "if accounts not empty" branch.
        // However, App::new() loads from config.
        // Let's assume for this test we manually inject an account.
        use crate::config::Account;
        app.config.accounts.push(Account {
            name: "Test".to_string(),
            base_url: "http://test".to_string(),
            username: "u".to_string(),
            password: "p".to_string(),
            epg_url: None,
            last_refreshed: None,
            total_channels: None,
            total_movies: None,
            total_series: None,
            server_timezone: None,
            account_type: Default::default(),
            hidden_categories: std::collections::HashSet::new(),
            category_sort_order: crate::config::CategorySortOrder::Default,
        });

        // Retry 'x'
        app.handle_key_event(make_key(KeyCode::Char('x')));
        assert!(
            app.session.state_loading,
            "State should be loading after pressing x"
        );

        // 3. Simulate Series Categories Loaded (Manual State Transition)
        app.session.state_loading = false;
        app.current_screen = CurrentScreen::SeriesCategories;
        app.series_categories = vec![
            Arc::new(Category {
                category_id: "1".into(),
                category_name: "Action".into(),
                parent_id: FlexId::Null,
                ..Default::default()
            }),
            Arc::new(Category {
                category_id: "2".into(),
                category_name: "Comedy".into(),
                parent_id: FlexId::Null,
                ..Default::default()
            }),
        ];
        // In real app, the AsyncAction handler sets this to 0
        app.series_category_list_state.select(Some(0));

        // 4. Test Navigation (j/Down)
        app.handle_key_event(make_key(KeyCode::Char('j')));
        assert_eq!(app.selected_series_category_index, 1);

        app.handle_key_event(make_key(KeyCode::Char('k')));
        assert_eq!(app.selected_series_category_index, 0);

        // 5. Test Back (Esc)
        app.handle_key_event(make_key(KeyCode::Esc));
        assert_eq!(app.current_screen, CurrentScreen::Home);
        assert_eq!(app.series_categories.len(), 0, "Should handle cleanup");
    }

    /// Regression test: Live Channels → All Channels → search "msnbc" must find results.
    /// This validates: TotalChannelsLoaded populates global_all_streams,
    /// update_search cross-pane search filters streams when on Categories pane,
    /// and the search_name field is used for matching.
    #[test]
    fn test_all_channels_msnbc_search() {
        let mut app = App::new();

        // Simulate TotalChannelsLoaded: populate global_all_streams with test data
        let msnbc = Stream {
            name: "US: MSNBC HD".to_string(),
            search_name: "us: msnbc hd".to_string(),
            stream_id: crate::flex_id::FlexId::from_number(101),
            category_id: Some("5".to_string()),
            is_american: true,
            ..Default::default()
        };
        let cnn = Stream {
            name: "US: CNN HD".to_string(),
            search_name: "us: cnn hd".to_string(),
            stream_id: crate::flex_id::FlexId::from_number(102),
            category_id: Some("5".to_string()),
            is_american: true,
            ..Default::default()
        };
        let bbc = Stream {
            name: "UK: BBC ONE HD".to_string(),
            search_name: "uk: bbc one hd".to_string(),
            stream_id: crate::flex_id::FlexId::from_number(103),
            category_id: Some("6".to_string()),
            is_american: false,
            ..Default::default()
        };

        app.global_all_streams = vec![Arc::new(msnbc), Arc::new(cnn), Arc::new(bbc)];
        app.all_streams = app.global_all_streams.clone();

        // Simulate CategoriesLoaded: populate categories
        app.all_categories = vec![
            Arc::new(Category {
                category_id: "ALL".into(),
                category_name: "All Channels".into(),
                parent_id: FlexId::Null,
                ..Default::default()
            }),
            Arc::new(Category {
                category_id: "5".into(),
                category_name: "NEWS".into(),
                parent_id: FlexId::Null,
                is_american: true,
                ..Default::default()
            }),
        ];
        app.categories = app.all_categories.clone();

        // Navigate to Categories (simulating user selecting Live Channels)
        app.current_screen = CurrentScreen::Categories;
        app.active_pane = Pane::Categories;
        app.selected_category_index = 0;
        app.category_list_state.select(Some(0));

        // Enter search mode and search for "msnbc"
        app.search_mode = true;
        app.search_state.query = "msnbc".to_string();
        app.update_search();

        // Verify: MSNBC found in cross-pane search results
        assert!(
            !app.streams.is_empty(),
            "Search for 'msnbc' should find streams in All Channels cross-pane search"
        );
        assert!(
            app.streams.iter().any(|s| s.search_name.contains("msnbc")),
            "Search results should contain MSNBC stream"
        );
        // CNN should NOT be in results (doesn't match "msnbc")
        assert!(
            !app.streams.iter().any(|s| s.search_name.contains("cnn")),
            "CNN should not appear in msnbc search results"
        );
    }

    /// Test that merica mode filters out non-American streams from search results
    #[test]
    fn test_merica_mode_filters_search() {
        let mut app = App::new();
        app.config.playlist_mode = crate::config::PlaylistMode::Merica;

        let msnbc = Stream {
            name: "US: MSNBC HD".to_string(),
            search_name: "us: msnbc hd".to_string(),
            stream_id: crate::flex_id::FlexId::from_number(201),
            is_american: true,
            ..Default::default()
        };
        let bbc_news = Stream {
            name: "UK: BBC NEWS HD".to_string(),
            search_name: "uk: bbc news hd".to_string(),
            stream_id: crate::flex_id::FlexId::from_number(202),
            is_american: false,
            ..Default::default()
        };

        app.global_all_streams = vec![Arc::new(msnbc), Arc::new(bbc_news)];
        app.all_streams = app.global_all_streams.clone();
        app.all_categories = vec![Arc::new(Category {
            category_id: "ALL".into(),
            category_name: "All Channels".into(),
            parent_id: FlexId::Null,
            ..Default::default()
        })];
        app.categories = app.all_categories.clone();

        app.current_screen = CurrentScreen::Categories;
        app.active_pane = Pane::Categories;

        // Search for "hd" — matches both streams, but merica filter should exclude BBC
        app.search_mode = true;
        app.search_state.query = "hd".to_string();
        app.update_search();

        assert!(
            app.streams.iter().any(|s| s.search_name.contains("msnbc")),
            "Merica mode should include American MSNBC in 'hd' results"
        );
        assert!(
            !app.streams.iter().any(|s| s.search_name.contains("bbc")),
            "Merica mode should exclude non-American BBC NEWS from results"
        );
    }

    #[test]
    fn test_background_load_view_consistency() {
        let mut app = App::new();

        // 1. Setup Categories
        app.categories = vec![
            Arc::new(Category {
                category_id: "ALL".into(),
                category_name: "All Channels".into(),
                ..Default::default()
            }),
            Arc::new(Category {
                category_id: "SPORTS".into(),
                category_name: "Sports".into(),
                ..Default::default()
            }),
        ];
        app.selected_category_index = 1; // User is on "Sports"
        app.current_screen = CurrentScreen::Categories;
        app.active_pane = Pane::Categories;

        // 2. Initial state: No global data, so view is empty
        assert!(app.streams.is_empty());

        // 3. Simulate specific streams for testing
        let sports_stream = Stream {
            name: "ESPN".to_string(),
            search_name: "espn".to_string(),
            stream_id: crate::flex_id::FlexId::from_number(100),
            category_id: Some("SPORTS".to_string()),
            is_american: true,
            ..Default::default()
        };
        let news_stream = Stream {
            name: "CNN".to_string(),
            search_name: "cnn".to_string(),
            stream_id: crate::flex_id::FlexId::from_number(101),
            category_id: Some("NEWS".to_string()),
            is_american: true,
            ..Default::default()
        };

        // 4. Simulate Background Scan Completion (TotalChannelsLoaded)
        app.global_all_streams = vec![Arc::new(sports_stream), Arc::new(news_stream)];
        app.build_category_indices();

        // Trigger the logic we added to AsyncAction::TotalChannelsLoaded
        app.select_category(app.selected_category_index);
        app.refresh_streams_from_cache();
        app.update_search();

        // 5. Verify View
        // Should ONLY contain Sports stream
        assert_eq!(app.streams.len(), 1, "Should filter to Sports streams only");
        assert_eq!(app.streams[0].name, "ESPN");

        // 6. Verify Search Capability
        // Search "CNN" (which is NOT in visible view, but is in global)
        app.search_mode = true;
        app.search_state.query = "CNN".to_string();
        app.update_search();

        assert_eq!(
            app.streams.len(),
            1,
            "Search should find CNN from global cache"
        );
        assert_eq!(app.streams[0].name, "CNN");
    }

    #[test]
    fn test_vod_non_all_category_queues_specific_lazy_load() {
        let mut app = App::new();

        app.current_screen = CurrentScreen::VodStreams;
        app.active_pane = Pane::Streams;
        app.vod_categories = vec![
            Arc::new(Category {
                category_id: "ALL".into(),
                category_name: "All Movies".into(),
                ..Default::default()
            }),
            Arc::new(Category {
                category_id: "ACTION".into(),
                category_name: "Action".into(),
                ..Default::default()
            }),
        ];
        app.selected_vod_category_index = 1;
        app.vod_category_list_state.select(Some(1));

        app.global_all_vod_streams = vec![Arc::new(Stream {
            name: "Global Movie".to_string(),
            search_name: "global movie".to_string(),
            stream_id: FlexId::from_number(9001),
            category_id: Some("DRAMA".to_string()),
            ..Default::default()
        })];
        app.all_vod_streams.clear();
        app.vod_streams.clear();
        app.session.state_loading = false;

        app.refresh_streams_from_cache();

        assert!(
            app.all_vod_streams.is_empty(),
            "Non-ALL category should not reuse the global ALL-movies cache"
        );
        assert!(
            app.session.state_loading,
            "Refreshing a non-cached VOD category should enter loading state"
        );
        assert!(
            app.pending_lazy_loads
                .iter()
                .any(|a| matches!(a, AsyncAction::LoadVodStreams(id) if id == "ACTION")),
            "Refreshing a non-cached VOD category should queue a category-specific lazy load"
        );
    }

    #[test]
    fn test_categories_grid_navigation() {
        let mut app = App::new();
        // Mock some categories
        for i in 0..10 {
            app.categories.push(Arc::new(Category {
                category_id: i.to_string(),
                category_name: format!("Cat {}", i),
                parent_id: FlexId::Null,
                ..Default::default()
            }));
        }

        // Emulate 4-column responsive grid
        app.grid_cols = 4;

        // Horizontal testing
        app.selected_category_index = 0; // Col 0 Row 0
        app.move_category_x(true); // Right -> Col 1 Row 0 (index 3)
        assert_eq!(app.selected_category_index, 3);

        app.move_category_x(false); // Left -> Col 0 Row 0 (index 0)
        assert_eq!(app.selected_category_index, 0);

        app.move_category_x(false); // Left bounded -> 0
        assert_eq!(app.selected_category_index, 0);

        // Vertical testing (Up/Down)
        app.move_category_y(true); // Down -> Col 0 Row 1 (index 1)
        assert_eq!(app.selected_category_index, 1);

        app.move_category_y(true); // Down -> Col 0 Row 2 (index 2)
        assert_eq!(app.selected_category_index, 2);

        app.move_category_y(true); // Down bounded -> Col 0 Row 2 (index 2)
        assert_eq!(app.selected_category_index, 2);

        app.move_category_y(false); // Up -> Col 0 Row 1 (index 1)
        assert_eq!(app.selected_category_index, 1);

        // Horizontal from row 2
        app.selected_category_index = 2; // Col 0 Row 2
        app.move_category_x(true); // Right to Col 1 -> Col 1 Row 2 (index 5)
        assert_eq!(app.selected_category_index, 5);

        app.move_category_x(true); // Right to Col 2 -> Col 2 only has 2 rows!
                                   // So it snaps to Col 2 Row 1 (index 7)
        assert_eq!(app.selected_category_index, 7);
    }

    #[test]
    fn test_page_navigation_live_streams() {
        let mut app = App::new();
        app.area_streams = Rect::new(0, 0, 50, 22);

        for i in 0..100 {
            app.streams.push(Arc::new(Stream {
                name: format!("Stream {}", i),
                search_name: format!("stream {}", i),
                stream_id: FlexId::from_number(i as i64),
                ..Default::default()
            }));
        }
        app.selected_stream_index = 0;
        app.stream_list_state.select(Some(0));

        let page = app.page_size_for_pane(Pane::Streams);
        assert_eq!(page, 20, "Page size should be height - 2");

        app.page_down_stream();
        assert_eq!(
            app.selected_stream_index, page,
            "PageDown should jump by page size"
        );

        app.page_up_stream();
        assert_eq!(
            app.selected_stream_index, 0,
            "PageUp should jump back to top"
        );

        app.half_page_down_stream();
        assert_eq!(
            app.selected_stream_index,
            page / 2,
            "Ctrl+D should jump by half page"
        );

        app.half_page_up_stream();
        assert_eq!(
            app.selected_stream_index, 0,
            "Ctrl+U should jump back to top"
        );

        app.jump_to_bottom();
        assert_eq!(
            app.selected_stream_index, 99,
            "End should jump to last item"
        );

        app.jump_to_top();
        assert_eq!(
            app.selected_stream_index, 0,
            "Home should jump to first item"
        );

        app.page_down_stream();
        app.page_down_stream();
        app.page_down_stream();
        app.page_down_stream();
        app.page_down_stream();
        assert_eq!(
            app.selected_stream_index, 99,
            "PageDown should clamp at last item, not wrap"
        );

        app.page_up_stream();
        assert_eq!(
            app.selected_stream_index,
            99 - page,
            "PageUp from bottom should go back by page"
        );
    }

    #[test]
    fn test_page_navigation_vod_streams() {
        let mut app = App::new();
        app.area_streams = Rect::new(0, 0, 50, 22);

        for i in 0..50 {
            app.vod_streams.push(Arc::new(Stream {
                name: format!("Movie {}", i),
                search_name: format!("movie {}", i),
                stream_id: FlexId::from_number(i as i64),
                stream_type: "movie".to_string(),
                ..Default::default()
            }));
        }
        app.selected_vod_stream_index = 0;
        app.vod_stream_list_state.select(Some(0));

        app.page_down_vod_stream();
        let page = app.page_size_for_pane(Pane::Streams);
        assert_eq!(app.selected_vod_stream_index, page);

        app.jump_to_vod_bottom();
        assert_eq!(app.selected_vod_stream_index, 49);

        app.jump_to_vod_top();
        assert_eq!(app.selected_vod_stream_index, 0);

        app.half_page_down_vod_stream();
        assert_eq!(app.selected_vod_stream_index, page / 2);
    }

    #[test]
    fn test_page_navigation_series_streams() {
        let mut app = App::new();
        app.area_streams = Rect::new(0, 0, 50, 22);

        for i in 0..50 {
            app.series_streams.push(Arc::new(Stream {
                name: format!("Series {}", i),
                search_name: format!("series {}", i),
                stream_id: FlexId::from_number(i as i64),
                stream_type: "series".to_string(),
                ..Default::default()
            }));
        }
        app.selected_series_stream_index = 0;
        app.series_stream_list_state.select(Some(0));

        app.page_down_series_stream();
        let page = app.page_size_for_pane(Pane::Streams);
        assert_eq!(app.selected_series_stream_index, page);

        app.jump_to_series_bottom();
        assert_eq!(app.selected_series_stream_index, 49);

        app.jump_to_series_top();
        assert_eq!(app.selected_series_stream_index, 0);
    }

    #[test]
    fn test_page_navigation_series_episodes() {
        let mut app = App::new();
        app.area_episodes = Rect::new(0, 0, 30, 22);

        for i in 0..30 {
            app.series_episodes.push(crate::api::SeriesEpisode {
                id: Some(FlexId::from_number(i as i64)),
                episode_num: i,
                title: Some(format!("Episode {}", i)),
                container_extension: Some("mp4".to_string()),
                info: None,
                season: 1,
                direct_source: String::new(),
            });
        }
        app.selected_series_episode_index = 0;
        app.series_episode_list_state.select(Some(0));

        let page = app.page_size_for_pane(Pane::Episodes);
        assert_eq!(page, 20);

        app.page_down_series_episode();
        assert_eq!(app.selected_series_episode_index, page);

        app.page_up_series_episode();
        assert_eq!(app.selected_series_episode_index, 0);
    }

    #[test]
    fn test_page_navigation_categories() {
        let mut app = App::new();
        app.area_categories = Rect::new(0, 0, 30, 22);
        app.grid_cols = 1;

        for i in 0..40 {
            app.categories.push(Arc::new(Category {
                category_id: i.to_string(),
                category_name: format!("Category {}", i),
                parent_id: FlexId::Null,
                ..Default::default()
            }));
        }
        app.selected_category_index = 0;
        app.category_list_state.select(Some(0));

        app.page_down_category();
        let page = app.page_size_for_pane(Pane::Categories);
        assert_eq!(app.selected_category_index, page);

        app.page_up_category();
        assert_eq!(app.selected_category_index, 0);

        app.jump_to_category_bottom();
        assert_eq!(app.selected_category_index, 39);

        app.jump_to_category_top();
        assert_eq!(app.selected_category_index, 0);

        app.half_page_down_category();
        assert_eq!(app.selected_category_index, page / 2);
    }

    #[test]
    fn test_page_navigation_global_search() {
        let mut app = App::new();
        app.area_streams = Rect::new(0, 0, 50, 22);

        for i in 0..80 {
            app.global_search_results.push(Arc::new(Stream {
                name: format!("Result {}", i),
                search_name: format!("result {}", i),
                stream_id: FlexId::from_number(i as i64),
                ..Default::default()
            }));
        }
        app.selected_stream_index = 0;
        app.global_search_list_state.select(Some(0));

        app.page_down_global_search();
        let page = app.page_size_for_pane(Pane::Streams);
        assert_eq!(app.selected_stream_index, page);

        app.jump_to_global_search_bottom();
        assert_eq!(app.selected_stream_index, 79);

        app.jump_to_global_search_top();
        assert_eq!(app.selected_stream_index, 0);
    }

    #[test]
    fn test_page_size_dynamic_viewport() {
        let mut app = App::new();

        app.area_categories = Rect::new(0, 0, 30, 10);
        app.area_streams = Rect::new(0, 0, 50, 22);
        app.area_episodes = Rect::new(0, 0, 30, 5);

        assert_eq!(app.page_size_for_pane(Pane::Categories), 8);
        assert_eq!(app.page_size_for_pane(Pane::Streams), 20);
        assert_eq!(app.page_size_for_pane(Pane::Episodes), 3);
    }

    #[test]
    fn test_jump_list_clamps_at_boundaries() {
        let mut app = App::new();

        app.streams.push(Arc::new(Stream {
            name: "Only".to_string(),
            search_name: "only".to_string(),
            stream_id: FlexId::from_number(1),
            ..Default::default()
        }));
        app.selected_stream_index = 0;
        app.stream_list_state.select(Some(0));

        app.page_down_stream();
        assert_eq!(
            app.selected_stream_index, 0,
            "PageDown on single item should stay at 0"
        );

        app.page_up_stream();
        assert_eq!(
            app.selected_stream_index, 0,
            "PageUp on single item should stay at 0"
        );

        app.jump_to_bottom();
        assert_eq!(app.selected_stream_index, 0);

        app.jump_to_top();
        assert_eq!(app.selected_stream_index, 0);
    }

    #[test]
    fn test_jump_list_empty_lists() {
        let mut app = App::new();
        app.area_streams = Rect::new(0, 0, 50, 22);

        app.page_down_stream();
        assert_eq!(app.selected_stream_index, 0);

        app.page_up_stream();
        assert_eq!(app.selected_stream_index, 0);

        app.jump_to_bottom();
        assert_eq!(app.selected_stream_index, 0);

        app.jump_to_top();
        assert_eq!(app.selected_stream_index, 0);
    }

    #[test]
    fn test_category_page_navigation_multi_screen() {
        let mut app = App::new();
        app.area_categories = Rect::new(0, 0, 30, 22);
        app.grid_cols = 1;

        for i in 0..30 {
            app.vod_categories.push(Arc::new(Category {
                category_id: i.to_string(),
                category_name: format!("VOD Cat {}", i),
                parent_id: FlexId::Null,
                ..Default::default()
            }));
            app.series_categories.push(Arc::new(Category {
                category_id: i.to_string(),
                category_name: format!("Series Cat {}", i),
                parent_id: FlexId::Null,
                ..Default::default()
            }));
        }

        app.current_screen = CurrentScreen::VodCategories;
        app.selected_vod_category_index = 0;
        app.vod_category_list_state.select(Some(0));

        app.page_down_category();
        let page = app.page_size_for_pane(Pane::Categories);
        assert_eq!(app.selected_vod_category_index, page);

        app.jump_to_category_bottom();
        assert_eq!(app.selected_vod_category_index, 29);

        app.jump_to_category_top();
        assert_eq!(app.selected_vod_category_index, 0);

        app.current_screen = CurrentScreen::SeriesCategories;
        app.selected_series_category_index = 0;
        app.series_category_list_state.select(Some(0));

        app.page_down_category();
        assert_eq!(app.selected_series_category_index, page);

        app.jump_to_category_bottom();
        assert_eq!(app.selected_series_category_index, 29);

        app.jump_to_category_top();
        assert_eq!(app.selected_series_category_index, 0);
    }
}
