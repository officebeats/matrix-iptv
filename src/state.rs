use std::collections::HashMap;
use std::sync::Arc;
use ratatui::widgets::ListState;
use ratatui::layout::Rect;

use crate::api::{Category, IptvClient, Stream, UserInfo, ServerInfo, SeriesEpisode, SeriesInfo, VodInfo};
use crate::errors::LoadingProgress;
use crate::app::{CurrentScreen, Pane, MatrixColumn};
use crate::sports::{StreamedMatch, StreamedStream};
use crate::scores::ScoreGame;

/// Session state for provider connection
#[derive(Default)]
pub struct SessionState {
    /// Active IPTV client
    pub current_client: Option<IptvClient>,
    /// User account details
    pub account_info: Option<UserInfo>,
    /// Server connection info
    pub server_info: Option<ServerInfo>,
    /// Provider's timezone
    pub provider_timezone: Option<String>,
    /// Total available channels
    pub total_channels: usize,
    /// Total available movies
    pub total_movies: usize,
    /// Total available series
    pub total_series: usize,
    /// User's timezone cache
    pub cached_user_timezone: String,
    /// Background refresh flag
    pub background_refresh_active: bool,
    /// Session loaded from cache
    pub cache_loaded: bool,
    /// Global loading state
    pub state_loading: bool,
    /// Loading message display
    pub loading_message: Option<String>,
    /// Progress tracking
    pub loading_progress: Option<LoadingProgress>,
    /// Loading animation tick
    pub loading_tick: u64,
    /// Selected account index
    pub selected_account_index: usize,
}

impl SessionState {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Check if session is connected
    pub fn is_connected(&self) -> bool {
        self.current_client.is_some()
    }
    
    /// Clear session data (logout)
    pub fn clear(&mut self) {
        self.current_client = None;
        self.account_info = None;
        self.server_info = None;
        self.provider_timezone = None;
        self.total_channels = 0;
        self.total_movies = 0;
        self.total_series = 0;
        self.cached_user_timezone.clear();
        self.background_refresh_active = false;
        self.cache_loaded = false;
        self.state_loading = false;
        self.loading_message = None;
        self.loading_progress = None;
        self.loading_tick = 0;
    }
}

/// Content state for categories and streams
/// Used for Live channels, VOD movies, and Series
pub struct ContentState {
    /// All categories (unfiltered)
    pub all_categories: Vec<Arc<Category>>,
    /// Filtered categories
    pub categories: Vec<Arc<Category>>,
    /// Selected category index
    pub selected_category_index: usize,
    /// Category list UI state
    pub category_list_state: ListState,
    /// All streams (unfiltered)
    pub all_streams: Vec<Arc<Stream>>,
    /// Filtered streams
    pub streams: Vec<Arc<Stream>>,
    /// Selected stream index
    pub selected_stream_index: usize,
    /// Stream list UI state
    pub stream_list_state: ListState,
    /// Category channel counts (Live only)
    pub category_channel_counts: HashMap<String, usize>,
    /// EPG data cache (Live only)
    pub epg_cache: HashMap<String, String>,
}

impl Default for ContentState {
    fn default() -> Self {
        Self {
            all_categories: Vec::new(),
            categories: Vec::new(),
            selected_category_index: 0,
            category_list_state: ListState::default(),
            all_streams: Vec::new(),
            streams: Vec::new(),
            selected_stream_index: 0,
            stream_list_state: ListState::default(),
            category_channel_counts: HashMap::new(),
            epg_cache: HashMap::new(),
        }
    }
}

impl ContentState {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Select a category by index
    pub fn select_category(&mut self, index: usize) {
        self.selected_category_index = index;
        if index < self.categories.len() {
            self.category_list_state.select(Some(index));
        }
    }
    
    /// Select a stream by index
    pub fn select_stream(&mut self, index: usize) {
        self.selected_stream_index = index;
        if index < self.streams.len() {
            self.stream_list_state.select(Some(index));
        }
    }
    
    /// Get the selected category
    pub fn selected_category(&self) -> Option<&Arc<Category>> {
        self.categories.get(self.selected_category_index)
    }
    
    /// Get the selected stream
    pub fn selected_stream(&self) -> Option<&Arc<Stream>> {
        self.streams.get(self.selected_stream_index)
    }
    
    /// Clear all content
    pub fn clear(&mut self) {
        self.all_categories.clear();
        self.categories.clear();
        self.selected_category_index = 0;
        self.all_streams.clear();
        self.streams.clear();
        self.selected_stream_index = 0;
        self.category_channel_counts.clear();
        self.epg_cache.clear();
    }
    
    /// Check if content is loaded
    pub fn has_content(&self) -> bool {
        !self.all_categories.is_empty() || !self.all_streams.is_empty()
    }
}

/// Series state extending ContentState with episode handling
pub struct SeriesState {
    /// Base content state
    pub content: ContentState,
    /// Episodes for selected series
    pub series_episodes: Vec<SeriesEpisode>,
    /// Selected episode index
    pub selected_series_episode_index: usize,
    /// Episode list state
    pub series_episode_list_state: ListState,
    /// Selected series metadata
    pub current_series_info: Option<SeriesInfo>,
}

impl Default for SeriesState {
    fn default() -> Self {
        Self {
            content: ContentState::default(),
            series_episodes: Vec::new(),
            selected_series_episode_index: 0,
            series_episode_list_state: ListState::default(),
            current_series_info: None,
        }
    }
}

impl SeriesState {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Select an episode by index
    pub fn select_episode(&mut self, index: usize) {
        self.selected_series_episode_index = index;
        if index < self.series_episodes.len() {
            self.series_episode_list_state.select(Some(index));
        }
    }
    
    /// Get the selected episode
    pub fn selected_episode(&self) -> Option<&SeriesEpisode> {
        self.series_episodes.get(self.selected_series_episode_index)
    }
    
    /// Clear series-specific data
    pub fn clear_series(&mut self) {
        self.series_episodes.clear();
        self.selected_series_episode_index = 0;
        self.current_series_info = None;
    }
}

/// VOD state with movie info
pub struct VodState {
    /// Base content state
    pub content: ContentState,
    /// Selected VOD metadata
    pub current_vod_info: Option<VodInfo>,
}

impl Default for VodState {
    fn default() -> Self {
        Self {
            content: ContentState::default(),
            current_vod_info: None,
        }
    }
}

impl VodState {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Login form field
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginField {
    #[default]
    Name,
    Url,
    Username,
    Password,
    EpgUrl,
    ServerTimezone,
}

/// Login form state
#[derive(Default)]
pub struct LoginFormState {
    /// Focused form field
    pub field_focus: LoginField,
    /// Account name input
    pub input_name: tui_input::Input,
    /// Server URL input
    pub input_url: tui_input::Input,
    /// Username input
    pub input_username: tui_input::Input,
    /// Password input
    pub input_password: tui_input::Input,
    /// EPG URL input
    pub input_epg_url: tui_input::Input,
    /// Server timezone input
    pub input_server_timezone: tui_input::Input,
    /// Login error message
    pub error: Option<String>,
    /// Account being edited
    pub editing_index: Option<usize>,
    /// Account list UI state
    pub account_list_state: ListState,
}

impl LoginFormState {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Clear form
    pub fn clear(&mut self) {
        self.input_name.reset();
        self.input_url.reset();
        self.input_username.reset();
        self.input_password.reset();
        self.input_epg_url.reset();
        self.input_server_timezone.reset();
        self.error = None;
        self.editing_index = None;
    }
}

/// UI state for navigation and display
pub struct UiState {
    /// Active screen
    pub current_screen: CurrentScreen,
    /// Navigation history
    pub previous_screen: Option<CurrentScreen>,
    /// Active pane
    pub active_pane: Pane,
    /// Quit flag
    pub should_quit: bool,
    /// Help overlay
    pub show_help: bool,
    /// Save confirmation popup
    pub show_save_confirmation: bool,
    /// Welcome popup
    pub show_welcome_popup: bool,
    /// Play details popup
    pub show_play_details: bool,
    /// Cast device picker
    pub show_cast_picker: bool,
    /// Player error display
    pub player_error: Option<String>,
    /// Category pane bounds
    pub area_categories: Rect,
    /// Stream pane bounds
    pub area_streams: Rect,
    /// Account pane bounds
    pub area_accounts: Rect,
    /// About content
    pub about_text: String,
    /// About scroll position
    pub about_scroll: u16,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            current_screen: CurrentScreen::Home,
            previous_screen: None,
            active_pane: Pane::Categories,
            should_quit: false,
            show_help: false,
            show_save_confirmation: false,
            show_welcome_popup: false,
            show_play_details: false,
            show_cast_picker: false,
            player_error: None,
            area_categories: Rect::default(),
            area_streams: Rect::default(),
            area_accounts: Rect::default(),
            about_text: String::new(),
            about_scroll: 0,
        }
    }
}

impl UiState {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Navigate to a screen, saving history
    pub fn navigate_to(&mut self, screen: CurrentScreen) {
        self.previous_screen = Some(self.current_screen.clone());
        self.current_screen = screen;
    }
    
    /// Go back to previous screen
    pub fn go_back(&mut self) {
        if let Some(prev) = self.previous_screen.take() {
            self.current_screen = prev;
        }
    }
}

/// Sports state for live matches
#[derive(Default)]
pub struct SportsState {
    /// Live sports matches
    pub matches: Vec<StreamedMatch>,
    /// Sports list state
    pub list_state: ListState,
    /// Sports categories
    pub categories: Vec<String>,
    /// Sports category list state
    pub category_list_state: ListState,
    /// Selected sports category index
    pub selected_category_index: usize,
    /// Current sports streams
    pub streams: Vec<StreamedStream>,
    /// Sports loading flag
    pub loading: bool,
    /// ESPN live scores
    pub live_scores: Vec<ScoreGame>,
}

impl SportsState {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Matrix rain animation state
pub struct MatrixRainState {
    /// Animation active
    pub active: bool,
    /// Animation start time (platform-specific)
    #[cfg(not(target_arch = "wasm32"))]
    pub start_time: Option<std::time::Instant>,
    #[cfg(target_arch = "wasm32")]
    pub start_time: Option<f64>,
    /// Screensaver vs startup mode
    pub screensaver_mode: bool,
    /// Column animation data
    pub columns: Vec<MatrixColumn>,
    /// Logo pixel activation
    pub logo_hits: Vec<bool>,
}

impl Default for MatrixRainState {
    fn default() -> Self {
        Self {
            active: false,
            start_time: None,
            screensaver_mode: false,
            columns: Vec::new(),
            logo_hits: Vec::new(),
        }
    }
}

impl MatrixRainState {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Search state for content filtering
#[derive(Default)]
pub struct SearchState {
    /// Search query
    pub query: String,
    /// Search active flag
    pub active: bool,
    /// Last search query (for incremental)
    pub last_query: String,
    /// Global search results
    pub results: Vec<Arc<Stream>>,
    /// Search results list state
    pub list_state: ListState,
}

impl SearchState {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Clear search
    pub fn clear(&mut self) {
        self.query.clear();
        self.active = false;
        self.last_query.clear();
        self.results.clear();
    }
}

/// Group management state
#[derive(Default)]
pub struct GroupManagementState {
    /// Selected group index
    pub selected_index: usize,
    /// Group list state
    pub list_state: ListState,
    /// Stream awaiting group assignment
    pub pending_stream: Option<String>,
    /// Group name input
    pub name_input: String,
}

impl GroupManagementState {
    pub fn new() -> Self {
        Self::default()
    }
}
