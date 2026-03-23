use crate::flex_id::{deserialize_flex_option_f32, FlexId};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};

static FUZZY_MATCHER: Lazy<SkimMatcherV2> = Lazy::new(SkimMatcherV2::default);

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Category {
    pub category_id: String,
    pub category_name: String,
    #[serde(default)]
    pub parent_id: FlexId, // frequent null or 0
    #[serde(skip)]
    pub search_name: String,
    #[serde(skip)]
    pub is_american: bool,
    #[serde(skip)]
    pub is_english: bool,
    #[serde(skip)]
    pub clean_name: String,
    #[serde(skip)]
    pub cached_parsed: Option<Box<crate::parser::ParsedCategory>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Stream {
    pub num: Option<FlexId>,
    #[serde(default)]
    pub name: String,
    pub stream_display_name: Option<String>,

    #[serde(default)]
    pub stream_type: String,

    #[serde(alias = "series_id", default)]
    pub stream_id: FlexId,

    #[serde(alias = "cover")]
    pub stream_icon: Option<String>,

    pub epg_channel_id: Option<String>,
    pub added: Option<String>,
    pub category_id: Option<String>,
    pub container_extension: Option<String>,
    #[serde(default, deserialize_with = "deserialize_flex_option_f32")]
    pub rating: Option<f32>,
    #[serde(default, deserialize_with = "deserialize_flex_option_f32")]
    pub rating_5: Option<f32>,

    #[serde(skip)]
    pub cached_parsed: Option<Box<crate::parser::ParsedStream>>,
    #[serde(skip)]
    pub search_name: String,
    #[serde(skip)]
    pub is_american: bool,
    #[serde(skip)]
    pub is_english: bool,
    #[serde(skip)]
    pub clean_name: String,
    #[serde(skip)]
    pub latency_ms: Option<u64>,
    #[serde(skip)]
    pub account_name: Option<String>,
    /// For M3U streams: the direct playback URL parsed from the playlist.
    /// For Xtream streams this is always `None` (URL is constructed via `get_stream_url()`).
    #[serde(skip)]
    pub direct_url: Option<String>,
}

impl Stream {
    /// Get or parse stream metadata with caching
    pub fn get_or_parse_cached(
        &mut self,
        provider_tz: Option<&str>,
    ) -> &crate::parser::ParsedStream {
        if self.cached_parsed.is_none() {
            self.cached_parsed = Some(Box::new(crate::parser::parse_stream(
                &self.name,
                provider_tz,
            )));
        }
        self.cached_parsed.as_ref().unwrap()
    }

    /// Fuzzy search match against query
    pub fn fuzzy_match(&self, query: &str, min_score: i64) -> bool {
        if query.is_empty() {
            return true;
        }

        if let Some(score) = FUZZY_MATCHER.fuzzy_match(&self.search_name, query) {
            score >= min_score
        } else {
            self.search_name.contains(&query.to_lowercase())
        }
    }
}

#[derive(Debug, Clone)]
pub struct XtreamClient {
    pub base_url: String,
    pub username: String,
    pub password: String,
    client: reqwest::Client,
    pending_requests: Arc<Mutex<HashMap<String, Arc<Notify>>>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserInfo {
    pub auth: i32,
    pub status: Option<String>,
    pub exp_date: Option<FlexId>,
    pub max_connections: Option<FlexId>,
    pub active_cons: Option<FlexId>,
    pub total_live_streams: Option<FlexId>,
    pub total_vod_streams: Option<FlexId>,
    pub total_series_streams: Option<FlexId>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerInfo {
    pub timezone: Option<String>,
    pub server_time: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SeriesEpisode {
    pub id: Option<FlexId>,
    pub episode_num: i32,
    pub title: Option<String>,
    pub container_extension: Option<String>,
    pub info: Option<serde_json::Value>,
    pub season: i32,
    #[serde(default)]
    pub direct_source: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SeriesInfo {
    pub seasons: Option<Vec<serde_json::Value>>,
    pub info: Option<serde_json::Value>,
    pub episodes: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct MovieData {
    #[serde(default)]
    pub stream_id: FlexId,
    pub name: Option<String>,
    pub added: Option<String>,
    pub category_id: Option<String>,
    pub container_extension: Option<String>,
    pub custom_sid: Option<FlexId>,
    pub direct_source: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct VodInfo {
    pub info: Option<serde_json::Value>,
    pub movie_data: Option<MovieData>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EpgListing {
    pub id: Option<String>,
    pub epg_id: Option<String>,
    pub title: String,
    pub start: String,
    pub end: String,
    pub description: Option<String>,
    pub start_timestamp: Option<FlexId>,
    pub stop_timestamp: Option<FlexId>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EpgResponse {
    pub epg_listings: Vec<EpgListing>,
}

/// Client for M3U/M3U8 playlist-based IPTV providers.
///
/// On authentication, the playlist URL is fetched and parsed. Parsed entries are cached
/// internally so that subsequent `get_live_categories()` / `get_live_streams()` calls are
/// cheap in-memory reads.
#[derive(Debug, Clone)]
pub struct M3uClient {
    /// The full M3U playlist URL (stored as-is; may contain credentials in query params).
    pub playlist_url: String,
    http_client: reqwest::Client,
    /// Cached parsed entries — populated once during `authenticate()`.
    entries: std::sync::Arc<std::sync::Mutex<Vec<crate::m3u_parser::M3uEntry>>>,
}

impl M3uClient {
    pub fn new(playlist_url: String) -> Self {
        let builder = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .danger_accept_invalid_certs(true);

        #[cfg(not(target_arch = "wasm32"))]
        let builder = builder
            .timeout(std::time::Duration::from_secs(120))
            .connect_timeout(std::time::Duration::from_secs(30))
            .gzip(true)
            .brotli(true);

        let http_client = builder.build().unwrap_or_else(|_| reqwest::Client::new());

        Self {
            playlist_url,
            http_client,
            entries: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    /// Fetch the M3U URL and parse all entries, caching them internally.
    /// Returns `(success, channel_count)`.
    async fn fetch_and_parse(&self) -> Result<usize, anyhow::Error> {
        let resp = self
            .http_client
            .get(&self.playlist_url)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch M3U playlist: {}", e))?;

        if !resp.status().is_success() {
            return Err(anyhow::anyhow!(
                "M3U playlist returned HTTP {}",
                resp.status()
            ));
        }

        let text = resp
            .text()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read M3U playlist body: {}", e))?;

        if !text.trim_start().starts_with("#EXTM3U") && !text.trim_start().starts_with("#EXTINF") {
            return Err(anyhow::anyhow!(
                "URL does not appear to be a valid M3U playlist (missing #EXTM3U header)"
            ));
        }

        // Parse in a blocking thread to avoid blocking the async runtime on large playlists
        let parsed = tokio::task::spawn_blocking(move || crate::m3u_parser::parse_m3u(&text))
            .await
            .map_err(|e| anyhow::anyhow!("M3U parse task failed: {}", e))?;

        let count = parsed.len();
        if let Ok(mut guard) = self.entries.lock() {
            *guard = parsed;
        }
        Ok(count)
    }

    pub async fn authenticate(
        &self,
    ) -> Result<(bool, Option<UserInfo>, Option<ServerInfo>), anyhow::Error> {
        let count = self.fetch_and_parse().await?;
        let live_count = {
            let guard = self
                .entries
                .lock()
                .map_err(|_| anyhow::anyhow!("M3U entries lock poisoned"))?;
            guard
                .iter()
                .filter(|e| e.tvg_type == crate::m3u_parser::M3uEntryType::Live)
                .count()
        };
        let user_info = UserInfo {
            auth: 1,
            status: Some("Active".to_string()),
            exp_date: None,
            max_connections: None,
            active_cons: None,
            total_live_streams: Some(crate::flex_id::FlexId::from_number(live_count as i64)),
            total_vod_streams: Some(crate::flex_id::FlexId::from_number(
                (count - live_count) as i64,
            )),
            total_series_streams: None,
        };
        Ok((true, Some(user_info), None))
    }

    pub fn get_live_categories(&self) -> Vec<Category> {
        let guard = match self.entries.lock() {
            Ok(g) => g,
            Err(_) => return Vec::new(),
        };
        let mut seen = std::collections::HashSet::new();
        let mut categories = Vec::new();
        for entry in guard
            .iter()
            .filter(|e| e.tvg_type == crate::m3u_parser::M3uEntryType::Live)
        {
            let group = if entry.group_title.is_empty() {
                "Uncategorized".to_string()
            } else {
                entry.group_title.clone()
            };
            if seen.insert(group.clone()) {
                categories.push(Category {
                    category_id: group.clone(),
                    category_name: group,
                    ..Default::default()
                });
            }
        }
        categories
    }

    pub fn get_live_streams(&self, category_id: &str) -> Vec<Stream> {
        let guard = match self.entries.lock() {
            Ok(g) => g,
            Err(_) => return Vec::new(),
        };
        guard
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                e.tvg_type == crate::m3u_parser::M3uEntryType::Live
                    && (category_id == "ALL" || {
                        let group = if e.group_title.is_empty() {
                            "Uncategorized"
                        } else {
                            &e.group_title
                        };
                        group == category_id
                    })
            })
            .map(|(idx, entry)| {
                let epg_id = if entry.tvg_id.is_empty() {
                    None
                } else {
                    Some(entry.tvg_id.clone())
                };
                let logo = if entry.tvg_logo.is_empty() {
                    None
                } else {
                    Some(entry.tvg_logo.clone())
                };
                let group = if entry.group_title.is_empty() {
                    "Uncategorized".to_string()
                } else {
                    entry.group_title.clone()
                };
                Stream {
                    num: None,
                    name: entry.name.clone(),
                    stream_display_name: None,
                    stream_type: "live".to_string(),
                    // Use index as a synthetic stable ID
                    stream_id: crate::flex_id::FlexId::from_number(idx as i64),
                    stream_icon: logo,
                    epg_channel_id: epg_id,
                    added: None,
                    category_id: Some(group),
                    container_extension: Some("ts".to_string()),
                    rating: None,
                    rating_5: None,
                    cached_parsed: None,
                    search_name: String::new(),
                    is_american: false,
                    is_english: false,
                    clean_name: String::new(),
                    latency_ms: None,
                    account_name: None,
                    direct_url: Some(entry.url.clone()),
                }
            })
            .collect()
    }

    pub fn get_stream_url_by_id(&self, stream_id_idx: usize) -> Option<String> {
        let guard = self.entries.lock().ok()?;
        guard.get(stream_id_idx).map(|e| e.url.clone())
    }
}

#[derive(Debug, Clone)]
pub enum IptvClient {
    Xtream(XtreamClient),
    M3u(M3uClient),
}

impl IptvClient {
    pub async fn authenticate(
        &self,
    ) -> Result<(bool, IptvClient, Option<UserInfo>, Option<ServerInfo>), anyhow::Error> {
        match self {
            IptvClient::Xtream(c) => {
                let (success, ui, si) = c.authenticate().await?;
                Ok((success, IptvClient::Xtream(c.clone()), ui, si))
            }
            IptvClient::M3u(c) => {
                let (success, ui, si) = c.authenticate().await?;
                Ok((success, IptvClient::M3u(c.clone()), ui, si))
            }
        }
    }

    pub async fn get_live_categories(&self) -> Result<Vec<Category>, anyhow::Error> {
        match self {
            IptvClient::Xtream(c) => c.get_live_categories().await,
            IptvClient::M3u(c) => Ok(c.get_live_categories()),
        }
    }

    pub async fn get_live_streams(
        &self,
        category_id: &str,
        tx: Option<tokio::sync::mpsc::Sender<crate::app::AsyncAction>>,
    ) -> Result<Vec<Stream>, anyhow::Error> {
        match self {
            IptvClient::Xtream(c) => c.get_live_streams(category_id, tx).await,
            IptvClient::M3u(c) => Ok(c.get_live_streams(category_id)),
        }
    }

    pub async fn get_vod_categories(&self) -> Result<Vec<Category>, anyhow::Error> {
        match self {
            IptvClient::Xtream(c) => c.get_vod_categories().await,
            // V1: VOD not supported for M3U
            IptvClient::M3u(_) => Ok(Vec::new()),
        }
    }

    pub async fn get_vod_streams(&self, category_id: &str) -> Result<Vec<Stream>, anyhow::Error> {
        match self {
            IptvClient::Xtream(c) => c.get_vod_streams(category_id).await,
            // V1: VOD not supported for M3U
            IptvClient::M3u(_) => Ok(Vec::new()),
        }
    }

    pub async fn get_vod_streams_all(&self) -> Result<Vec<Stream>, anyhow::Error> {
        match self {
            IptvClient::Xtream(c) => c.get_vod_streams_all().await,
            // V1: VOD not supported for M3U
            IptvClient::M3u(_) => Ok(Vec::new()),
        }
    }

    pub async fn get_series_categories(&self) -> Result<Vec<Category>, anyhow::Error> {
        match self {
            IptvClient::Xtream(c) => c.get_series_categories().await,
            // V1: Series not supported for M3U
            IptvClient::M3u(_) => Ok(Vec::new()),
        }
    }

    pub async fn get_series_all(&self) -> Result<Vec<Stream>, anyhow::Error> {
        match self {
            IptvClient::Xtream(c) => c.get_series_all().await,
            // V1: Series not supported for M3U
            IptvClient::M3u(_) => Ok(Vec::new()),
        }
    }

    pub async fn get_series_streams(
        &self,
        category_id: &str,
    ) -> Result<Vec<Stream>, anyhow::Error> {
        match self {
            IptvClient::Xtream(c) => c.get_series_streams(category_id).await,
            // V1: Series not supported for M3U
            IptvClient::M3u(_) => Ok(Vec::new()),
        }
    }

    pub async fn get_series_info(&self, series_id: &str) -> Result<SeriesInfo, anyhow::Error> {
        match self {
            IptvClient::Xtream(c) => c.get_series_info(series_id).await,
            IptvClient::M3u(_) => Err(anyhow::anyhow!(
                "Series info not supported for M3U playlists"
            )),
        }
    }

    pub async fn get_vod_info(&self, vod_id: &str) -> Result<VodInfo, anyhow::Error> {
        match self {
            IptvClient::Xtream(c) => c.get_vod_info(vod_id).await,
            IptvClient::M3u(_) => Err(anyhow::anyhow!("VOD info not supported for M3U playlists")),
        }
    }

    pub async fn get_short_epg(&self, stream_id: &str) -> Result<EpgResponse, anyhow::Error> {
        match self {
            IptvClient::Xtream(c) => c.get_short_epg(stream_id).await,
            // EPG not yet implemented for M3U — return empty
            IptvClient::M3u(_) => Ok(EpgResponse {
                epg_listings: Vec::new(),
            }),
        }
    }

    /// Returns the playback URL for a live stream.
    ///
    /// For Xtream clients, the URL is constructed from base_url + credentials + stream_id.
    /// For M3U clients, the `direct_url` stored on the stream is the authoritative URL.
    /// The `stream` parameter is used by M3U to look up the cached direct URL when available.
    pub fn get_stream_url(&self, stream_id: &str, extension: &str) -> String {
        match self {
            IptvClient::Xtream(c) => c.get_stream_url(stream_id, extension),
            IptvClient::M3u(c) => {
                // stream_id is the index into the parsed entries list
                if let Ok(idx) = stream_id.parse::<usize>() {
                    c.get_stream_url_by_id(idx).unwrap_or_default()
                } else {
                    String::new()
                }
            }
        }
    }

    pub fn get_vod_url(&self, stream_id: &str, extension: &str) -> String {
        match self {
            IptvClient::Xtream(c) => c.get_vod_url(stream_id, extension),
            IptvClient::M3u(_) => String::new(),
        }
    }

    pub fn get_series_url(&self, stream_id: &str, extension: &str) -> String {
        match self {
            IptvClient::Xtream(c) => c.get_series_url(stream_id, extension),
            IptvClient::M3u(_) => String::new(),
        }
    }

    /// Returns true if this client is an M3U playlist client.
    pub fn is_m3u(&self) -> bool {
        matches!(self, IptvClient::M3u(_))
    }
}

pub fn get_id_str(id: &FlexId) -> String {
    id.to_string_value().unwrap_or_else(|| id.to_string())
}

impl XtreamClient {
    pub fn new(base_url: String, username: String, password: String) -> Self {
        let base_url = if base_url.ends_with('/') {
            base_url[..base_url.len() - 1].to_string()
        } else {
            base_url
        };

        // Build client with User-Agent and timeouts
        // Updated to mimic Chrome to avoid provider blocking
        let builder = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .danger_accept_invalid_certs(true);

        #[cfg(not(target_arch = "wasm32"))]
        let builder = builder
            .timeout(std::time::Duration::from_secs(60))
            .connect_timeout(std::time::Duration::from_secs(30)) // Increased to 30s
            .tcp_keepalive(std::time::Duration::from_secs(60))
            .gzip(true)
            .brotli(true);

        let client = builder.build().unwrap_or_else(|_| reqwest::Client::new());

        Self {
            base_url,
            username,
            password,
            client,
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create an async client with DNS-over-HTTPS resolver
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn new_with_doh(
        base_url: String,
        username: String,
        password: String,
        dns_provider: crate::config::DnsProvider,
    ) -> Result<Self, anyhow::Error> {
        use crate::config::DnsProvider;
        use hickory_resolver::config::{NameServerConfig, Protocol, ResolverConfig, ResolverOpts};
        use hickory_resolver::AsyncResolver;
        use std::net::SocketAddr;
        use std::sync::Arc;

        let base_url = if base_url.ends_with('/') {
            base_url[..base_url.len() - 1].to_string()
        } else {
            base_url
        };

        // If System DNS, skip custom resolver
        if dns_provider == DnsProvider::System {
            let client = reqwest::Client::builder()
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
                .danger_accept_invalid_certs(true)
                .timeout(std::time::Duration::from_secs(60))
                .connect_timeout(std::time::Duration::from_secs(30))
                .tcp_keepalive(std::time::Duration::from_secs(60))
                .gzip(true)
                .brotli(true)
                .build()?;

            return Ok(Self {
                base_url,
                username,
                password,
                client,
                pending_requests: Arc::new(Mutex::new(HashMap::new())),
            });
        }

        // Configure DNS-over-HTTPS based on provider
        let mut config = ResolverConfig::new();

        let (ips, tls_name) = match dns_provider {
            DnsProvider::Quad9 => (
                vec![([9, 9, 9, 11], 443), ([149, 112, 112, 11], 443)],
                "dns11.quad9.net",
            ),
            DnsProvider::AdGuard => (
                vec![([94, 140, 14, 14], 443), ([94, 140, 15, 15], 443)],
                "dns.adguard-dns.com",
            ),
            DnsProvider::Cloudflare => (
                vec![([1, 1, 1, 1], 443), ([1, 0, 0, 1], 443)],
                "cloudflare-dns.com",
            ),
            DnsProvider::Google => (vec![([8, 8, 8, 8], 443), ([8, 8, 4, 4], 443)], "dns.google"),
            DnsProvider::System => unreachable!(), // Handled above
        };

        for (ip, port) in ips {
            let mut ns = NameServerConfig::new(SocketAddr::from((ip, port)), Protocol::Https);
            ns.tls_dns_name = Some(tls_name.to_string());
            config.add_name_server(ns);
        }

        let async_resolver = AsyncResolver::tokio(config, ResolverOpts::default());

        // Custom Bridge to implement reqwest's Resolve trait using hickory-resolver
        #[derive(Clone)]
        struct DohResolver(hickory_resolver::TokioAsyncResolver);
        impl reqwest::dns::Resolve for DohResolver {
            fn resolve(&self, name: reqwest::dns::Name) -> reqwest::dns::Resolving {
                let resolver = self.0.clone();
                let name = name.as_str().to_string();
                Box::pin(async move {
                    match resolver.lookup_ip(name).await {
                        Ok(lookup) => {
                            let addrs: Vec<SocketAddr> =
                                lookup.iter().map(|ip| SocketAddr::new(ip, 0)).collect();
                            Ok(Box::new(addrs.into_iter()) as reqwest::dns::Addrs)
                        }
                        Err(e) => Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>),
                    }
                })
            }
        }

        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .danger_accept_invalid_certs(true)
            .dns_resolver(Arc::new(DohResolver(async_resolver)))
            .timeout(std::time::Duration::from_secs(60))
            .connect_timeout(std::time::Duration::from_secs(30))
            .pool_max_idle_per_host(8) // Limit excessive connections to satisfy ISP constraints
            .tcp_keepalive(std::time::Duration::from_secs(60))
            .gzip(true)
            .brotli(true)
            .build()?;

        Ok(Self {
            base_url,
            username,
            password,
            client,
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Execute a request with automatic DNS-over-HTTPS fallback.
    /// Credentials are redacted from any error messages returned to the UI.
    async fn execute_request(
        &self,
        url: &str,
        timeout_secs: u64,
    ) -> Result<reqwest::Response, crate::errors::IptvError> {
        use crate::errors::{ConnectionStage, IptvError};

        let result = self
            .client
            .get(url)
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .send()
            .await;

        match result {
            Ok(resp) => Ok(resp),
            Err(e) => {
                // Use shared DNS error detection
                if crate::doh::is_dns_error(&e) {
                    #[cfg(debug_assertions)]
                    println!("DEBUG: DNS error detected, trying DoH fallbacks...");

                    // Try DoH fallback (skips HTTPS due to SNI mismatch)
                    if let Some(resp) = crate::doh::try_doh_fallback(&self.client, url).await {
                        return Ok(resp);
                    }

                    return Err(IptvError::DnsResolution(
                        crate::doh::redact_url(&self.base_url),
                        e.to_string(),
                    ));
                }

                let safe_url = crate::doh::redact_url(&self.base_url);
                if e.is_timeout() {
                    Err(IptvError::ConnectionTimeout(safe_url, timeout_secs))
                } else if e.is_connect() {
                    Err(IptvError::ConnectionFailed(
                        ConnectionStage::TcpConnection,
                        e.to_string(),
                    ))
                } else {
                    Err(IptvError::ConnectionFailed(
                        ConnectionStage::HttpHandshake,
                        e.to_string(),
                    ))
                }
            }
        }
    }

    pub async fn authenticate(
        &self,
    ) -> Result<(bool, Option<UserInfo>, Option<ServerInfo>), crate::errors::IptvError> {
        let url = format!(
            "{}/player_api.php?username={}&password={}",
            self.base_url, self.username, self.password
        );

        let resp = self.execute_request(&url, 60).await?;

        if !resp.status().is_success() {
            return Err(crate::errors::IptvError::ServerError(
                resp.status().as_u16(),
                resp.status()
                    .canonical_reason()
                    .unwrap_or("Unknown error")
                    .to_string(),
            ));
        }

        #[derive(Deserialize)]
        struct AuthResponse {
            user_info: Option<UserInfo>,
            server_info: Option<ServerInfo>,
        }

        let bytes = resp.bytes().await.map_err(|e| {
            crate::errors::IptvError::ConnectionFailed(
                crate::errors::ConnectionStage::ResponseParsing,
                e.to_string(),
            )
        })?;

        let bytes_for_auth = bytes.clone();
        let auth_res = tokio::task::spawn_blocking(move || {
            serde_json::from_slice::<AuthResponse>(&bytes_for_auth)
        })
        .await
        .map_err(|e| crate::errors::IptvError::ParseError(e.to_string()))?;

        match auth_res {
            Ok(json) => {
                if let Some(info) = json.user_info {
                    return Ok((info.auth == 1, Some(info), json.server_info));
                }
                Ok((false, None, None))
            }
            Err(e) => {
                // Check if it's plain text error or ISP block
                let text = String::from_utf8_lossy(&bytes).to_lowercase();
                if text.contains("invalid") || text.contains("expired") || text.contains("disabled")
                {
                    return Ok((false, None, None));
                }

                if text.contains("<!doctype html>")
                    || text.contains("<html")
                    || text.contains("at&t")
                    || text.contains("home network security")
                {
                    return Err(crate::errors::IptvError::IspBlock);
                }

                Err(crate::errors::IptvError::ParseError(format!(
                    "{} Body: {}",
                    e,
                    text.chars().take(100).collect::<String>()
                )))
            }
        }
    }

    /// Execute a request with coalescing to prevent duplicate concurrent requests.
    /// Returns None if the request was coalesced (caller should retry/get from cache),
    /// or returns Some(()) if this caller should make the request.
    async fn begin_coalesced_request(&self, url_key: &str) -> Option<Arc<Notify>> {
        let mut pending = self.pending_requests.lock().await;

        if let Some(notify) = pending.get(url_key) {
            // Another request is in progress, wait for it
            let notify = notify.clone();
            drop(pending); // Release lock before waiting
            notify.notified().await;
            return None; // Signal that caller should check cache/retry
        }

        // No pending request, register this one
        let notify = Arc::new(Notify::new());
        pending.insert(url_key.to_string(), notify.clone());
        Some(notify)
    }

    /// Complete a coalesced request, notifying any waiters
    async fn end_coalesced_request(&self, url_key: &str, notify: Arc<Notify>) {
        let mut pending = self.pending_requests.lock().await;
        pending.remove(url_key);
        drop(pending); // Release lock before notifying
        notify.notify_waiters();
    }

    pub async fn get_live_categories(&self) -> Result<Vec<Category>, anyhow::Error> {
        let url = format!(
            "{}/player_api.php?username={}&password={}&action=get_live_categories",
            self.base_url, self.username, self.password
        );
        let url_key = url.clone();

        // Check for pending request (coalescing)
        let notify = match self.begin_coalesced_request(&url_key).await {
            Some(n) => n,
            None => {
                // Request was coalesced, caller should ignore and use the original's result
                return Err(anyhow::anyhow!("Request coalesced"));
            }
        };

        let result = async {
            let resp = self
                .execute_request(&url, 60)
                .await
                .map_err(|e| anyhow::anyhow!(e))?;

            let bytes = resp
                .bytes()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to read live categories body: {}", e))?;

            if bytes.is_empty() || bytes == "{}" || bytes == "null" {
                return Ok(Vec::new());
            }

            let categories = tokio::task::spawn_blocking(move || {
                serde_json::from_slice::<Vec<Category>>(&bytes)
            })
            .await
            .map_err(|e| anyhow::anyhow!("Spawn blocking failed: {}", e))?
            .map_err(|e| anyhow::anyhow!("Failed to parse live categories JSON: {}", e))?;

            Ok(categories)
        }
        .await;

        // Notify any waiters
        self.end_coalesced_request(&url_key, notify).await;

        result
    }

    pub async fn get_live_streams(
        &self,
        category_id: &str,
        tx: Option<tokio::sync::mpsc::Sender<crate::app::AsyncAction>>,
    ) -> Result<Vec<Stream>, anyhow::Error> {
        let url = if category_id == "ALL" {
            // Fetch all streams
            format!(
                "{}/player_api.php?username={}&password={}&action=get_live_streams",
                self.base_url, self.username, self.password
            )
        } else {
            format!(
                "{}/player_api.php?username={}&password={}&action=get_live_streams&category_id={}",
                self.base_url, self.username, self.password, category_id
            )
        };
        let url_key = url.clone();

        // Check for pending request (coalescing)
        let notify = match self.begin_coalesced_request(&url_key).await {
            Some(n) => n,
            None => {
                // Request was coalesced, caller should ignore and use the original's result
                return Err(anyhow::anyhow!("Request coalesced"));
            }
        };

        // Resilient Fetching: For the "ALL" category, use a higher timeout and
        // handle potential decompression issues by disabling compression on retry if it fails.
        let is_all = category_id == "ALL";
        let timeout = if is_all { 120 } else { 60 };

        let result = async {
            let mut resp = self.execute_request(&url, timeout).await
                .map_err(|e| anyhow::anyhow!(e))?;

            let total_size = resp.content_length();

            // Stream chunks to track progress
            let mut bytes = Vec::new();
            let mut downloaded: u64 = 0;
            let mut last_percent: usize = 0;
            let mut last_progress_mb: u64 = 0; // Milestone tracker for unknown-size downloads
            let start_time = std::time::Instant::now();

            let mut fetch_error = None;

            while let Some(chunk_res) = resp.chunk().await.transpose() {
                match chunk_res {
                    Ok(chunk) => {
                        bytes.extend_from_slice(&chunk);
                        downloaded += chunk.len() as u64;

                        if let Some(total) = total_size {
                            let percent = ((downloaded as f64 / total as f64) * 100.0) as usize;
                            if percent > last_percent && percent % 2 == 0 {
                                last_percent = percent;
                                if let Some(ref sender) = tx {
                                    let bytes_mb = downloaded as f64 / 1_048_576.0;
                                    let total_mb = total as f64 / 1_048_576.0;
                                    let elapsed = start_time.elapsed().as_secs_f64();
                                    let speed_mb_s = if elapsed > 0.0 { bytes_mb / elapsed } else { 0.0 };
                                    let eta_secs = if speed_mb_s > 0.0 { (total_mb - bytes_mb) / speed_mb_s } else { 0.0 };

                                    let msg = format!("Phase 0/3: Downloading playlist... {}% (ETA {:.0}s) [{:.1}/{:.1} MB]", percent, eta_secs, bytes_mb, total_mb);
                                    let _ = sender.send(crate::app::AsyncAction::LoadingMessage(msg)).await;
                                }
                            }
                        } else {
                            // Milestone-based: fire exactly once per 5 MB boundary
                            let current_mb = downloaded / 5_242_880;
                            if current_mb > last_progress_mb {
                                last_progress_mb = current_mb;
                                if let Some(ref sender) = tx {
                                    let bytes_mb = downloaded as f64 / 1_048_576.0;
                                    let msg = format!("Phase 0/3: Downloading playlist... {:.1} MB", bytes_mb);
                                    let _ = sender.send(crate::app::AsyncAction::LoadingMessage(msg)).await;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        fetch_error = Some(e);
                        break;
                    }
                }
            }

            let bytes = if let Some(e) = fetch_error {
                if is_all {
                    let url_for_fallback = url.clone();
                    // Try decompression fallback on truncate/error
                    let builder = reqwest::Client::builder()
                        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
                        .danger_accept_invalid_certs(true)
                        .timeout(std::time::Duration::from_secs(120))
                        .gzip(false)
                        .brotli(false);

                    let no_comp_client = builder.build()
                        .map_err(|e| anyhow::anyhow!("Failed to build no-compression client: {}", e))?;

                    let mut resp = no_comp_client.get(&url_for_fallback)
                        .timeout(std::time::Duration::from_secs(timeout))
                        .send().await;

                    if let Err(e) = resp {
                        if crate::doh::is_dns_error(&e) {
                            if let Some(r) = crate::doh::try_doh_fallback(&no_comp_client, &url_for_fallback).await {
                                resp = Ok(r);
                            } else {
                                resp = Err(e);
                            }
                        } else {
                            resp = Err(e);
                        }
                    }

                    let resp = resp.map_err(|e| anyhow::anyhow!("Fallback request failed: {}", e))?;
                    resp.bytes().await.map_err(|e| anyhow::anyhow!("Failed to read raw body on retry: {}", e))?.to_vec()
                } else {
                    return Err(anyhow::anyhow!("Failed to read live streams body (category {}): {}", category_id, e));
                }
            } else {
                bytes
            };

            if bytes.is_empty() || bytes == b"{}" || bytes == b"null" {
                return Ok(Vec::new());
            }

            if let Some(ref sender) = tx {
                let bytes_mb = bytes.len() as f64 / 1_048_576.0;
                let msg = format!("Phase 1/3: Received {:.1} MB data stream. Preparing memory mapping...", bytes_mb);
                let _ = sender.send(crate::app::AsyncAction::LoadingMessage(msg)).await;
            }

            let tx_clone = tx.clone();
            let cat_id = category_id.to_string();

            let unique_streams = tokio::task::spawn_blocking(move || -> Result<Vec<Stream>, anyhow::Error> {
                if let Some(ref sender) = tx_clone {
                    let _ = sender.blocking_send(crate::app::AsyncAction::LoadingMessage(format!("Phase 2/3: Deserializing JSON structures out of RAM heap...")));
                }

                let streams = serde_json::from_slice::<Vec<Stream>>(&bytes)
                    .map_err(|e| anyhow::anyhow!("Failed to parse live streams JSON (category {}): {}", cat_id, e))?;

                if let Some(ref sender) = tx_clone {
                    let _ = sender.blocking_send(crate::app::AsyncAction::LoadingMessage(format!("Phase 3/3: Deduplicating {} structured streams...", streams.len())));
                }

                // Deduplicate streams based on ID AND name
                // Optimized for performance: avoided unnecessary lowercasing and optimized pre-allocation
                use std::collections::HashSet;
                let mut seen_ids = HashSet::with_capacity(streams.len());
                let mut seen_names = HashSet::with_capacity(streams.len());

                let filtered: Vec<Stream> = streams
                    .into_iter()
                    .filter(|s| {
                        // Efficient ID conversion
                        let id_str = get_id_str(&s.stream_id);

                        // Check ID first (fastest)
                        if seen_ids.contains(&id_str) {
                            return false;
                        }

                        // Then check Name (Exact match is usually sufficient and much faster)
                        if seen_names.contains(&s.name) {
                            return false;
                        }

                        seen_ids.insert(id_str);
                        seen_names.insert(s.name.clone());
                        true
                    })
                    .collect();

                if let Some(ref sender) = tx_clone {
                    let _ = sender.blocking_send(crate::app::AsyncAction::LoadingMessage(format!("Finalized: Extracted {} unique validated streams...", filtered.len())));
                }

                Ok(filtered)
            }).await.map_err(|e| anyhow::anyhow!("Spawn blocking failed: {}", e))??;

            Ok(unique_streams)
        }.await;

        // Notify any waiters
        self.end_coalesced_request(&url_key, notify).await;

        result
    }

    pub async fn get_vod_categories(&self) -> Result<Vec<Category>, anyhow::Error> {
        let url = format!(
            "{}/player_api.php?username={}&password={}&action=get_vod_categories",
            self.base_url, self.username, self.password
        );
        let url_key = url.clone();

        // Check for pending request (coalescing)
        let notify = match self.begin_coalesced_request(&url_key).await {
            Some(n) => n,
            None => {
                return Err(anyhow::anyhow!("Request coalesced"));
            }
        };

        let result = async {
            let resp = self
                .execute_request(&url, 60)
                .await
                .map_err(|e| anyhow::anyhow!(e))?;

            let bytes = resp
                .bytes()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to read VOD categories body: {}", e))?;

            if bytes.is_empty() || bytes == "{}" || bytes == "null" {
                return Ok(Vec::new());
            }

            let categories = tokio::task::spawn_blocking(move || {
                serde_json::from_slice::<Vec<Category>>(&bytes)
            })
            .await
            .map_err(|e| anyhow::anyhow!("Spawn blocking failed: {}", e))?
            .map_err(|e| anyhow::anyhow!("Failed to parse VOD categories JSON: {}", e))?;

            Ok(categories)
        }
        .await;

        // Notify any waiters
        self.end_coalesced_request(&url_key, notify).await;

        result
    }

    pub async fn get_vod_streams(&self, category_id: &str) -> Result<Vec<Stream>, anyhow::Error> {
        let url = format!(
            "{}/player_api.php?username={}&password={}&action=get_vod_streams&category_id={}",
            self.base_url, self.username, self.password, category_id
        );
        let url_key = url.clone();

        // Check for pending request (coalescing)
        let notify = match self.begin_coalesced_request(&url_key).await {
            Some(n) => n,
            None => {
                return Err(anyhow::anyhow!("Request coalesced"));
            }
        };

        let result = async {
            let resp = self
                .execute_request(&url, 90)
                .await
                .map_err(|e| anyhow::anyhow!(e))?;

            let bytes = resp.bytes().await.map_err(|e| {
                anyhow::anyhow!(
                    "Failed to read VOD streams body (category {}): {}",
                    category_id,
                    e
                )
            })?;

            if bytes.is_empty() || bytes == "{}" || bytes == "null" {
                return Ok(Vec::new());
            }

            let streams =
                tokio::task::spawn_blocking(move || serde_json::from_slice::<Vec<Stream>>(&bytes))
                    .await
                    .map_err(|e| anyhow::anyhow!("Spawn blocking failed: {}", e))?
                    .map_err(|e| {
                        anyhow::anyhow!(
                            "Failed to parse VOD streams JSON (category {}): {}",
                            category_id,
                            e
                        )
                    })?;

            Ok(streams)
        }
        .await;

        // Notify any waiters
        self.end_coalesced_request(&url_key, notify).await;

        result
    }

    pub async fn get_vod_streams_all(&self) -> Result<Vec<Stream>, anyhow::Error> {
        let url = format!(
            "{}/player_api.php?username={}&password={}&action=get_vod_streams",
            self.base_url, self.username, self.password
        );
        let url_key = url.clone();

        // Check for pending request (coalescing)
        let notify = match self.begin_coalesced_request(&url_key).await {
            Some(n) => n,
            None => {
                return Err(anyhow::anyhow!("Request coalesced"));
            }
        };

        let result = async {
            let resp = self.client.get(&url).send().await.map_err(|e| {
                let mut msg = format!("Failed to fetch all VOD streams: {}", e);
                if e.is_connect() {
                    msg = format!("Connection failed: {}", e);
                }
                if e.is_timeout() {
                    msg = format!("Request timed out: {}", e);
                }
                if e.is_request() && e.to_string().contains("dns") {
                    msg = format!("DNS Resolution Error: {}", e);
                }
                anyhow::anyhow!(msg)
            })?;

            let bytes = resp
                .bytes()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to read all VOD streams body: {}", e))?;

            if bytes.is_empty() || bytes == "{}" || bytes == "null" {
                return Ok(Vec::new());
            }

            let streams =
                tokio::task::spawn_blocking(move || serde_json::from_slice::<Vec<Stream>>(&bytes))
                    .await
                    .map_err(|e| anyhow::anyhow!("Spawn blocking failed: {}", e))?
                    .map_err(|e| anyhow::anyhow!("Failed to parse all VOD streams JSON: {}", e))?;

            Ok(streams)
        }
        .await;

        // Notify any waiters
        self.end_coalesced_request(&url_key, notify).await;

        result
    }

    pub async fn get_series_categories(&self) -> Result<Vec<Category>, anyhow::Error> {
        let url = format!(
            "{}/player_api.php?username={}&password={}&action=get_series_categories",
            self.base_url, self.username, self.password
        );
        let url_key = url.clone();

        // Check for pending request (coalescing)
        let notify = match self.begin_coalesced_request(&url_key).await {
            Some(n) => n,
            None => {
                return Err(anyhow::anyhow!("Request coalesced"));
            }
        };

        let result = async {
            let resp = self.client.get(&url).send().await.map_err(|e| {
                let mut msg = format!("Failed to fetch series categories: {}", e);
                if e.is_connect() {
                    msg = format!("Connection failed: {}", e);
                }
                if e.is_timeout() {
                    msg = format!("Request timed out: {}", e);
                }
                if e.is_request() && e.to_string().contains("dns") {
                    msg = format!("DNS Resolution Error: {}", e);
                }
                anyhow::anyhow!(msg)
            })?;
            let bytes = resp
                .bytes()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to read series categories body: {}", e))?;

            if bytes.is_empty() || bytes == "{}" || bytes == "null" {
                return Ok(Vec::new());
            }

            let categories = tokio::task::spawn_blocking(move || {
                serde_json::from_slice::<Vec<Category>>(&bytes)
            })
            .await
            .map_err(|e| anyhow::anyhow!("Spawn blocking failed: {}", e))?
            .map_err(|e| anyhow::anyhow!("Failed to parse series categories JSON: {}", e))?;

            Ok(categories)
        }
        .await;

        // Notify any waiters
        self.end_coalesced_request(&url_key, notify).await;

        result
    }

    pub async fn get_series_all(&self) -> Result<Vec<Stream>, anyhow::Error> {
        let url = format!(
            "{}/player_api.php?username={}&password={}&action=get_series",
            self.base_url, self.username, self.password
        );
        let url_key = url.clone();

        // Check for pending request (coalescing)
        let notify = match self.begin_coalesced_request(&url_key).await {
            Some(n) => n,
            None => {
                return Err(anyhow::anyhow!("Request coalesced"));
            }
        };

        let result = async {
            let resp = self
                .execute_request(&url, 90)
                .await
                .map_err(|e| anyhow::anyhow!(e))?;
            let bytes = resp
                .bytes()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to read all series body: {}", e))?;

            // Providers sometimes return {} when there are no series instead of []
            if bytes.is_empty() || bytes == "{}" || bytes == "null" {
                return Ok(Vec::new());
            }

            let series =
                tokio::task::spawn_blocking(move || serde_json::from_slice::<Vec<Stream>>(&bytes))
                    .await
                    .map_err(|e| anyhow::anyhow!("Spawn blocking failed: {}", e))?
                    .map_err(|e| anyhow::anyhow!("Failed to parse all series JSON: {}", e))?;

            Ok(series)
        }
        .await;

        // Notify any waiters
        self.end_coalesced_request(&url_key, notify).await;

        result
    }

    pub async fn get_series_streams(
        &self,
        category_id: &str,
    ) -> Result<Vec<Stream>, anyhow::Error> {
        let url = format!(
            "{}/player_api.php?username={}&password={}&action=get_series&category_id={}",
            self.base_url, self.username, self.password, category_id
        );
        let resp = self
            .execute_request(&url, 60)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

        let bytes = resp.bytes().await.map_err(|e| {
            anyhow::anyhow!(
                "Failed to read series streams body (category {}): {}",
                category_id,
                e
            )
        })?;

        if bytes.is_empty() || bytes == "{}" || bytes == "null" {
            return Ok(Vec::new());
        }

        let streams =
            tokio::task::spawn_blocking(move || serde_json::from_slice::<Vec<Stream>>(&bytes))
                .await
                .map_err(|e| anyhow::anyhow!("Spawn blocking failed: {}", e))?
                .map_err(|e| {
                    anyhow::anyhow!(
                        "Failed to parse series streams JSON (category {}): {}",
                        category_id,
                        e
                    )
                })?;

        Ok(streams)
    }

    pub async fn get_series_info(&self, series_id: &str) -> Result<SeriesInfo, anyhow::Error> {
        let url = format!(
            "{}/player_api.php?username={}&password={}&action=get_series_info&series_id={}",
            self.base_url, self.username, self.password, series_id
        );
        let resp = self
            .execute_request(&url, 60)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        let info: SeriesInfo = resp.json().await.map_err(|e| {
            anyhow::anyhow!(
                "Failed to parse series info JSON (series {}): {}",
                series_id,
                e
            )
        })?;
        Ok(info)
    }

    pub fn get_stream_url(&self, stream_id: &str, extension: &str) -> String {
        format!(
            "{}/live/{}/{}/{}.{}",
            self.base_url, self.username, self.password, stream_id, extension
        )
    }

    pub fn get_vod_url(&self, stream_id: &str, extension: &str) -> String {
        format!(
            "{}/movie/{}/{}/{}.{}",
            self.base_url, self.username, self.password, stream_id, extension
        )
    }

    pub fn get_series_url(&self, stream_id: &str, extension: &str) -> String {
        format!(
            "{}/series/{}/{}/{}.{}",
            self.base_url, self.username, self.password, stream_id, extension
        )
    }

    pub async fn get_vod_info(&self, vod_id: &str) -> Result<VodInfo, anyhow::Error> {
        let url = format!(
            "{}/player_api.php?username={}&password={}&action=get_vod_info&vod_id={}",
            self.base_url, self.username, self.password, vod_id
        );
        let resp = self
            .execute_request(&url, 60)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

        let bytes = resp
            .bytes()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read VOD info body (VOD {}): {}", vod_id, e))?;

        if bytes.is_empty() || bytes == "{}" || bytes == "null" {
            return Ok(VodInfo::default());
        }

        let info = tokio::task::spawn_blocking(move || serde_json::from_slice::<VodInfo>(&bytes))
            .await
            .map_err(|e| anyhow::anyhow!("Spawn blocking failed: {}", e))?
            .map_err(|e| {
                anyhow::anyhow!("Failed to parse VOD info JSON (VOD {}): {}", vod_id, e)
            })?;

        Ok(info)
    }

    pub async fn get_short_epg(&self, stream_id: &str) -> Result<EpgResponse, anyhow::Error> {
        let url = format!(
            "{}/player_api.php?username={}&password={}&action=get_short_epg&stream_id={}",
            self.base_url, self.username, self.password, stream_id
        );
        let resp = self
            .execute_request(&url, 30)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        let mut epg: EpgResponse = resp.json().await.map_err(|e| {
            anyhow::anyhow!("Failed to parse EPG JSON (stream {}): {}", stream_id, e)
        })?;

        use base64::{engine::general_purpose, Engine as _};
        for listing in &mut epg.epg_listings {
            if let Ok(decoded) = general_purpose::STANDARD.decode(&listing.title) {
                if let Ok(decoded_str) = String::from_utf8(decoded) {
                    listing.title = decoded_str;
                }
            }
            if let Some(desc) = &listing.description {
                if let Ok(decoded) = general_purpose::STANDARD.decode(desc) {
                    if let Ok(decoded_str) = String::from_utf8(decoded) {
                        listing.description = Some(decoded_str);
                    }
                }
            }
        }

        Ok(epg)
    }
}
