use crate::api::{Category, ServerInfo, Stream, UserInfo, IptvClient, XtreamClient};
use crate::config::AppConfig;
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
    Type,
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
    pub active_account_type: crate::config::AccountType,
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
    pub all_categories: Vec<Category>,
    pub categories: Vec<Category>,
    pub selected_category_index: usize,
    pub category_list_state: ListState,

    // Streams
    pub all_streams: Vec<Stream>,
    pub streams: Vec<Stream>,
    pub selected_stream_index: usize,
    pub stream_list_state: ListState,

    // VOD Categories
    pub all_vod_categories: Vec<Category>,
    pub vod_categories: Vec<Category>,
    pub selected_vod_category_index: usize,
    pub vod_category_list_state: ListState,

    // VOD Streams
    pub all_vod_streams: Vec<Stream>,
    pub vod_streams: Vec<Stream>,
    pub selected_vod_stream_index: usize,
    pub vod_stream_list_state: ListState,

    // Series Data
    pub all_series_categories: Vec<Category>,
    pub series_categories: Vec<Category>,
    pub selected_series_category_index: usize,
    pub series_category_list_state: ListState,

    pub all_series_streams: Vec<Stream>, // Series are treated as 'Streams' for listing
    pub series_streams: Vec<Stream>,
    pub selected_series_stream_index: usize,
    pub series_stream_list_state: ListState,

    // Series Episodes (for 3-column view)
    pub series_episodes: Vec<crate::api::SeriesEpisode>,
    pub selected_series_episode_index: usize,
    pub series_episode_list_state: ListState,
    pub current_series_info: Option<crate::api::SeriesInfo>,
    pub current_vod_info: Option<crate::api::VodInfo>,
    
    // Global caches for "ALL" categories
    pub global_all_streams: Vec<Stream>,
    pub global_all_vod_streams: Vec<Stream>,
    pub global_all_series_streams: Vec<Stream>,

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

    // Auto-Refresh selection
    pub auto_refresh_list_state: ListState,

    // Editing
    pub editing_account_index: Option<usize>,

    // 2-Pane Navigation
    pub active_pane: Pane,

    // Search/Filter
    pub search_query: String,
    pub search_mode: bool,

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

    // Global Search
    pub global_search_results: Vec<Stream>,
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

    // Chromecast Casting
    #[cfg(all(not(target_arch = "wasm32"), feature = "chromecast"))]
    pub cast_manager: crate::cast::CastManager,
    pub cast_devices: Vec<CastDevice>,
    pub cast_device_list_state: ListState,
    pub show_cast_picker: bool,
    pub cast_discovering: bool,
    pub selected_cast_device_index: usize,
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

            editing_account_index: None,

            selected_account_index: 0,
            account_list_state,

            login_field_focus: LoginField::Name,
            active_account_type: crate::config::AccountType::Xtream,
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
            auto_refresh_list_state: ListState::default(),

            active_pane: Pane::Categories,
            search_query: String::new(),
            search_mode: false,
            show_help: false,
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

            // Chromecast Casting
            #[cfg(all(not(target_arch = "wasm32"), feature = "chromecast"))]
            cast_manager: crate::cast::CastManager::new(),
            cast_devices: Vec::new(),
            cast_device_list_state: ListState::default(),
            show_cast_picker: false,
            cast_discovering: false,
            selected_cast_device_index: 0,
        };

        app.refresh_settings_options();
        app
    }

    pub fn refresh_settings_options(&mut self) {
        self.settings_options = vec![
            "Manage Playlists".to_string(),
            format!(
                "Set Timezone (Current: {})",
                self.config.get_user_timezone()
            ),
            format!(
                "Playlist Mode: {}",
                self.config.playlist_mode.display_name()
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
            "Enhanced = Interpolation, upscaling, and soap opera effect for smoother video. MPV Default = Standard MPV settings with no enhancements.".to_string(),
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

    pub fn get_selected_category(&self) -> Option<&Category> {
        self.categories.get(self.selected_category_index)
    }

    pub fn get_selected_stream(&self) -> Option<&Stream> {
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
        Self::navigate_list(
            self.categories.len(),
            &mut self.selected_category_index,
            &mut self.category_list_state,
            true,
        );
    }

    pub fn previous_category(&mut self) {
        Self::navigate_list(
            self.categories.len(),
            &mut self.selected_category_index,
            &mut self.category_list_state,
            false,
        );
    }

    pub fn next_stream(&mut self) {
        Self::navigate_list(
            self.streams.len(),
            &mut self.selected_stream_index,
            &mut self.stream_list_state,
            true,
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

    pub fn previous_global_search_result(&mut self) {
        Self::navigate_list(
            self.global_search_results.len(),
            &mut self.selected_stream_index,
            &mut self.global_search_list_state,
            false,
        );
    }

    pub fn update_search(&mut self) {
        let query = self.search_query.to_lowercase();
        let is_merica = self.config.playlist_mode.is_merica_variant();

        match self.current_screen {
            CurrentScreen::Categories | CurrentScreen::Streams => {
                match self.active_pane {
                    Pane::Categories => {
                        self.categories = self
                            .all_categories
                            .iter()
                            .filter(|c| {
                                c.search_name.contains(&query) && (!is_merica || c.is_american)
                            })
                            .map(|c| {
                                if !is_merica { return c.clone(); }
                                let mut c_mod = c.clone();
                                c_mod.category_name = c_mod.clean_name.clone();
                                c_mod
                            })
                            .collect();
                        self.selected_category_index = 0;
                        if !self.categories.is_empty() {
                            self.category_list_state.select(Some(0));
                        } else {
                            self.category_list_state.select(None);
                        }
                    }
                    Pane::Streams => {
                        use rayon::prelude::*;
                        let mut filtered: Vec<Stream> = self
                            .all_streams
                            .par_iter()
                            .filter(|s| {
                                s.search_name.contains(&query) && (!is_merica || s.is_american)
                            })
                            .map(|s| {
                                if !is_merica { return s.clone(); }
                                let mut s_mod = s.clone();
                                s_mod.name = s_mod.clean_name.clone();
                                s_mod
                            })
                            .collect();
                        filtered.truncate(1000);
                        self.streams = filtered;
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
                        self.vod_categories = self
                            .all_vod_categories
                            .iter()
                            .filter(|c| {
                                c.search_name.contains(&query) && (!is_merica || c.is_english)
                            })
                            .map(|c| {
                                if !is_merica { return c.clone(); }
                                let mut c_mod = c.clone();
                                c_mod.category_name = c_mod.clean_name.clone();
                                c_mod
                            })
                            .collect();
                        self.selected_vod_category_index = 0;
                        if !self.vod_categories.is_empty() {
                            self.vod_category_list_state.select(Some(0));
                        } else {
                            self.vod_category_list_state.select(None);
                        }
                    }
                    Pane::Streams => {
                        use rayon::prelude::*;
                        let mut filtered: Vec<Stream> = self
                            .all_vod_streams
                            .par_iter()
                            .filter(|s| {
                                s.search_name.contains(&query) && (!is_merica || s.is_english)
                            })
                            .map(|s| {
                                if !is_merica { return s.clone(); }
                                let mut s_mod = s.clone();
                                s_mod.name = s_mod.clean_name.clone();
                                s_mod
                            })
                            .collect();
                        filtered.truncate(1000);
                        self.vod_streams = filtered;
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
                        self.series_categories = self
                            .all_series_categories
                            .iter()
                            .filter(|c| {
                                c.search_name.contains(&query) && (!is_merica || c.is_english)
                            })
                            .map(|c| {
                                if !is_merica { return c.clone(); }
                                let mut c_mod = c.clone();
                                c_mod.category_name = c_mod.clean_name.clone();
                                c_mod
                            })
                            .collect();
                        self.selected_series_category_index = 0;
                        if !self.series_categories.is_empty() {
                            self.series_category_list_state.select(Some(0));
                        } else {
                            self.series_category_list_state.select(None);
                        }
                    }
                    Pane::Streams => {
                        use rayon::prelude::*;
                        let mut filtered: Vec<Stream> = self
                            .all_series_streams
                            .par_iter()
                            .filter(|s| {
                                s.search_name.contains(&query) && (!is_merica || s.is_english)
                            })
                            .map(|s| {
                                if !is_merica { return s.clone(); }
                                let mut s_mod = s.clone();
                                s_mod.name = s_mod.clean_name.clone();
                                s_mod
                            })
                            .collect();
                        filtered.truncate(1000);
                        self.series_streams = filtered;
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
                use rayon::prelude::*;
                // Search through global caches in parallel
                let mut results: Vec<_> = self.global_all_streams
                    .par_iter()
                    .filter(|s| s.search_name.contains(&query))
                    .map(|s| {
                        if !is_merica { return s.clone(); }
                        let mut s_mod = s.clone();
                        s_mod.name = s_mod.clean_name.clone();
                        s_mod
                    })
                    .collect();
                results.truncate(100);

                if results.len() < 100 {
                    let mut movie_results: Vec<_> = self.global_all_vod_streams
                        .par_iter()
                        .filter(|s| s.search_name.contains(&query))
                        .map(|s| {
                            if !is_merica { return s.clone(); }
                            let mut s_mod = s.clone();
                            s_mod.name = s_mod.clean_name.clone();
                            s_mod
                        })
                        .collect();
                    movie_results.truncate(100 - results.len());
                    results.extend(movie_results);
                }

                if results.len() < 100 {
                    let mut series_results: Vec<_> = self.global_all_series_streams
                        .par_iter()
                        .filter(|s| s.search_name.contains(&query))
                        .map(|s| {
                            if !is_merica { return s.clone(); }
                            let mut s_mod = s.clone();
                            s_mod.name = s_mod.clean_name.clone();
                            s_mod
                        })
                        .collect();
                    series_results.truncate(100 - results.len());
                    results.extend(series_results);
                }

                self.global_search_results = results;
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
                account_type: self.active_account_type,
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
                        self.search_query.clear();
                    }
                    KeyCode::Char('2') => {
                        self.current_screen = CurrentScreen::VodCategories;
                        self.active_pane = Pane::Categories;
                        self.search_mode = false;
                        self.search_query.clear();
                    }
                    KeyCode::Char('3') => {
                        self.current_screen = CurrentScreen::SeriesCategories;
                        self.active_pane = Pane::Categories;
                        self.search_mode = false;
                        self.search_query.clear();
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
                            self.search_query.clear();
                        }
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Esc | KeyCode::Backspace => {
                            self.current_screen = CurrentScreen::ContentTypeSelection;
                            self.search_mode = false;
                            self.search_query.clear();
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
                    self.current_screen = CurrentScreen::ContentTypeSelection;
                    self.search_mode = false;
                    self.search_query.clear();
                }
                KeyCode::Char('j') | KeyCode::Down => self.next_series_category(),
                KeyCode::Char('k') | KeyCode::Up => self.previous_series_category(),
                _ => {}
            },
            CurrentScreen::SeriesStreams => match key.code {
                KeyCode::Esc | KeyCode::Backspace | KeyCode::Left => {
                    self.series_streams.clear();
                    self.all_series_streams.clear();
                    self.selected_series_stream_index = 0;
                    self.series_stream_list_state.select(None);
                    self.current_screen = CurrentScreen::SeriesCategories;
                }
                KeyCode::Char('j') | KeyCode::Down => self.next_series_stream(),
                KeyCode::Char('k') | KeyCode::Up => self.previous_series_stream(),
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
            Category {
                category_id: "1".into(),
                category_name: "Action".into(),
                parent_id: serde_json::Value::Null,
                ..Default::default()
            },
            Category {
                category_id: "2".into(),
                category_name: "Comedy".into(),
                parent_id: serde_json::Value::Null,
                ..Default::default()
            },
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
}
