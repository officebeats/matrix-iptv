use std::time::Duration;
use thiserror::Error;

/// Detailed connection stage for diagnostic purposes
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStage {
    /// DNS resolution failed to find the server
    DnsResolution,
    /// TCP connection to server failed
    TcpConnection,
    /// TLS handshake failed
    TlsHandshake,
    /// HTTP request failed to connect
    HttpHandshake,
    /// Authentication with server failed
    Authentication,
    /// Failed to parse server response
    ResponseParsing,
}

impl std::fmt::Display for ConnectionStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl ConnectionStage {
    /// Get a user-friendly name for the stage
    pub fn display_name(&self) -> &'static str {
        match self {
            ConnectionStage::DnsResolution => "DNS Resolution",
            ConnectionStage::TcpConnection => "TCP Connection",
            ConnectionStage::TlsHandshake => "TLS Handshake",
            ConnectionStage::HttpHandshake => "HTTP Handshake",
            ConnectionStage::Authentication => "Authentication",
            ConnectionStage::ResponseParsing => "Response Parsing",
        }
    }

    /// Get actionable suggestion for fixing the issue at this stage
    pub fn suggestion(&self) -> &'static str {
        match self {
            ConnectionStage::DnsResolution => {
                "Try using a different DNS provider (Quad9 or Cloudflare) in settings, or check internet connection."
            }
            ConnectionStage::TcpConnection => {
                "Server appears to be offline or blocking your IP. Try checking the server URL or use a VPN."
            }
            ConnectionStage::TlsHandshake => {
                "SSL certificate error. Try disabling certificate verification in settings (not recommended)."
            }
            ConnectionStage::HttpHandshake => {
                "Server is not responding properly. Check the URL and try again later."
            }
            ConnectionStage::Authentication => {
                "Username or password is incorrect. Verify your credentials."
            }
            ConnectionStage::ResponseParsing => {
                "Server response is invalid. This may be a provider issue. Try again later."
            }
        }
    }
}

/// Detailed error type for IPTV operations
#[derive(Debug, Error, Clone)]
pub enum IptvError {
    /// DNS resolution failed
    #[error("DNS resolution failed for {0}: {1}")]
    DnsResolution(String, String),

    /// Connection timeout
    #[error("Connection timeout after {1}s to {0}")]
    ConnectionTimeout(String, u64),

    /// Connection failed
    #[error("Connection failed at {0}: {1}")]
    ConnectionFailed(ConnectionStage, String),

    /// Authentication failed
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    /// Server returned an error status
    #[error("Server returned {0}: {1}")]
    ServerError(u16, String),

    /// Failed to parse response
    #[error("Failed to parse response: {0}")]
    ParseError(String),

    /// Empty or invalid response
    #[error("Empty or invalid response received: {0}")]
    EmptyResponse(String),

    /// ISP block detected
    #[error("ISP BLOCK DETECTED: Your provider (likely AT&T) is blocking this IPTV server. Disable 'Home Network Security' or use a VPN.")]
    IspBlock,

    /// Generic error
    #[error("Error: {0}")]
    Generic(String),
}

impl IptvError {
    /// Get detailed diagnostic information about the error
    pub fn diagnostics(&self) -> String {
        match self {
            IptvError::DnsResolution(host, source) => {
                format!("DNS Resolution Error\nHost: {}\nError: {}\nSuggestion: Try using Quad9 DNS", host, source)
            }
            IptvError::ConnectionTimeout(host, timeout) => {
                format!("Connection Timeout\nHost: {}\nTimeout: {} seconds\nSuggestion: Server is slow or offline", host, timeout)
            }
            IptvError::ConnectionFailed(stage, source) => {
                format!("Connection Failed at {}\nError: {}\nSuggestion: {}", stage.display_name(), source, stage.suggestion())
            }
            IptvError::AuthenticationFailed(reason) => {
                format!("Authentication Failed\nReason: {}\nSuggestion: Verify username and password", reason)
            }
            IptvError::ServerError(status, message) => {
                format!("Server Error\nStatus: {}\nMessage: {}\nSuggestion: Try again later", status, message)
            }
            IptvError::ParseError(source) => {
                format!("Parse Error\nError: {}\nSuggestion: Provider response is invalid", source)
            }
            IptvError::EmptyResponse(reason) => {
                format!("Empty Response\nReason: {}\nSuggestion: Try again later", reason)
            }
            IptvError::IspBlock => {
                format!("ISP Block Detected\nSuggestion: Disable AT&T Home Network Security or use a VPN")
            }
            IptvError::Generic(message) => {
                format!("Error\nMessage: {}\nSuggestion: Try again", message)
            }
        }
    }
}

/// Loading progress tracking
#[derive(Debug, Clone)]
pub struct LoadingProgress {
    pub stage: LoadingStage,
    pub current: usize,
    pub total: usize,
    pub eta: Option<Duration>,
}

impl LoadingProgress {
    pub fn new(stage: LoadingStage, current: usize, total: usize) -> Self {
        Self {
            stage,
            current,
            total,
            eta: None,
        }
    }

    pub fn with_eta(mut self, eta: Duration) -> Self {
        self.eta = Some(eta);
        self
    }

    pub fn to_message(&self) -> String {
        let progress = format!("[{}/{}]", self.current, self.total);
        let eta = self.eta.as_ref().map(|d| format!(" ETA: {}s", d.as_secs())).unwrap_or_default();
        format!("{} - {} {}", self.stage.display_name(), progress, eta)
    }
}

/// Loading stages for the application
#[derive(Debug, Clone, PartialEq)]
pub enum LoadingStage {
    Initializing,
    Authenticating,
    FetchingCategories,
    FetchingStreams { category: String },
    Preprocessing,
    Indexing,
    Complete,
}

impl LoadingStage {
    pub fn display_name(&self) -> String {
        match self {
            LoadingStage::Initializing => "Initializing Application".to_string(),
            LoadingStage::Authenticating => "Authenticating".to_string(),
            LoadingStage::FetchingCategories => "Fetching Categories".to_string(),
            LoadingStage::FetchingStreams { category } => format!("Loading {} Streams", category),
            LoadingStage::Preprocessing => "Preprocessing Content".to_string(),
            LoadingStage::Indexing => "Building Search Index".to_string(),
            LoadingStage::Complete => "Complete".to_string(),
        }
    }
}

/// Search state with history and suggestions
#[derive(Debug, Clone)]
pub struct SearchState {
    pub query: String,
    pub history: VecDeque<String>,
    pub suggestions: Vec<String>,
    pub last_search_time: Option<Instant>,
    pub debounce_timer: Option<Instant>,
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            query: String::new(),
            history: VecDeque::with_capacity(20),
            suggestions: Vec::new(),
            last_search_time: None,
            debounce_timer: None,
        }
    }
}

impl SearchState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_to_history(&mut self, query: String) {
        if !query.is_empty() && !self.history.contains(&query) {
            if self.history.len() >= 20 {
                self.history.pop_back();
            }
            self.history.push_front(query);
        }
    }

    pub fn get_suggestions(&self, partial: &str) -> Vec<String> {
        self.history
            .iter()
            .filter(|h| h.to_lowercase().contains(&partial.to_lowercase()))
            .take(5)
            .cloned()
            .collect()
    }

    pub fn should_search(&mut self, query: &str, debounce_ms: u64) -> bool {
        if query.is_empty() {
            return true;
        }

        let now = Instant::now();
        match self.debounce_timer {
            None => {
                self.debounce_timer = Some(now);
                false
            }
            Some(last) => {
                if now.duration_since(last).as_millis() >= debounce_ms.into() {
                    self.debounce_timer = Some(now);
                    true
                } else {
                    false
                }
            }
        }
    }
}

// Re-export necessary types
pub use std::collections::VecDeque;
pub use std::time::Instant;
