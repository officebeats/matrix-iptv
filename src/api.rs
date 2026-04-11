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
    pub upper_clean_name: String,
    #[serde(skip)]
    pub is_sports: bool,
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
    #[serde(
        alias = "rating_5based",
        default,
        deserialize_with = "deserialize_flex_option_f32"
    )]
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

#[cfg(test)]
mod tests {
    use super::Stream;

    #[test]
    fn test_stream_deserialize_tolerates_invalid_rating_and_maps_rating_5based() {
        let stream: Stream = serde_json::from_str(
            r#"{
                "name":"EN| The Irish Mob",
                "stream_type":"movie",
                "stream_id":356189,
                "rating":"Meet big-time crime boss Val Fagan",
                "rating_5based":0
            }"#,
        )
        .unwrap();

        assert_eq!(stream.rating, None);
        assert_eq!(stream.rating_5, Some(0.0));
    }
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
            IptvClient::M3u(c) => c.get_live_categories().await,
        }
    }

    pub async fn get_live_streams(
        &self,
        category_id: &str,
        tx: Option<tokio::sync::mpsc::Sender<crate::app::AsyncAction>>,
    ) -> Result<Vec<Stream>, anyhow::Error> {
        match self {
            IptvClient::Xtream(c) => c.get_live_streams(category_id, tx).await,
            IptvClient::M3u(c) => c.get_live_streams(category_id, tx).await,
        }
    }

    pub async fn get_vod_categories(&self) -> Result<Vec<Category>, anyhow::Error> {
        match self {
            IptvClient::Xtream(c) => c.get_vod_categories().await,
            IptvClient::M3u(_) => Ok(Vec::new()), // M3U playlists don't have VOD categories
        }
    }

    pub async fn get_vod_streams(&self, category_id: &str) -> Result<Vec<Stream>, anyhow::Error> {
        match self {
            IptvClient::Xtream(c) => c.get_vod_streams(category_id).await,
            IptvClient::M3u(_) => Ok(Vec::new()),
        }
    }

    pub async fn get_vod_streams_all(&self) -> Result<Vec<Stream>, anyhow::Error> {
        match self {
            IptvClient::Xtream(c) => c.get_vod_streams_all().await,
            IptvClient::M3u(_) => Ok(Vec::new()),
        }
    }

    pub async fn get_series_categories(&self) -> Result<Vec<Category>, anyhow::Error> {
        match self {
            IptvClient::Xtream(c) => c.get_series_categories().await,
            IptvClient::M3u(_) => Ok(Vec::new()),
        }
    }

    pub async fn get_series_all(&self) -> Result<Vec<Stream>, anyhow::Error> {
        match self {
            IptvClient::Xtream(c) => c.get_series_all().await,
            IptvClient::M3u(_) => Ok(Vec::new()),
        }
    }

    pub async fn get_series_streams(
        &self,
        category_id: &str,
    ) -> Result<Vec<Stream>, anyhow::Error> {
        match self {
            IptvClient::Xtream(c) => c.get_series_streams(category_id).await,
            IptvClient::M3u(_) => Ok(Vec::new()),
        }
    }

    pub async fn get_series_info(&self, series_id: &str) -> Result<SeriesInfo, anyhow::Error> {
        match self {
            IptvClient::Xtream(c) => c.get_series_info(series_id).await,
            IptvClient::M3u(_) => Err(anyhow::anyhow!("Series info not available for M3U playlists")),
        }
    }

    pub async fn get_vod_info(&self, vod_id: &str) -> Result<VodInfo, anyhow::Error> {
        match self {
            IptvClient::Xtream(c) => c.get_vod_info(vod_id).await,
            IptvClient::M3u(_) => Ok(VodInfo::default()),
        }
    }

    pub async fn get_short_epg(&self, stream_id: &str) -> Result<EpgResponse, anyhow::Error> {
        match self {
            IptvClient::Xtream(c) => c.get_short_epg(stream_id).await,
            IptvClient::M3u(_) => Ok(EpgResponse { epg_listings: Vec::new() }),
        }
    }

    pub fn get_stream_url(&self, stream_id: &str, extension: &str) -> String {
        match self {
            IptvClient::Xtream(c) => c.get_stream_url(stream_id, extension),
            IptvClient::M3u(c) => c.get_stream_url(stream_id),
        }
    }

    pub fn get_vod_url(&self, stream_id: &str, extension: &str) -> String {
        match self {
            IptvClient::Xtream(c) => c.get_vod_url(stream_id, extension),
            IptvClient::M3u(c) => c.get_stream_url(stream_id),
        }
    }

    pub fn get_series_url(&self, stream_id: &str, extension: &str) -> String {
        match self {
            IptvClient::Xtream(c) => c.get_series_url(stream_id, extension),
            IptvClient::M3u(c) => c.get_stream_url(stream_id),
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

                                    let msg = format!(
                                        "Phase 1/4: Downloading playlist payload... {}% (ETA {:.0}s) [{:.1}/{:.1} MB]",
                                        percent, eta_secs, bytes_mb, total_mb
                                    );
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
                                    let msg = format!("Phase 1/4: Downloading playlist payload... {:.1} MB", bytes_mb);
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
                let msg = format!("Phase 2/4: Received {:.1} MB. Staging provider data in memory...", bytes_mb);
                let _ = sender.send(crate::app::AsyncAction::LoadingMessage(msg)).await;
            }

            let tx_clone = tx.clone();
            let cat_id = category_id.to_string();

            let unique_streams = tokio::task::spawn_blocking(move || -> Result<Vec<Stream>, anyhow::Error> {
                if let Some(ref sender) = tx_clone {
                    let _ = sender.blocking_send(crate::app::AsyncAction::LoadingMessage(
                        "Phase 3/4: Decoding provider payload into channel records...".to_string()
                    ));
                }

                let streams = serde_json::from_slice::<Vec<Stream>>(&bytes)
                    .map_err(|e| anyhow::anyhow!("Failed to parse live streams JSON (category {}): {}", cat_id, e))?;

                if let Some(ref sender) = tx_clone {
                    let _ = sender.blocking_send(crate::app::AsyncAction::LoadingMessage(format!(
                        "Phase 4/4: Cleaning, filtering, and organizing {} channels...",
                        streams.len()
                    )));
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
                    let _ = sender.blocking_send(crate::app::AsyncAction::LoadingMessage(format!(
                        "Phase 4/4: Finalized {} unique channels. Building the browser index...",
                        filtered.len()
                    )));
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

    /// Get stream URL with fallback extensions if the primary fails
    pub fn get_stream_url_with_fallback(&self, stream_id: &str, primary_ext: &str) -> Vec<String> {
        let formats = match primary_ext {
            "ts" => vec!["ts", "m3u8", "mp4"],
            "m3u8" => vec!["m3u8", "ts", "mp4"],
            "mp4" => vec!["mp4", "m3u8", "ts"],
            _ => vec![primary_ext, "ts", "m3u8", "mp4"],
        };

        formats
            .iter()
            .map(|ext| {
                format!(
                    "{}/live/{}/{}/{}.{}",
                    self.base_url, self.username, self.password, stream_id, ext
                )
            })
            .collect()
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

// ============================================================================
// M3U URL Playlist Client
// ============================================================================

/// Parsed M3U entry from an M3U playlist file
#[derive(Debug, Clone)]
struct M3uEntry {
    name: String,
    group: String,
    logo: Option<String>,
    url: String,
}

/// Client for M3U URL playlists. Downloads and parses .m3u/.m3u8 files
/// into the same Category/Stream data model used by Xtream.
#[derive(Debug, Clone)]
pub struct M3uClient {
    pub m3u_url: String,
    client: reqwest::Client,
    /// Parsed entries cached after first download
    cached_entries: Arc<Mutex<Option<Vec<M3uEntry>>>>,
    /// Map of stream_id -> direct URL for playback
    stream_urls: Arc<Mutex<HashMap<String, String>>>,
}

impl M3uClient {
    pub fn new(m3u_url: String) -> Self {
        let m3u_url = m3u_url.trim().to_string();

        let builder = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .danger_accept_invalid_certs(true);

        #[cfg(not(target_arch = "wasm32"))]
        let builder = builder
            .timeout(std::time::Duration::from_secs(120))
            .connect_timeout(std::time::Duration::from_secs(30))
            .gzip(true)
            .brotli(true);

        let client = builder.build().unwrap_or_else(|_| reqwest::Client::new());

        Self {
            m3u_url,
            client,
            cached_entries: Arc::new(Mutex::new(None)),
            stream_urls: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create an M3U client with DNS-over-HTTPS resolver for ISP-blocked domains
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn new_with_doh(m3u_url: String, dns_provider: crate::config::DnsProvider) -> Result<Self, anyhow::Error> {
        use hickory_resolver::AsyncResolver;
        use hickory_resolver::config::{ResolverConfig, ResolverOpts, NameServerConfig, Protocol};
        use std::net::SocketAddr;
        use crate::config::DnsProvider;

        let m3u_url = m3u_url.trim().to_string();

        // If System DNS, skip custom resolver
        if dns_provider == DnsProvider::System {
            return Ok(Self::new(m3u_url));
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
            DnsProvider::System => unreachable!(),
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
            .timeout(std::time::Duration::from_secs(120))
            .connect_timeout(std::time::Duration::from_secs(30))
            .pool_max_idle_per_host(4)
            .tcp_keepalive(std::time::Duration::from_secs(60))
            .gzip(true)
            .brotli(true)
            .build()?;

        Ok(Self {
            m3u_url,
            client,
            cached_entries: Arc::new(Mutex::new(None)),
            stream_urls: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Download and parse the M3U file, caching the result
    async fn fetch_and_parse(&self) -> Result<Vec<M3uEntry>, anyhow::Error> {
        // Check cache first
        {
            let cache = self.cached_entries.lock().await;
            if let Some(ref entries) = *cache {
                return Ok(entries.clone());
            }
        }

        let resp = self.client.get(&self.m3u_url)
            .send().await
            .map_err(|e| anyhow::anyhow!("Failed to download M3U playlist: {}", e))?;

        if !resp.status().is_success() {
            return Err(anyhow::anyhow!("M3U download failed with status: {}", resp.status()));
        }

        let body = resp.text().await
            .map_err(|e| anyhow::anyhow!("Failed to read M3U playlist body: {}", e))?;

        let entries = Self::parse_m3u(&body);

        // Cache the stream URLs for playback
        {
            let mut url_map = self.stream_urls.lock().await;
            for entry in &entries {
                let id = Self::make_stream_id(&entry.url);
                url_map.insert(id, entry.url.clone());
            }
        }

        // Cache parsed entries
        {
            let mut cache = self.cached_entries.lock().await;
            *cache = Some(entries.clone());
        }

        Ok(entries)
    }

    /// Parse M3U file content into entries
    fn parse_m3u(content: &str) -> Vec<M3uEntry> {
        let mut entries = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();

            if line.starts_with("#EXTINF:") {
                // Parse the EXTINF line for metadata
                let extinf = &line[8..]; // Skip "#EXTINF:"

                // Extract group-title
                let group = Self::extract_attribute(extinf, "group-title")
                    .unwrap_or_else(|| "Uncategorized".to_string());

                // Extract tvg-logo
                let logo = Self::extract_attribute(extinf, "tvg-logo");

                // Extract stream name (after the last comma)
                let name = extinf.rsplit(',').next()
                    .unwrap_or("Unknown")
                    .trim()
                    .to_string();

                // Next non-comment, non-empty line should be the URL
                i += 1;
                while i < lines.len() {
                    let url_line = lines[i].trim();
                    if !url_line.is_empty() && !url_line.starts_with('#') {
                        entries.push(M3uEntry {
                            name,
                            group,
                            logo,
                            url: url_line.to_string(),
                        });
                        break;
                    }
                    i += 1;
                }
            }

            i += 1;
        }

        entries
    }

    /// Extract an attribute value from an EXTINF line, e.g. group-title="Sports"
    fn extract_attribute(extinf: &str, attr: &str) -> Option<String> {
        let search = format!("{}=\"", attr);
        if let Some(start) = extinf.find(&search) {
            let value_start = start + search.len();
            if let Some(end) = extinf[value_start..].find('"') {
                let value = extinf[value_start..value_start + end].trim().to_string();
                if !value.is_empty() {
                    return Some(value);
                }
            }
        }
        None
    }

    /// Generate a deterministic stream ID from a URL using a simple hash
    fn make_stream_id(url: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        url.hash(&mut hasher);
        format!("m3u_{}", hasher.finish())
    }

    /// Authenticate by testing if the M3U URL is reachable and valid
    pub async fn authenticate(&self) -> Result<(bool, Option<UserInfo>, Option<ServerInfo>), crate::errors::IptvError> {
        let resp = self.client.get(&self.m3u_url)
            .send().await
            .map_err(|e| {
                if crate::doh::is_dns_error(&e) {
                    crate::errors::IptvError::DnsResolution(
                        crate::doh::redact_url(&self.m3u_url),
                        e.to_string(),
                    )
                } else if e.is_timeout() {
                    crate::errors::IptvError::ConnectionTimeout(
                        crate::doh::redact_url(&self.m3u_url), 120
                    )
                } else {
                    crate::errors::IptvError::ConnectionFailed(
                        crate::errors::ConnectionStage::TcpConnection,
                        e.to_string(),
                    )
                }
            })?;

        if !resp.status().is_success() {
            return Ok((false, None, None));
        }

        // Peek at the body to check if it looks like an M3U file
        let body = resp.text().await
            .map_err(|e| crate::errors::IptvError::ParseError(e.to_string()))?;

        let is_valid = body.contains("#EXTINF") || body.starts_with("#EXTM3U");

        if is_valid {
            // Parse and cache the entries while we have the body
            let entries = Self::parse_m3u(&body);
            let total = entries.len();

            // Cache stream URLs
            {
                let mut url_map = self.stream_urls.lock().await;
                for entry in &entries {
                    let id = Self::make_stream_id(&entry.url);
                    url_map.insert(id, entry.url.clone());
                }
            }

            // Cache parsed entries
            {
                let mut cache = self.cached_entries.lock().await;
                *cache = Some(entries);
            }

            // Build a synthetic UserInfo
            let ui = UserInfo {
                auth: 1,
                status: Some("Active".to_string()),
                exp_date: None,
                max_connections: None,
                active_cons: None,
                total_live_streams: Some(crate::flex_id::FlexId::Number(total as i64)),
                total_vod_streams: Some(crate::flex_id::FlexId::Number(0)),
                total_series_streams: Some(crate::flex_id::FlexId::Number(0)),
            };

            Ok((true, Some(ui), None))
        } else {
            Ok((false, None, None))
        }
    }

    /// Get categories from the parsed M3U data
    pub async fn get_live_categories(&self) -> Result<Vec<Category>, anyhow::Error> {
        let entries = self.fetch_and_parse().await?;

        // Collect unique group names as categories
        let mut seen = std::collections::HashSet::new();
        let mut categories = Vec::new();
        let mut id_counter: usize = 1;

        for entry in &entries {
            if seen.insert(entry.group.clone()) {
                categories.push(Category {
                    category_id: format!("m3u_cat_{}", id_counter),
                    category_name: entry.group.clone(),
                    ..Default::default()
                });
                id_counter += 1;
            }
        }

        Ok(categories)
    }

    /// Get streams for a given category
    pub async fn get_live_streams(&self, category_id: &str, _tx: Option<tokio::sync::mpsc::Sender<crate::app::AsyncAction>>) -> Result<Vec<Stream>, anyhow::Error> {
        let entries = self.fetch_and_parse().await?;

        // Build category name -> id mapping
        let categories = self.get_live_categories().await?;
        let cat_name_to_id: HashMap<String, String> = categories.iter()
            .map(|c| (c.category_name.clone(), c.category_id.clone()))
            .collect();

        let target_cat_name = if category_id == "ALL" {
            None
        } else {
            categories.iter()
                .find(|c| c.category_id == category_id)
                .map(|c| c.category_name.clone())
        };

        let mut streams = Vec::new();
        for entry in &entries {
            // Filter by category if specified
            if let Some(ref target) = target_cat_name {
                if &entry.group != target {
                    continue;
                }
            }

            let stream_id_str = Self::make_stream_id(&entry.url);
            let cat_id = cat_name_to_id.get(&entry.group).cloned();

            streams.push(Stream {
                num: None,
                name: entry.name.clone(),
                stream_display_name: None,
                stream_type: "live".to_string(),
                stream_id: crate::flex_id::FlexId::String(stream_id_str),
                stream_icon: entry.logo.clone(),
                epg_channel_id: None,
                added: None,
                category_id: cat_id,
                container_extension: None,
                rating: None,
                rating_5: None,
                cached_parsed: None,
                search_name: String::new(),
                is_american: false,
                is_english: false,
                clean_name: String::new(),
                latency_ms: None,
                account_name: None,
            });
        }

        Ok(streams)
    }

    /// Get the direct stream URL for a given stream ID
    pub fn get_stream_url(&self, stream_id: &str) -> String {
        // Try to get from the cached URL map synchronously
        // Since this is called from a sync context, we use try_lock
        if let Ok(urls) = self.stream_urls.try_lock() {
            if let Some(url) = urls.get(stream_id) {
                return url.clone();
            }
        }
        // Fallback: return the stream_id itself (it might be a URL already)
        stream_id.to_string()
    }
}
