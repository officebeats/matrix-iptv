use serde::{Deserialize, Serialize};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};
use crate::flex_id::{FlexId, deserialize_flex_option_f32};


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
}

impl Stream {
    /// Get or parse stream metadata with caching
    pub fn get_or_parse_cached(&mut self, provider_tz: Option<&str>) -> &crate::parser::ParsedStream {
        if self.cached_parsed.is_none() {
            self.cached_parsed = Some(Box::new(crate::parser::parse_stream(&self.name, provider_tz)));
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

#[derive(Debug, Clone)]
pub enum IptvClient {
    Xtream(XtreamClient),
}

impl IptvClient {
    pub async fn authenticate(&self) -> Result<(bool, IptvClient, Option<UserInfo>, Option<ServerInfo>), anyhow::Error> {
        match self {
            IptvClient::Xtream(c) => {
                let (success, ui, si) = c.authenticate().await?;
                Ok((success, IptvClient::Xtream(c.clone()), ui, si))
            }
        }
    }

    pub async fn get_live_categories(&self) -> Result<Vec<Category>, anyhow::Error> {
        match self {
            IptvClient::Xtream(c) => c.get_live_categories().await,
        }
    }

    pub async fn get_live_streams(&self, category_id: &str) -> Result<Vec<Stream>, anyhow::Error> {
        match self {
            IptvClient::Xtream(c) => c.get_live_streams(category_id).await,
        }
    }

    pub async fn get_vod_categories(&self) -> Result<Vec<Category>, anyhow::Error> {
        match self {
            IptvClient::Xtream(c) => c.get_vod_categories().await,
        }
    }

    pub async fn get_vod_streams(&self, category_id: &str) -> Result<Vec<Stream>, anyhow::Error> {
        match self {
            IptvClient::Xtream(c) => c.get_vod_streams(category_id).await,
        }
    }

    pub async fn get_vod_streams_all(&self) -> Result<Vec<Stream>, anyhow::Error> {
        match self {
            IptvClient::Xtream(c) => c.get_vod_streams_all().await,
        }
    }

    pub async fn get_series_categories(&self) -> Result<Vec<Category>, anyhow::Error> {
        match self {
            IptvClient::Xtream(c) => c.get_series_categories().await,
        }
    }

    pub async fn get_series_all(&self) -> Result<Vec<Stream>, anyhow::Error> {
        match self {
            IptvClient::Xtream(c) => c.get_series_all().await,
        }
    }

    pub async fn get_series_streams(&self, category_id: &str) -> Result<Vec<Stream>, anyhow::Error> {
        match self {
            IptvClient::Xtream(c) => c.get_series_streams(category_id).await,
        }
    }

    pub async fn get_series_info(&self, series_id: &str) -> Result<SeriesInfo, anyhow::Error> {
        match self {
            IptvClient::Xtream(c) => c.get_series_info(series_id).await,
        }
    }

    pub async fn get_vod_info(&self, vod_id: &str) -> Result<VodInfo, anyhow::Error> {
        match self {
            IptvClient::Xtream(c) => c.get_vod_info(vod_id).await,
        }
    }

    pub async fn get_short_epg(&self, stream_id: &str) -> Result<EpgResponse, anyhow::Error> {
        match self {
            IptvClient::Xtream(c) => c.get_short_epg(stream_id).await,
        }
    }

    pub fn get_stream_url(&self, stream_id: &str, extension: &str) -> String {
        match self {
            IptvClient::Xtream(c) => c.get_stream_url(stream_id, extension),
        }
    }

    pub fn get_vod_url(&self, stream_id: &str, extension: &str) -> String {
        match self {
            IptvClient::Xtream(c) => c.get_vod_url(stream_id, extension),
        }
    }

    pub fn get_series_url(&self, stream_id: &str, extension: &str) -> String {
        match self {
            IptvClient::Xtream(c) => c.get_series_url(stream_id, extension),
        }
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
        use hickory_resolver::AsyncResolver;
        use hickory_resolver::config::{ResolverConfig, ResolverOpts, NameServerConfig, Protocol};
        use std::sync::Arc;
        use std::net::SocketAddr;
        use crate::config::DnsProvider;

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
            DnsProvider::Google => (
                vec![([8, 8, 8, 8], 443), ([8, 8, 4, 4], 443)],
                "dns.google",
            ),
            DnsProvider::System => unreachable!(), // Handled above
        };

        for (ip, port) in ips {
            let mut ns = NameServerConfig::new(
                SocketAddr::from((ip, port)),
                Protocol::Https,
            );
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
                            let addrs: Vec<SocketAddr> = lookup.iter()
                                .map(|ip| SocketAddr::new(ip, 0))
                                .collect();
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

    pub async fn authenticate(
        &self,
    ) -> Result<(bool, Option<UserInfo>, Option<ServerInfo>), crate::errors::IptvError> {
        let url = format!(
            "{}/player_api.php?username={}&password={}",
            self.base_url, self.username, self.password
        );
        
        let resp = self.client.get(&url).send().await.map_err(|e| {
            use crate::errors::ConnectionStage;
            let err_str = e.to_string().to_lowercase();
            
            // Prioritize DNS detection as it's the most common failure point with custom resolvers
            if err_str.contains("dns") || err_str.contains("resolution") || err_str.contains("resolve") {
                crate::errors::IptvError::DnsResolution(self.base_url.clone(), e.to_string())
            } else if e.is_timeout() {
                crate::errors::IptvError::ConnectionTimeout(self.base_url.clone(), 60)
            } else if e.is_connect() {
                crate::errors::IptvError::ConnectionFailed(ConnectionStage::TcpConnection, e.to_string())
            } else {
                crate::errors::IptvError::ConnectionFailed(ConnectionStage::HttpHandshake, e.to_string())
            }
        })?;

        if !resp.status().is_success() {
            return Err(crate::errors::IptvError::ServerError(resp.status().as_u16(), resp.status().canonical_reason().unwrap_or("Unknown error").to_string()));
        }

        #[derive(Deserialize)]
        struct AuthResponse {
            user_info: Option<UserInfo>,
            server_info: Option<ServerInfo>,
        }

        let bytes = resp.bytes().await
            .map_err(|e| crate::errors::IptvError::ConnectionFailed(crate::errors::ConnectionStage::ResponseParsing, e.to_string()))?;

        let bytes_for_auth = bytes.clone();
        let auth_res = tokio::task::spawn_blocking(move || {
            serde_json::from_slice::<AuthResponse>(&bytes_for_auth)
        }).await.map_err(|e| crate::errors::IptvError::ParseError(e.to_string()))?;

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
                if text.contains("invalid") || text.contains("expired") || text.contains("disabled") {
                    return Ok((false, None, None));
                }
                
                if text.contains("<!doctype html>") || text.contains("<html") || text.contains("at&t") || text.contains("home network security") {
                     return Err(crate::errors::IptvError::IspBlock);
                }

                Err(crate::errors::IptvError::ParseError(format!("{} Body: {}", e, text.chars().take(100).collect::<String>())))
            }
        }
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
                // Request was coalesced, return empty (cache should be checked by caller)
                return Ok(Vec::new());
            }
        };
        
        let result = async {
            let resp = self.client.get(&url).send().await
                .map_err(|e| {
                    let mut msg = format!("Failed to fetch live categories: {}", e);
                    if e.is_connect() { msg = format!("Connection failed: {}", e); }
                    if e.is_timeout() { msg = format!("Request timed out: {}", e); }
                    if e.is_request() && e.to_string().contains("dns") { msg = format!("DNS Resolution Error: {}", e); }
                    anyhow::anyhow!(msg)
                })?;
            let bytes = resp.bytes().await
                .map_err(|e| anyhow::anyhow!("Failed to read live categories body: {}", e))?;
            
            if bytes.is_empty() || bytes == "{}" || bytes == "null" {
                return Ok(Vec::new());
            }

            let categories = tokio::task::spawn_blocking(move || {
                serde_json::from_slice::<Vec<Category>>(&bytes)
            }).await.map_err(|e| anyhow::anyhow!("Spawn blocking failed: {}", e))?
              .map_err(|e| anyhow::anyhow!("Failed to parse live categories JSON: {}", e))?;
              
            Ok(categories)
        }.await;
        
        // Notify any waiters
        self.end_coalesced_request(&url_key, notify).await;
        
        result
    }

    pub async fn get_live_streams(&self, category_id: &str) -> Result<Vec<Stream>, anyhow::Error> {
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
                // Request was coalesced, return empty (cache should be checked by caller)
                return Ok(Vec::new());
            }
        };

        // Resilient Fetching: For the "ALL" category, use a higher timeout and
        // handle potential decompression issues by disabling compression on retry if it fails.
        let is_all = category_id == "ALL";
        let timeout = if is_all { 120 } else { 60 };
        
        let result = async {
            let resp = self.client.get(&url)
                .timeout(std::time::Duration::from_secs(timeout))
                .send().await
                .map_err(|e| {
                    let mut msg = format!("Failed to fetch live streams (category {}): {}", category_id, e);
                    if e.is_connect() { msg = format!("Connection failed: {}", e); }
                    if e.is_timeout() { msg = format!("Request timed out ({}s): {}", timeout, e); }
                    if e.is_request() && e.to_string().contains("dns") { msg = format!("DNS Resolution Error: {}", e); }
                    anyhow::anyhow!(msg)
                })?;
            
            let bytes = match resp.bytes().await {
                Ok(b) => b,
                Err(_e) if is_all => {
                    // If decoding compressed body fails for ALL, OR if connection was reset/truncated,
                    // retry with no compression and a fresh client instance.
                    let builder = reqwest::Client::builder()
                        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
                        .danger_accept_invalid_certs(true)
                        .timeout(std::time::Duration::from_secs(120))
                        .gzip(false)
                        .brotli(false);

                    let no_comp_client = builder.build().unwrap_or_else(|_| self.client.clone());
                    let resp = no_comp_client.get(&url)
                        .header("Accept-Encoding", "identity")
                        .send().await
                        .map_err(|e| anyhow::anyhow!("Decompression/Truncation retry failed: {}", e))?;
                    resp.bytes().await.map_err(|e| anyhow::anyhow!("Failed to read raw body on retry: {}", e))?
                }
                Err(e) => return Err(anyhow::anyhow!("Failed to read live streams body (category {}): {}", category_id, e)),
            };

            if bytes.is_empty() || bytes == "{}" || bytes == "null" {
                return Ok(Vec::new());
            }

            let streams = tokio::task::spawn_blocking(move || {
                serde_json::from_slice::<Vec<Stream>>(&bytes)
            }).await.map_err(|e| anyhow::anyhow!("Spawn blocking failed: {}", e))?
              .map_err(|e| anyhow::anyhow!("Failed to parse live streams JSON (category {}): {}", category_id, e))?;
            
            // Deduplicate streams based on ID AND name
            // Optimized for performance: avoided unnecessary lowercasing and optimized pre-allocation
            use std::collections::HashSet;
            let mut seen_ids = HashSet::with_capacity(streams.len());
            let mut seen_names = HashSet::with_capacity(streams.len());
            
            let unique_streams: Vec<Stream> = streams
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
                // Request was coalesced, return empty (cache should be checked by caller)
                return Ok(Vec::new());
            }
        };
        
        let result = async {
            let resp = self.client.get(&url).send().await
                .map_err(|e| {
                    let mut msg = format!("Failed to fetch VOD categories: {}", e);
                    if e.is_connect() { msg = format!("Connection failed: {}", e); }
                    if e.is_timeout() { msg = format!("Request timed out: {}", e); }
                    if e.is_request() && e.to_string().contains("dns") { msg = format!("DNS Resolution Error: {}", e); }
                    anyhow::anyhow!(msg)
                })?;
            let bytes = resp.bytes().await
                .map_err(|e| anyhow::anyhow!("Failed to read VOD categories body: {}", e))?;

            if bytes.is_empty() || bytes == "{}" || bytes == "null" {
                return Ok(Vec::new());
            }

            let categories = tokio::task::spawn_blocking(move || {
                serde_json::from_slice::<Vec<Category>>(&bytes)
            }).await.map_err(|e| anyhow::anyhow!("Spawn blocking failed: {}", e))?
              .map_err(|e| anyhow::anyhow!("Failed to parse VOD categories JSON: {}", e))?;

            Ok(categories)
        }.await;
        
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
                // Request was coalesced, return empty (cache should be checked by caller)
                return Ok(Vec::new());
            }
        };
        
        // Implementation with retry logic for VOD
        let mut retry_count = 0;
        let max_retries = 3;
        
        let result = loop {
            let client = if retry_count > 0 {
                // Disable compression on retry to handle truncation issues
                reqwest::Client::builder()
                    .timeout(std::time::Duration::from_secs(90))
                    .gzip(false)
                    .brotli(false)
                    .build().unwrap_or(self.client.clone())
            } else {
                self.client.clone()
            };

            let resp = match client.get(&url).send().await {
                Ok(r) => r,
                Err(e) => {
                    let mut msg = format!("Failed to fetch VOD streams (category {}): {}", category_id, e);
                    if e.is_connect() { msg = format!("Connection failed: {}", e); }
                    if e.is_timeout() { msg = format!("Request timed out: {}", e); }
                    break Err(anyhow::anyhow!(msg));
                }
            };
            
            let bytes = match resp.bytes().await {
                Ok(b) => b,
                Err(e) => {
                    if retry_count < max_retries {
                        retry_count += 1;
                        continue;
                    }
                    break Err(anyhow::anyhow!("Failed to read VOD streams body (category {}): {}", category_id, e));
                }
            };

            if bytes.is_empty() || bytes == "{}" || bytes == "null" {
                break Ok(Vec::new());
            }

            let streams_res = tokio::task::spawn_blocking(move || {
                serde_json::from_slice::<Vec<Stream>>(&bytes)
            }).await;

            match streams_res {
                Ok(Ok(streams)) => break Ok(streams),
                _ => {
                    if retry_count < max_retries {
                        retry_count += 1;
                        continue;
                    }
                    break Err(anyhow::anyhow!("Failed to parse VOD streams JSON after {} retries", retry_count));
                }
            }
        };
        
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
                // Request was coalesced, return empty (cache should be checked by caller)
                return Ok(Vec::new());
            }
        };
        
        let result = async {
            let resp = self.client.get(&url).send().await
                .map_err(|e| {
                    let mut msg = format!("Failed to fetch all VOD streams: {}", e);
                    if e.is_connect() { msg = format!("Connection failed: {}", e); }
                    if e.is_timeout() { msg = format!("Request timed out: {}", e); }
                    if e.is_request() && e.to_string().contains("dns") { msg = format!("DNS Resolution Error: {}", e); }
                    anyhow::anyhow!(msg)
                })?;
            
            let bytes = resp.bytes().await
                .map_err(|e| anyhow::anyhow!("Failed to read all VOD streams body: {}", e))?;

            if bytes.is_empty() || bytes == "{}" || bytes == "null" {
                return Ok(Vec::new());
            }

            let streams = tokio::task::spawn_blocking(move || {
                serde_json::from_slice::<Vec<Stream>>(&bytes)
            }).await.map_err(|e| anyhow::anyhow!("Spawn blocking failed: {}", e))?
              .map_err(|e| anyhow::anyhow!("Failed to parse all VOD streams JSON: {}", e))?;

            Ok(streams)
        }.await;
        
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
                // Request was coalesced, return empty (cache should be checked by caller)
                return Ok(Vec::new());
            }
        };
        
        let result = async {
            let resp = self.client.get(&url).send().await
                .map_err(|e| {
                    let mut msg = format!("Failed to fetch series categories: {}", e);
                    if e.is_connect() { msg = format!("Connection failed: {}", e); }
                    if e.is_timeout() { msg = format!("Request timed out: {}", e); }
                    if e.is_request() && e.to_string().contains("dns") { msg = format!("DNS Resolution Error: {}", e); }
                    anyhow::anyhow!(msg)
                })?;
            let bytes = resp.bytes().await
                .map_err(|e| anyhow::anyhow!("Failed to read series categories body: {}", e))?;

            if bytes.is_empty() || bytes == "{}" || bytes == "null" {
                return Ok(Vec::new());
            }

            let categories = tokio::task::spawn_blocking(move || {
                serde_json::from_slice::<Vec<Category>>(&bytes)
            }).await.map_err(|e| anyhow::anyhow!("Spawn blocking failed: {}", e))?
              .map_err(|e| anyhow::anyhow!("Failed to parse series categories JSON: {}", e))?;

            Ok(categories)
        }.await;
        
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
                // Request was coalesced, return empty (cache should be checked by caller)
                return Ok(Vec::new());
            }
        };
        
        let result = async {
            let resp = self.client.get(&url).send().await
                .map_err(|e| anyhow::anyhow!("Failed to fetch all series: {}", e))?;
            let bytes = resp.bytes().await
                .map_err(|e| anyhow::anyhow!("Failed to read all series body: {}", e))?;
            
            // Providers sometimes return {} when there are no series instead of []
            if bytes.is_empty() || bytes == "{}" || bytes == "null" {
                return Ok(Vec::new());
            }

            let series = tokio::task::spawn_blocking(move || {
                serde_json::from_slice::<Vec<Stream>>(&bytes)
            }).await.map_err(|e| anyhow::anyhow!("Spawn blocking failed: {}", e))?
              .map_err(|e| anyhow::anyhow!("Failed to parse all series JSON: {}", e))?;

            Ok(series)
        }.await;
        
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
        let resp = self.client.get(&url).send().await
            .map_err(|e| anyhow::anyhow!("Failed to fetch series streams (category {}): {}", category_id, e))?;
        
        let bytes = resp.bytes().await
            .map_err(|e| anyhow::anyhow!("Failed to read series streams body (category {}): {}", category_id, e))?;

        if bytes.is_empty() || bytes == "{}" || bytes == "null" {
            return Ok(Vec::new());
        }

        let streams = tokio::task::spawn_blocking(move || {
            serde_json::from_slice::<Vec<Stream>>(&bytes)
        }).await.map_err(|e| anyhow::anyhow!("Spawn blocking failed: {}", e))?
          .map_err(|e| anyhow::anyhow!("Failed to parse series streams JSON (category {}): {}", category_id, e))?;
        
        Ok(streams)
    }

    pub async fn get_series_info(&self, series_id: &str) -> Result<SeriesInfo, anyhow::Error> {
        let url = format!(
            "{}/player_api.php?username={}&password={}&action=get_series_info&series_id={}",
            self.base_url, self.username, self.password, series_id
        );
        let resp = self.client.get(&url).send().await
            .map_err(|e| anyhow::anyhow!("Failed to fetch series info (series {}): {}", series_id, e))?;
        let info: SeriesInfo = resp.json().await
            .map_err(|e| anyhow::anyhow!("Failed to parse series info JSON (series {}): {}", series_id, e))?;
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
        let resp = self.client.get(&url).send().await
            .map_err(|e| anyhow::anyhow!("Failed to fetch VOD info (VOD {}): {}", vod_id, e))?;
        
        let bytes = resp.bytes().await
            .map_err(|e| anyhow::anyhow!("Failed to read VOD info body (VOD {}): {}", vod_id, e))?;

        if bytes.is_empty() || bytes == "{}" || bytes == "null" {
            return Ok(VodInfo::default());
        }

        let info = tokio::task::spawn_blocking(move || {
            serde_json::from_slice::<VodInfo>(&bytes)
        }).await.map_err(|e| anyhow::anyhow!("Spawn blocking failed: {}", e))?
          .map_err(|e| anyhow::anyhow!("Failed to parse VOD info JSON (VOD {}): {}", vod_id, e))?;

        Ok(info)
    }

    pub async fn get_short_epg(&self, stream_id: &str) -> Result<EpgResponse, anyhow::Error> {
        let url = format!(
            "{}/player_api.php?username={}&password={}&action=get_short_epg&stream_id={}",
            self.base_url, self.username, self.password, stream_id
        );
        let resp = self.client.get(&url).send().await
            .map_err(|e| anyhow::anyhow!("Failed to fetch short EPG (stream {}): {}", stream_id, e))?;
        let epg: EpgResponse = resp.json().await
            .map_err(|e| anyhow::anyhow!("Failed to parse EPG JSON (stream {}): {}", stream_id, e))?;
        Ok(epg)
    }
}
