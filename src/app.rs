use crate::api::{Category, ServerInfo, Stream, UserInfo, IptvClient};
use crate::flex_id::FlexId;
use crate::config::AppConfig;
use crate::errors::{SearchState, LoadingProgress};
use crate::state::{
    SessionState, ContentState, SeriesState, VodState, LoginFormState,
    UiState, SportsState, MatrixRainState, SearchState as DecomposedSearchState,
    GroupManagementState,
};
use std::sync::Arc;
use std::collections::VecDeque;
use rayon::prelude::*;
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
    TotalMoviesLoaded(Vec<Stream>),
    TotalSeriesLoaded(Vec<Stream>),
    PlaylistRefreshed(IptvClient, Option<UserInfo>, Option<ServerInfo>),
    EpgLoaded(String, String), // stream_id, program_title
    StreamHealthLoaded(String, u64), // stream_id, latency_ms
    UpdateAvailable(String), // new_version
    NoUpdateFound,
    SportsMatchesLoaded(Vec<crate::sports::StreamedMatch>),
    SportsStreamsLoaded(Vec<crate::sports::StreamedStream>),
    ScoresLoaded(Vec<crate::scores::ScoreGame>),
    ScanProgress { current: usize, total: usize, eta_secs: u64 },
    // Chromecast casting
    CastDevicesDiscovered(Vec<CastDevice>),
    CastStarted(String), // Device name
    CastFailed(String),  // Error message
    Error(String),
}

#[derive(PartialEq, Debug, Clone)]
pub enum CurrentScreen {
    Home,             // List of accounts
    Login,            // Add new account form
    Categories,       // List of channel categories
    Streams,          // List of streams in a category
    VodCategories,    // List of VOD categories
    VodStreams,       // List of VODs in a category
    SeriesCategories, // List of Series categories
    SeriesStreams,    // List of Series (shows as streams/list)
    Settings,         // App settings
    TimezoneSettings, // Edit Timezone
    Play,             // (Optional) Info screen before playing
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
    pub state_loading: bool,
    pub cached_user_timezone: String,

    // Home / Accounts
    pub selected_account_index: usize,
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
    pub loading_tick: u64,

    // Active Session
    pub current_client: Option<IptvClient>,

    // Categories
    pub all_categories: Vec<Arc<Category>>,
    pub categories: Vec<Arc<Category>>,
    pub selected_category_index: usize,
    pub category_list_state: ListState,

    // Streams
    pub all_streams: Vec<Arc<Stream>>,
    pub streams: Vec<Arc<Stream>>,
    pub selected_stream_index: usize,
    pub stream_list_state: ListState,

    // VOD Categories
    pub all_vod_categories: Vec<Arc<Category>>,
    pub vod_categories: Vec<Arc<Category>>,
    pub selected_vod_category_index: usize,
    pub vod_category_list_state: ListState,

    // VOD Streams
    pub all_vod_streams: Vec<Arc<Stream>>,
    pub vod_streams: Vec<Arc<Stream>>,
    pub selected_vod_stream_index: usize,
    pub vod_stream_list_state: ListState,

    // Series Data
    pub all_series_categories: Vec<Arc<Category>>,
    pub series_categories: Vec<Arc<Category>>,
    pub selected_series_category_index: usize,
    pub series_category_list_state: ListState,

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
    pub global_all_vod_streams: Vec<Arc<Stream>>,
    pub global_all_series_streams: Vec<Arc<Stream>>,

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
    pub last_search_query: String, // Track last query for incremental narrowing
    
    // Loading progress
    pub loading_progress: Option<LoadingProgress>,

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
    pub provider_timezone: Option<String>,
    pub loading_message: Option<String>,
    pub player_error: Option<String>,
    pub loading_log: std::collections::VecDeque<String>,

    // Account details
    pub account_info: Option<UserInfo>,
    pub server_info: Option<ServerInfo>,
    pub total_channels: usize,
    pub total_movies: usize,
    pub total_series: usize,

    // Layout tracking for mouse support
    pub area_categories: Rect,
    pub area_streams: Rect,
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

    // EPG Enrichment
    pub epg_cache: std::collections::HashMap<String, String>,
    pub last_focused_stream_id: Option<String>,
    #[cfg(not(target_arch = "wasm32"))]
    pub focus_timestamp: Option<std::time::Instant>,
    #[cfg(target_arch = "wasm32")]
    pub focus_timestamp: Option<f64>,

    // Detail panel lazy rendering
    #[cfg(not(target_arch = "wasm32"))]
    pub detail_last_index: usize,
    #[cfg(not(target_arch = "wasm32"))]
    pub detail_settle_time: Option<std::time::Instant>,

    // Global Search
    pub global_search_results: Vec<Arc<Stream>>,
    pub global_search_list_state: ListState,

    // Group Management
    pub selected_group_index: usize,
    pub group_list_state: ListState,
    pub pending_stream_for_group: Option<String>,  // Stream ID waiting to be added to a group
    pub group_name_input: String,  // For creating/renaming groups
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
    pub scan_start_time: Option<std::time::Instant>,   // For loading ETA
    #[cfg(not(target_arch = "wasm32"))]
    pub last_search_update: Option<std::time::Instant>, // 150ms debounce gate
    pub category_channel_counts: std::collections::HashMap<String, usize>, // Counts per category_id

    // Cache state
    pub background_refresh_active: bool,  // True when background refresh is in progress
    pub cache_loaded: bool,               // True if current session loaded from cache

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
    /// Search state
    pub search: DecomposedSearchState,
    /// Group management state
    pub groups: GroupManagementState,
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

        let mut app = App {
            cached_user_timezone: config.get_user_timezone(),
            config,
            current_screen: CurrentScreen::Home,
            input_mode: InputMode::Normal,
            should_quit: false,
            state_loading: false,

            epg_cache: std::collections::HashMap::new(),
            last_focused_stream_id: None,
            focus_timestamp: None,
            #[cfg(not(target_arch = "wasm32"))]
            detail_last_index: usize::MAX,
            #[cfg(not(target_arch = "wasm32"))]
            detail_settle_time: None,

            editing_account_index: None,

            selected_account_index: 0,
            account_list_state,

            login_field_focus: LoginField::Name,
            input_name: Input::default(),
            input_url: Input::default(),
            input_username: Input::default(),
            input_password: Input::default(),
            input_epg_url: Input::default(),
            input_server_timezone: Input::default(),
            login_error: None,
            loading_tick: 0,

            current_client: None,
            all_categories: vec![],
            categories: vec![],
            selected_category_index: 0,
            category_list_state: ListState::default(),

            all_streams: vec![],
            streams: vec![],
            selected_stream_index: 0,
            stream_list_state: ListState::default(),

            all_vod_categories: vec![],
            vod_categories: vec![],
            selected_vod_category_index: 0,
            vod_category_list_state: ListState::default(),

            all_vod_streams: vec![],
            vod_streams: vec![],
            selected_vod_stream_index: 0,
            vod_stream_list_state: ListState::default(),

            // Series Init
            all_series_categories: vec![],
            series_categories: vec![],
            selected_series_category_index: 0,
            series_category_list_state: ListState::default(),
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
            global_all_vod_streams: vec![],
            global_all_series_streams: vec![],

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
            category_grid_view: true,
            grid_cols: 3, // default, overwritten by renderer
            search_state: SearchState::new(),
            search_mode: false,
            last_search_query: String::new(),
            show_help: false,
            loading_progress: None,
            show_guide: None,
            guide_scroll: 0,
            settings_state: SettingsState::Main,
            previous_screen: None,
            show_save_confirmation: false,
            about_text,
            about_scroll: 0,
            provider_timezone: None,
            loading_message: None,
            player_error: None,
            account_info: None,
            server_info: None,
            total_channels: 0,
            total_movies: 0,
            total_series: 0,
            area_categories: Rect::default(),
            area_streams: Rect::default(),
            area_accounts: Rect::default(),
            
            // Matrix rain: Always show on startup for 3 seconds
            show_matrix_rain: true,
            matrix_rain_start_time: matrix_rain_start,
            matrix_rain_screensaver_mode: false, // Startup mode (with logo)
            show_welcome_popup: false,
            matrix_rain_columns: vec![],
            matrix_rain_logo_hits: vec![false; 101 * 6], // 101 wide x 6 high logo
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

            // Cache state
            background_refresh_active: false,
            cache_loaded: false,

            // --- Decomposed State Structs ---
            session: SessionState::new(),
            live: ContentState::new(),
            vod: VodState::new(),
            series: SeriesState::new(),
            login_form: LoginFormState::new(),
            ui: UiState::new(),
            sports: SportsState::new(),
            matrix_rain: MatrixRainState::new(),
            search: DecomposedSearchState::new(),
            groups: GroupManagementState::new(),
            loading_log: VecDeque::with_capacity(30),
        };

        app.refresh_settings_options();
        app
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
                
                let t1_in_game = g_home.contains(&t1) || g_away.contains(&t1) || t1.contains(&g_home) || t1.contains(&g_away);
                let t2_in_game = g_home.contains(&t2) || g_away.contains(&t2) || t2.contains(&g_home) || t2.contains(&g_away);
                
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
            let home_short = game.home_team.split_whitespace().last().unwrap_or("").to_lowercase();
            let away_short = game.away_team.split_whitespace().last().unwrap_or("").to_lowercase();
            
            // Check if stream contains either team's short name (at least 4 chars to avoid false positives)
            (home_short.len() >= 4 && stream_lower.contains(&home_short)) ||
            (away_short.len() >= 4 && stream_lower.contains(&away_short))
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
                    self.config.processing_modes.iter()
                        .map(|m| match m {
                            crate::config::ProcessingMode::Merica => "'merica",
                            crate::config::ProcessingMode::Sports => "Sports",
                            crate::config::ProcessingMode::AllEnglish => "All English",
                        })
                        .collect::<Vec<_>>()
                        .join(" + ")
                }
            ),
            format!(
                "DNS Provider: {}",
                self.config.dns_provider.display_name()
            ),
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
                "Smooth Motion (VLC): {}",
                if self.config.smooth_motion { "ON" } else { "OFF" }
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
            "VLC ONLY: Enables Bob-interpolation to double the perceived frame-rate of live TV.".to_string(),
            "How often to automatically refresh playlist data when logging in. Set to 0 to disable.".to_string(),
            "Launch the iconic Matrix digital rain animation.".to_string(),
            "Check if a newer version of Matrix IPTV is available for download.".to_string(),
            "View application info, version, and credits.".to_string(),
        ];

        if self.settings_list_state.selected().is_none() {
            self.settings_list_state.select(Some(0));
        }
    }

    pub fn get_selected_account(&self) -> Option<&crate::config::Account> {
        self.config.accounts.get(self.selected_account_index)
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

    pub fn next_account(&mut self) {
        Self::navigate_list(
            self.config.accounts.len(),
            &mut self.selected_account_index,
            &mut self.account_list_state,
            true,
        );
    }

    pub fn previous_account(&mut self) {
        Self::navigate_list(
            self.config.accounts.len(),
            &mut self.selected_account_index,
            &mut self.account_list_state,
            false,
        );
    }

    pub fn next_category(&mut self) {
        let len = self.categories.len();
        if len == 0 { return; }
        let next = (self.selected_category_index + 1) % len;
        self.select_category(next);
    }

    pub fn previous_category(&mut self) {
        let len = self.categories.len();
        if len == 0 { return; }
        let prev = if self.selected_category_index == 0 { len - 1 } else { self.selected_category_index - 1 };
        self.select_category(prev);
    }

    pub fn jump_to_category(&mut self, index: usize) {
        if index < self.categories.len() {
            self.select_category(index);
        }
    }

    pub fn jump_to_category_bottom(&mut self) {
        if !self.categories.is_empty() {
            self.select_category(self.categories.len() - 1);
        }
    }

    pub fn jump_to_category_top(&mut self) {
        if !self.categories.is_empty() {
            self.select_category(0);
        }
    }

    /// Primary navigation logic for Categories
    /// Updates selection AND filters the streams pane from global cache immediately (Auto-Load)
    pub fn select_category(&mut self, index: usize) {
        self.selected_category_index = index;
        self.category_list_state.select(Some(index));

        // Fix #1: Auto-populate streams pane from global cache if available
        if !self.global_all_streams.is_empty() {
            let cat_id = self.categories[index].category_id.clone();
            
            // 1. Filter source
            if cat_id == "ALL" {
                self.all_streams = self.global_all_streams.clone();
            } else {
                // Determine filtered set
                // Cloning Arcs is cheap (pointer copy)
                self.all_streams = self.global_all_streams.iter()
                    .filter(|s| s.category_id.as_deref() == Some(&cat_id))
                    .cloned()
                    .collect();
            }

            // 2. Apply display filters (American Mode, Search Query if any) via update_search
            // This ensures self.streams (the visible list) matches self.all_streams (the category context)
            self.update_search();
        } else {
            // If we don't have global data yet, visible streams should be cleared 
            // so we don't show "Sports" channels while "Movies" is selected
            self.streams.clear();
            self.all_streams.clear();
        }
    }

    pub fn next_stream(&mut self) {
        Self::navigate_list(
            self.streams.len(),
            &mut self.selected_stream_index,
            &mut self.stream_list_state,
            true,
        );
    }

    pub fn jump_to_stream(&mut self, index: usize) {
        if index < self.streams.len() {
            self.selected_stream_index = index;
            self.stream_list_state.select(Some(index));
        }
    }

    pub fn jump_to_bottom(&mut self) {
        if !self.streams.is_empty() {
            self.selected_stream_index = self.streams.len() - 1;
            self.stream_list_state.select(Some(self.streams.len() - 1));
        }
    }

    pub fn jump_to_top(&mut self) {
        if !self.streams.is_empty() {
            self.selected_stream_index = 0;
            self.stream_list_state.select(Some(0));
        }
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
        Self::navigate_list(
            self.vod_categories.len(),
            &mut self.selected_vod_category_index,
            &mut self.vod_category_list_state,
            true,
        );
    }

    pub fn previous_vod_category(&mut self) {
        Self::navigate_list(
            self.vod_categories.len(),
            &mut self.selected_vod_category_index,
            &mut self.vod_category_list_state,
            false,
        );
    }

    pub fn next_vod_stream(&mut self) {
        Self::navigate_list(
            self.vod_streams.len(),
            &mut self.selected_vod_stream_index,
            &mut self.vod_stream_list_state,
            true,
        );
    }

    pub fn jump_to_vod_stream(&mut self, index: usize) {
        if index < self.vod_streams.len() {
            self.selected_vod_stream_index = index;
            self.vod_stream_list_state.select(Some(index));
        }
    }

    pub fn jump_to_vod_bottom(&mut self) {
        if !self.vod_streams.is_empty() {
            self.selected_vod_stream_index = self.vod_streams.len() - 1;
            self.vod_stream_list_state.select(Some(self.vod_streams.len() - 1));
        }
    }

    pub fn jump_to_vod_top(&mut self) {
        if !self.vod_streams.is_empty() {
            self.selected_vod_stream_index = 0;
            self.vod_stream_list_state.select(Some(0));
        }
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

    pub fn jump_to_global_search_result(&mut self, index: usize) {
        if index < self.global_search_results.len() {
            self.selected_stream_index = index;
            self.global_search_list_state.select(Some(index));
        }
    }

    pub fn jump_to_global_search_bottom(&mut self) {
        if !self.global_search_results.is_empty() {
            self.selected_stream_index = self.global_search_results.len() - 1;
            self.global_search_list_state.select(Some(self.global_search_results.len() - 1));
        }
    }

    pub fn jump_to_global_search_top(&mut self) {
        if !self.global_search_results.is_empty() {
            self.selected_stream_index = 0;
            self.global_search_list_state.select(Some(0));
        }
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
    pub fn pre_cache_parsed(streams: &mut [Arc<Stream>], provider_tz: Option<&str>) {
        for s in streams.iter_mut() {
            let inner = Arc::make_mut(s);
            if inner.cached_parsed.is_none() {
                inner.cached_parsed = Some(Box::new(crate::parser::parse_stream(&inner.name, provider_tz)));
            }
        }
    }

    /// Build a map of category_id → channel count from global_all_streams.
    /// Called once after TotalChannelsLoaded to display counts next to category names.
    pub fn build_category_counts(&mut self) {
        self.category_channel_counts.clear();
        for s in &self.global_all_streams {
            if let Some(ref cid) = s.category_id {
                *self.category_channel_counts.entry(cid.clone()).or_insert(0) += 1;
            }
        }
    }

    /// Record a recently watched channel. Deduplicates by stream_id, caps at 20.
    pub fn record_recently_watched(&mut self, stream_id: String, stream_name: String) {
        // Remove existing entry for this stream_id (dedup)
        self.config.recently_watched.retain(|(id, _)| id != &stream_id);
        // Push to front
        self.config.recently_watched.insert(0, (stream_id, stream_name));
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
        
        // Phase 3: Incremental Search Narrowing
        // Only re-filter if the query has actually changed
        // Note: Empty query always passes through to allow view resets
        if !query.is_empty() && query == self.last_search_query {
            return; // No change, skip re-filtering
        }
        
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
                match self.active_pane {
                    Pane::Categories => {
                        self.categories = self.all_categories.par_iter()
                            .filter(|c| {
                                c.search_name.contains(&query) && (!is_merica || c.is_american)
                            })
                            .cloned()
                            .collect();
                        self.selected_category_index = 0;
                        if !self.categories.is_empty() {
                            self.category_list_state.select(Some(0));
                        } else {
                            self.category_list_state.select(None);
                        }

                        // Cross-pane search: also search streams so users can find channels
                        // directly from the categories view (e.g. searching "msnbc" in All Channels)
                        if !query.is_empty() && !self.global_all_streams.is_empty() {
                            let mut stream_results: Vec<Arc<Stream>> = self.global_all_streams.par_iter()
                                .filter(|s| {
                                    if is_merica && !s.is_american { return false; }
                                    if s.search_name.contains(&query) { return true; }
                                    query.len() >= 3 && s.fuzzy_match(&query, 60)
                                })
                                .cloned()
                                .collect();
                            stream_results.sort_by_cached_key(|s| !s.search_name.contains(&query));
                            self.streams = stream_results.into_iter().take(1000).collect();
                        } else if query.is_empty() && !self.all_streams.is_empty() {
                            // Restore from loaded category streams when search is cleared
                            self.streams = self.all_streams.iter()
                                .filter(|s| !is_merica || s.is_american)
                                .take(1000)
                                .cloned()
                                .collect();
                        }
                        // pre_cache_parsed removed from search hot path — renderer falls back to
                        // parse_stream when cached_parsed is None, avoiding 1000 regex ops per keystroke
                        self.selected_stream_index = 0;
                        if !self.streams.is_empty() {
                            self.stream_list_state.select(Some(0));
                        } else {
                            self.stream_list_state.select(None);
                        }
                    }
                    Pane::Streams => {
                        if query.is_empty() {
                            self.streams = self.all_streams.iter()
                                .filter(|s| !is_merica || s.is_american)
                                .take(1000)
                                .cloned()
                                .collect();
                        } else {
                            // Multi-pass parallel search prioritization
                            let mut results: Vec<Arc<Stream>> = self.all_streams.par_iter()
                                .filter(|s| {
                                    if is_merica && !s.is_american { return false; }
                                    
                                    // Layer 1: Substring match (Fast)
                                    if s.search_name.contains(&query) { return true; }
                                    
                                    // Layer 2: Fuzzy match (only for 3+ char queries to avoid lag)
                                    query.len() >= 3 && s.fuzzy_match(&query, 60)
                                })
                                .cloned()
                                .collect();
                            
                            // Result classification (Sort exact matches higher)
                            results.sort_by_cached_key(|s| !s.search_name.contains(&query));
                            
                            self.streams = results.into_iter().take(1000).collect();
                        }

                        App::pre_cache_parsed(&mut self.streams, self.provider_timezone.as_deref());
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
            CurrentScreen::VodCategories | CurrentScreen::VodStreams => {
                match self.active_pane {
                    Pane::Categories => {
                        self.vod_categories = self.all_vod_categories.par_iter()
                            .filter(|c| {
                                c.search_name.contains(&query) && (!is_merica || c.is_english)
                            })
                            .cloned()
                            .collect();
                        self.selected_vod_category_index = 0;
                        if !self.vod_categories.is_empty() {
                            self.vod_category_list_state.select(Some(0));
                        } else {
                            self.vod_category_list_state.select(None);
                        }
                    }
                    Pane::Streams => {
                        if query.is_empty() {
                            self.vod_streams = self.all_vod_streams.iter()
                                .filter(|s| !is_merica || s.is_english)
                                .take(1000)
                                .cloned()
                                .collect();
                        } else {
                            let mut results: Vec<Arc<Stream>> = self.all_vod_streams.par_iter()
                                .filter(|s| {
                                    if is_merica && !s.is_english { return false; }
                                    if s.search_name.contains(&query) { return true; }
                                    query.len() >= 3 && s.fuzzy_match(&query, 60)
                                })
                                .cloned()
                                .collect();
                            
                            results.sort_by_cached_key(|s| !s.search_name.contains(&query));
                            self.vod_streams = results.into_iter().take(1000).collect();
                        }

                        App::pre_cache_parsed(&mut self.vod_streams, self.provider_timezone.as_deref());
                        self.selected_vod_stream_index = 0;
                        if !self.vod_streams.is_empty() {
                            self.vod_stream_list_state.select(Some(0));
                        } else {
                            self.vod_stream_list_state.select(None);
                        }
                    }
                    Pane::Episodes => {}
                }
            }
            CurrentScreen::SeriesCategories | CurrentScreen::SeriesStreams => {
                match self.active_pane {
                    Pane::Categories => {
                        self.series_categories = self.all_series_categories.par_iter()
                            .filter(|c| {
                                c.search_name.contains(&query) && (!is_merica || c.is_english)
                            })
                            .cloned()
                            .collect();
                        self.selected_series_category_index = 0;
                        if !self.series_categories.is_empty() {
                            self.series_category_list_state.select(Some(0));
                        } else {
                            self.series_category_list_state.select(None);
                        }
                    }
                    Pane::Streams => {
                        if query.is_empty() {
                            self.series_streams = self.all_series_streams.iter()
                                .filter(|s| !is_merica || s.is_english)
                                .take(1000)
                                .cloned()
                                .collect();
                        } else {
                            let mut results: Vec<Arc<Stream>> = self.all_series_streams.par_iter()
                                .filter(|s| {
                                    if is_merica && !s.is_english { return false; }
                                    if s.search_name.contains(&query) { return true; }
                                    query.len() >= 3 && s.fuzzy_match(&query, 60)
                                })
                                .cloned()
                                .collect();

                            results.sort_by_cached_key(|s| !s.search_name.contains(&query));
                            self.series_streams = results.into_iter().take(1000).collect();
                        }


                        App::pre_cache_parsed(&mut self.series_streams, self.provider_timezone.as_deref());
                        self.selected_series_stream_index = 0;
                        if !self.series_streams.is_empty() {
                            self.series_stream_list_state.select(Some(0));
                        } else {
                            self.series_stream_list_state.select(None);
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
                    let mut hits: Vec<Arc<Stream>> = self.global_all_streams.par_iter()
                        .filter(|s| s.search_name.contains(&query) || (use_fuzzy && s.fuzzy_match(&query, 70)))
                        .chain(self.global_all_vod_streams.par_iter()
                            .filter(|s| s.search_name.contains(&query) || (use_fuzzy && s.fuzzy_match(&query, 70))))
                        .chain(self.global_all_series_streams.par_iter()
                            .filter(|s| s.search_name.contains(&query) || (use_fuzzy && s.fuzzy_match(&query, 70))))
                        .cloned()
                        .collect();

                    // Prioritize exact substring matches
                    hits.sort_by_cached_key(|s| !s.search_name.contains(&query));
                    hits.into_iter().take(100).collect()
                };


                self.global_search_results = results;
                App::pre_cache_parsed(&mut self.global_search_results, self.provider_timezone.as_deref());
                self.selected_stream_index = 0;
                if !self.global_search_results.is_empty() {
                    self.global_search_list_state.select(Some(0));
                } else {
                    self.global_search_list_state.select(None);
                }
            }
            _ => {}
        }
    }

    // Series Navigation Helpers
    pub fn next_series_category(&mut self) {
        Self::navigate_list(
            self.series_categories.len(),
            &mut self.selected_series_category_index,
            &mut self.series_category_list_state,
            true,
        );
    }

    pub fn previous_series_category(&mut self) {
        Self::navigate_list(
            self.series_categories.len(),
            &mut self.selected_series_category_index,
            &mut self.series_category_list_state,
            false,
        );
    }

    pub fn next_series_stream(&mut self) {
        Self::navigate_list(
            self.series_streams.len(),
            &mut self.selected_series_stream_index,
            &mut self.series_stream_list_state,
            true,
        );
    }

    pub fn jump_to_series_stream(&mut self, index: usize) {
        if index < self.series_streams.len() {
            self.selected_series_stream_index = index;
            self.series_stream_list_state.select(Some(index));
        }
    }

    pub fn jump_to_series_bottom(&mut self) {
        if !self.series_streams.is_empty() {
            self.selected_series_stream_index = self.series_streams.len() - 1;
            self.series_stream_list_state.select(Some(self.series_streams.len() - 1));
        }
    }

    pub fn jump_to_series_top(&mut self) {
        if !self.series_streams.is_empty() {
            self.selected_series_stream_index = 0;
            self.series_stream_list_state.select(Some(0));
        }
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
                base_url: final_url,
                username: user,
                password: pass,
                account_type: crate::config::AccountType::Xtream,
                epg_url: epg_opt,
                last_refreshed: None,
                total_channels: None,
                total_movies: None,
                total_series: None,
                server_timezone: tz_opt,
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
                            self.state_loading = true;
                            // For testing: we can assert that state_loading became true.
                        }
                    }
                    // ... other Home keys
                    _ => {}
                }
            }
            CurrentScreen::ContentTypeSelection => {
                match key.code {
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
                }
            }
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
                                self.next_category();
                            } else {
                                self.next_stream();
                            }
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            if self.active_pane == Pane::Categories {
                                self.previous_category();
                            } else {
                                self.previous_stream();
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
                        self.series_category_list_state.select(Some(self.series_categories.len() - 1));
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
        });

        // Retry 'x'
        app.handle_key_event(make_key(KeyCode::Char('x')));
        assert!(
            app.state_loading,
            "State should be loading after pressing x"
        );

        // 3. Simulate Series Categories Loaded (Manual State Transition)
        app.state_loading = false;
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

        app.global_all_streams = vec![
            Arc::new(msnbc),
            Arc::new(cnn),
            Arc::new(bbc),
        ];
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
            Arc::new(Category { category_id: "ALL".into(), category_name: "All Channels".into(), ..Default::default() }),
            Arc::new(Category { category_id: "SPORTS".into(), category_name: "Sports".into(), ..Default::default() }),
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
        
        // Trigger the logic we added to AsyncAction::TotalChannelsLoaded
        app.select_category(app.selected_category_index);

        // 5. Verify View
        // Should ONLY contain Sports stream
        assert_eq!(app.streams.len(), 1, "Should filter to Sports streams only");
        assert_eq!(app.streams[0].name, "ESPN");

        // 6. Verify Search Capability
        // Search "CNN" (which is NOT in visible view, but is in global)
        app.search_mode = true;
        app.search_state.query = "CNN".to_string();
        app.update_search();

        assert_eq!(app.streams.len(), 1, "Search should find CNN from global cache");
        assert_eq!(app.streams[0].name, "CNN");
    }
}
