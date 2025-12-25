use crate::api::{Category, ServerInfo, Stream, UserInfo, XtreamClient};
use crate::config::AppConfig;
use crate::parser::{is_american_live, is_english_vod, clean_american_name, parse_stream};
use ratatui::layout::Rect;
use ratatui::widgets::ListState;
use tui_input::Input;

#[derive(Debug, Clone)]
pub enum AsyncAction {
    LoginSuccess(XtreamClient, Option<UserInfo>, Option<ServerInfo>),
    LoginFailed(String),
    CategoriesLoaded(Vec<Category>),
    StreamsLoaded(Vec<Stream>),
    VodCategoriesLoaded(Vec<Category>),
    VodStreamsLoaded(Vec<Stream>),
    SeriesCategoriesLoaded(Vec<Category>),
    SeriesStreamsLoaded(Vec<Stream>),
    SeriesInfoLoaded(crate::api::SeriesInfo),
    PlayerStarted,
    PlayerFailed(String),
    LoadingMessage(String),
    TotalChannelsLoaded(usize),
    TotalMoviesLoaded(usize),
    TotalSeriesLoaded(usize),
    PlaylistRefreshed(Option<UserInfo>, Option<ServerInfo>),
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
}

#[derive(PartialEq, Debug)]
pub enum InputMode {
    Normal,
    Editing,
}

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
    pub current_client: Option<XtreamClient>,

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

    // Settings
    pub settings_options: Vec<String>,
    pub selected_settings_index: usize,
    pub settings_list_state: ListState,
    pub selected_content_type_index: usize,

    // Timezone selection
    pub input_timezone: Input,
    pub timezone_list: Vec<String>,
    pub timezone_list_state: ListState,

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
    pub matrix_rain_start_time: Option<std::time::Instant>,
    pub matrix_rain_screensaver_mode: bool, // true = screensaver (no logo), false = startup (with logo)
    pub show_welcome_popup: bool,
    pub matrix_rain_columns: Vec<MatrixColumn>,
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
        let matrix_rain_start = Some(std::time::Instant::now());

        let mut app = App {
            cached_user_timezone: config.get_user_timezone(),
            config,
            current_screen: CurrentScreen::Home,
            input_mode: InputMode::Normal,
            should_quit: false,
            state_loading: false,

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

            settings_options: vec![],
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
                "American Playlist Mode: {}",
                if self.config.american_mode {
                    "ON 🇺🇸"
                } else {
                    "OFF"
                }
            ),
            "Matrix Rain Screensaver".to_string(),
            "About".to_string(),
        ];

        if self.settings_list_state.selected().is_none() {
            self.settings_list_state.select(Some(0));
        }
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

    pub fn next_account(&mut self) {
        let len = self.config.accounts.len();
        if len > 0 {
            let i = match self.account_list_state.selected() {
                Some(i) => (i + 1) % len,
                None => 0,
            };
            self.selected_account_index = i;
            self.account_list_state.select(Some(i));
        }
    }

    pub fn previous_account(&mut self) {
        let len = self.config.accounts.len();
        if len > 0 {
            let i = match self.account_list_state.selected() {
                Some(i) => {
                    if i == 0 {
                        len - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.selected_account_index = i;
            self.account_list_state.select(Some(i));
        }
    }

    pub fn next_category(&mut self) {
        let len = self.categories.len();
        if len > 0 {
            let i = match self.category_list_state.selected() {
                Some(i) => (i + 1) % len,
                None => 0,
            };
            self.selected_category_index = i;
            self.category_list_state.select(Some(i));
        }
    }

    pub fn previous_category(&mut self) {
        let len = self.categories.len();
        if len > 0 {
            let i = match self.category_list_state.selected() {
                Some(i) => {
                    if i == 0 {
                        len - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.selected_category_index = i;
            self.category_list_state.select(Some(i));
        }
    }

    pub fn next_stream(&mut self) {
        let len = self.streams.len();
        if len > 0 {
            let i = match self.stream_list_state.selected() {
                Some(i) => (i + 1) % len,
                None => 0,
            };
            self.selected_stream_index = i;
            self.stream_list_state.select(Some(i));
        }
    }

    pub fn previous_stream(&mut self) {
        let len = self.streams.len();
        if len > 0 {
            let i = match self.stream_list_state.selected() {
                Some(i) => {
                    if i == 0 {
                        len - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.selected_stream_index = i;
            self.stream_list_state.select(Some(i));
        }
    }

    pub fn next_vod_category(&mut self) {
        let len = self.vod_categories.len();
        if len > 0 {
            let i = match self.vod_category_list_state.selected() {
                Some(i) => (i + 1) % len,
                None => 0,
            };
            self.selected_vod_category_index = i;
            self.vod_category_list_state.select(Some(i));
        }
    }

    pub fn previous_vod_category(&mut self) {
        let len = self.vod_categories.len();
        if len > 0 {
            let i = match self.vod_category_list_state.selected() {
                Some(i) => {
                    if i == 0 {
                        len - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.selected_vod_category_index = i;
            self.vod_category_list_state.select(Some(i));
        }
    }

    pub fn next_vod_stream(&mut self) {
        let len = self.vod_streams.len();
        if len > 0 {
            let i = match self.vod_stream_list_state.selected() {
                Some(i) => (i + 1) % len,
                None => 0,
            };
            self.selected_vod_stream_index = i;
            self.vod_stream_list_state.select(Some(i));
        }
    }

    pub fn previous_vod_stream(&mut self) {
        let len = self.vod_streams.len();
        if len > 0 {
            let i = match self.vod_stream_list_state.selected() {
                Some(i) => {
                    if i == 0 {
                        len - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.selected_vod_stream_index = i;
            self.vod_stream_list_state.select(Some(i));
        }
    }

    pub fn next_setting(&mut self) {
        let len = self.settings_options.len();
        if len > 0 {
            let i = match self.settings_list_state.selected() {
                Some(i) => (i + 1) % len,
                None => 0,
            };
            self.selected_settings_index = i;
            self.settings_list_state.select(Some(i));
        }
    }

    pub fn previous_setting(&mut self) {
        let len = self.settings_options.len();
        if len > 0 {
            let i = match self.settings_list_state.selected() {
                Some(i) => {
                    if i == 0 {
                        len - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.selected_settings_index = i;
            self.settings_list_state.select(Some(i));
        }
    }

    pub fn update_search(&mut self) {
        let query = self.search_query.to_lowercase();

        match self.current_screen {
            CurrentScreen::Categories | CurrentScreen::Streams => {
                let american_mode = self.config.american_mode;
                match self.active_pane {
                    Pane::Categories => {
                        self.categories = self
                            .all_categories
                            .iter()
                            .filter(|c| {
                                let matches_query = c.category_name.to_lowercase().contains(&query);
                                if !american_mode {
                                    return matches_query;
                                }
                                // American Mode: USA channels & All Channels
                                matches_query && (c.category_id == "ALL" || is_american_live(&c.category_name))
                            })
                            .map(|c| {
                                if american_mode {
                                    let mut cleaned = c.clone();
                                    cleaned.category_name = clean_american_name(&cleaned.category_name);
                                    cleaned
                                } else {
                                    c.clone()
                                }
                            })
                            .collect();
                        // Reset selection
                        self.selected_category_index = 0;
                        if !self.categories.is_empty() {
                            self.category_list_state.select(Some(0));
                        } else {
                            self.category_list_state.select(None);
                        }
                    }
                    Pane::Streams => {
                        let provider_tz = self
                            .config
                            .accounts
                            .get(self.selected_account_index)
                            .and_then(|a| a.server_timezone.clone());

                        self.streams = self
                            .all_streams
                            .iter()
                            .filter(|s| {
                                let matches_query = s.name.to_lowercase().contains(&query);
                                if !american_mode {
                                    return matches_query;
                                }
                                // American Mode: USA streams
                                matches_query && is_american_live(&s.name)
                            })
                            .map(|s| {
                                let mut s_mod = s.clone();
                                if american_mode {
                                    s_mod.name = clean_american_name(&s_mod.name);
                                }
                                
                                // Eagerly parse and cache to allow instant scrolling
                                let effective_name = s_mod.stream_display_name.as_deref().unwrap_or(&s_mod.name).to_string();
                                s_mod.cached_parsed = Some(Box::new(parse_stream(&effective_name, provider_tz.as_deref())));
                                
                                s_mod
                            })
                            .collect();
                        // Reset selection
                        self.selected_stream_index = 0;
                        if !self.streams.is_empty() {
                            self.stream_list_state.select(Some(0));
                        } else {
                            self.stream_list_state.select(None);
                        }
                    }
                    Pane::Episodes => {
                        // Episodes don't support search in this context
                    }
                }
            }
            CurrentScreen::VodCategories | CurrentScreen::VodStreams => {
                let american_mode = self.config.american_mode;
                match self.active_pane {
                    Pane::Categories => {
                        self.vod_categories = self
                            .all_vod_categories
                            .iter()
                            .filter(|c| {
                                let matches_query = c.category_name.to_lowercase().contains(&query);
                                if !american_mode {
                                    return matches_query;
                                }
                                // American Mode: English VOD categories
                                matches_query && is_english_vod(&c.category_name)
                            })
                            .map(|c| {
                                if american_mode {
                                    let mut cleaned = c.clone();
                                    cleaned.category_name = clean_american_name(&cleaned.category_name);
                                    cleaned
                                } else {
                                    c.clone()
                                }
                            })
                            .collect();
                        // Reset selection
                        self.selected_vod_category_index = 0;
                        if !self.vod_categories.is_empty() {
                            self.vod_category_list_state.select(Some(0));
                        } else {
                            self.vod_category_list_state.select(None);
                        }
                    }
                    Pane::Streams => {
                        self.vod_streams = self
                            .all_vod_streams
                            .iter()
                            .filter(|s| {
                                let matches_query = s.name.to_lowercase().contains(&query);
                                if !american_mode {
                                    return matches_query;
                                }
                                // American Mode: English VOD streams
                                matches_query && is_english_vod(&s.name)
                            })
                            .map(|s| {
                                if american_mode {
                                    let mut cleaned = s.clone();
                                    cleaned.name = clean_american_name(&cleaned.name);
                                    cleaned
                                } else {
                                    s.clone()
                                }
                            })
                            .collect();
                        // Reset selection
                        self.selected_vod_stream_index = 0;
                        if !self.vod_streams.is_empty() {
                            self.vod_stream_list_state.select(Some(0));
                        } else {
                            self.vod_stream_list_state.select(None);
                        }
                    }
                    Pane::Episodes => {
                        // Episodes don't support search in VOD context
                    }
                }
            }
            CurrentScreen::SeriesCategories | CurrentScreen::SeriesStreams => {
                let american_mode = self.config.american_mode;
                match self.active_pane {
                    Pane::Categories => {
                        self.series_categories = self
                            .all_series_categories
                            .iter()
                            .filter(|c| {
                                let matches_query = c.category_name.to_lowercase().contains(&query);
                                if !american_mode {
                                    return matches_query;
                                }
                                // American Mode: English Series categories
                                matches_query && is_english_vod(&c.category_name)
                            })
                            .map(|c| {
                                if american_mode {
                                    let mut cleaned = c.clone();
                                    cleaned.category_name = clean_american_name(&cleaned.category_name);
                                    cleaned
                                } else {
                                    c.clone()
                                }
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
                        self.series_streams = self
                            .all_series_streams
                            .iter()
                            .filter(|s| {
                                let matches_query = s.name.to_lowercase().contains(&query);
                                if !american_mode {
                                    return matches_query;
                                }
                                // American Mode: English Series streams
                                matches_query && is_english_vod(&s.name)
                            })
                            .map(|s| {
                                if american_mode {
                                    let mut cleaned = s.clone();
                                    cleaned.name = clean_american_name(&cleaned.name);
                                    cleaned
                                } else {
                                    s.clone()
                                }
                            })
                            .collect();
                        self.selected_series_stream_index = 0;
                        if !self.series_streams.is_empty() {
                            self.series_stream_list_state.select(Some(0));
                        } else {
                            self.series_stream_list_state.select(None);
                        }
                    }
                    Pane::Episodes => {
                        // Filter episodes by title
                        // Note: We don't have all_series_episodes, so search is limited
                    }
                }
            }
            _ => {}
        }
    }

    // Series Navigation Helpers
    pub fn next_series_category(&mut self) {
        let i = match self.series_category_list_state.selected() {
            Some(i) => {
                if i >= self.series_categories.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.selected_series_category_index = i;
        self.series_category_list_state.select(Some(i));
    }

    pub fn previous_series_category(&mut self) {
        let i = match self.series_category_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.series_categories.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.selected_series_category_index = i;
        self.series_category_list_state.select(Some(i));
    }

    pub fn next_series_stream(&mut self) {
        let i = match self.series_stream_list_state.selected() {
            Some(i) => {
                if i >= self.series_streams.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.selected_series_stream_index = i;
        self.series_stream_list_state.select(Some(i));
    }

    pub fn previous_series_stream(&mut self) {
        let i = match self.series_stream_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.series_streams.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.selected_series_stream_index = i;
        self.series_stream_list_state.select(Some(i));
    }

    pub fn next_series_episode(&mut self) {
        if self.series_episodes.is_empty() {
            return;
        }
        let i = match self.series_episode_list_state.selected() {
            Some(i) => {
                if i >= self.series_episodes.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.selected_series_episode_index = i;
        self.series_episode_list_state.select(Some(i));
    }

    pub fn previous_series_episode(&mut self) {
        if self.series_episodes.is_empty() {
            return;
        }
        let i = match self.series_episode_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.series_episodes.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.selected_series_episode_index = i;
        self.series_episode_list_state.select(Some(i));
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
            },
            Category {
                category_id: "2".into(),
                category_name: "Comedy".into(),
                parent_id: serde_json::Value::Null,
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
