use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Category {
    pub category_id: String,
    pub category_name: String,
    pub parent_id: ::serde_json::Value, // frequent null or 0
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
    pub num: Option<serde_json::Value>,
    #[serde(default)]
    pub name: String,
    pub stream_display_name: Option<String>,

    #[serde(default)]
    pub stream_type: String,

    #[serde(alias = "series_id", default)]
    pub stream_id: serde_json::Value,

    #[serde(alias = "cover")]
    pub stream_icon: Option<String>,

    pub epg_channel_id: Option<String>,
    pub added: Option<String>,
    pub category_id: Option<String>,
    pub container_extension: Option<String>,
    pub rating: Option<serde_json::Value>,
    pub rating_5: Option<serde_json::Value>,
    
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserInfo {
    pub auth: i32,
    pub status: Option<String>,
    pub exp_date: Option<serde_json::Value>,
    pub max_connections: Option<serde_json::Value>,
    pub active_cons: Option<serde_json::Value>,
    pub total_live_streams: Option<serde_json::Value>,
    pub total_vod_streams: Option<serde_json::Value>,
    pub total_series_streams: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerInfo {
    pub timezone: Option<String>,
    pub server_time: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SeriesEpisode {
    pub id: Option<serde_json::Value>,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EpgListing {
    pub id: Option<String>,
    pub epg_id: Option<String>,
    pub title: String,
    pub start: String,
    pub end: String,
    pub description: Option<String>,
    pub start_timestamp: Option<serde_json::Value>,
    pub stop_timestamp: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EpgResponse {
    pub epg_listings: Vec<EpgListing>,
}

#[derive(Debug, Clone)]
pub struct XtreamClient {
    pub base_url: String,
    pub username: String,
    pub password: String,
    client: reqwest::Client,
}

pub fn get_id_str(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(val) => val.clone(),
        serde_json::Value::Number(val) => val.to_string(),
        _ => v.to_string(),
    }
}

impl XtreamClient {
    pub fn new(base_url: String, username: String, password: String) -> Self {
        let base_url = if base_url.ends_with('/') {
            base_url[..base_url.len() - 1].to_string()
        } else {
            base_url
        };

        // Build client with User-Agent and timeouts
        let builder = reqwest::Client::builder()
            .user_agent("IPTV Smarters Pro");
        
        #[cfg(not(target_arch = "wasm32"))]
        let builder = builder
            .timeout(std::time::Duration::from_secs(60))
            .connect_timeout(std::time::Duration::from_secs(10));

        let client = builder.build().unwrap_or_else(|_| reqwest::Client::new());

        Self {
            base_url,
            username,
            password,
            client,
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
                .user_agent("IPTV Smarters Pro")
                .timeout(std::time::Duration::from_secs(60))
                .connect_timeout(std::time::Duration::from_secs(10))
                .build()?;

            return Ok(Self {
                base_url,
                username,
                password,
                client,
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
            .user_agent("IPTV Smarters Pro")
            .dns_resolver(Arc::new(DohResolver(async_resolver)))
            .timeout(std::time::Duration::from_secs(60))
            .connect_timeout(std::time::Duration::from_secs(10))
            .build()?;

        Ok(Self {
            base_url,
            username,
            password,
            client,
        })
    }

    pub async fn authenticate(
        &self,
    ) -> Result<(bool, Option<UserInfo>, Option<ServerInfo>), anyhow::Error> {
        let url = format!(
            "{}/player_api.php?username={}&password={}",
            self.base_url, self.username, self.password
        );
        let resp = self.client.get(&url).send().await
            .map_err(|e| {
                let mut msg = format!("Network request failed: {}", e);
                if e.is_connect() { msg = format!("Connection failed (Check internet/URL): {}", e); }
                if e.is_timeout() { msg = format!("Request timed out (Server slow?): {}", e); }
                if e.is_request() && e.to_string().contains("dns") { msg = format!("DNS Resolution Error (Try Quad9/Cloudflare DNS in Settings): {}", e); }
                anyhow::anyhow!(msg)
            })?;

        if !resp.status().is_success() {
            return Err(anyhow::anyhow!("Server returned error status: {}", resp.status()));
        }

        #[derive(Deserialize)]
        struct AuthResponse {
            user_info: Option<UserInfo>,
            server_info: Option<ServerInfo>,
        }

        let bytes = resp.bytes().await
            .map_err(|e| anyhow::anyhow!("Failed to read response body: {}", e))?;

        let bytes_for_auth = bytes.clone();
        let auth_res = tokio::task::spawn_blocking(move || {
            serde_json::from_slice::<AuthResponse>(&bytes_for_auth)
        }).await.map_err(|e| anyhow::anyhow!("Spawn blocking failed: {}", e))?;

        match auth_res {
            Ok(json) => {
                if let Some(info) = json.user_info {
                    return Ok((info.auth == 1, Some(info), json.server_info));
                }
                Ok((false, None, None))
            }
            Err(e) => {
                // Check if it's plain text error
                let text = String::from_utf8_lossy(&bytes).to_lowercase();
                if text.contains("invalid") || text.contains("expired") || text.contains("disabled") {
                    return Ok((false, None, None));
                }
                Err(anyhow::anyhow!("Failed to parse server response: {}. Body: {}", e, text.chars().take(100).collect::<String>()))
            }
        }
    }

    pub async fn get_live_categories(&self) -> Result<Vec<Category>, anyhow::Error> {
        let url = format!(
            "{}/player_api.php?username={}&password={}&action=get_live_categories",
            self.base_url, self.username, self.password
        );
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
        let resp = self.client.get(&url).send().await
            .map_err(|e| {
                let mut msg = format!("Failed to fetch live streams (category {}): {}", category_id, e);
                if e.is_connect() { msg = format!("Connection failed: {}", e); }
                if e.is_timeout() { msg = format!("Request timed out: {}", e); }
                if e.is_request() && e.to_string().contains("dns") { msg = format!("DNS Resolution Error: {}", e); }
                anyhow::anyhow!(msg)
            })?;
        
        let bytes = resp.bytes().await
            .map_err(|e| anyhow::anyhow!("Failed to read live streams body (category {}): {}", category_id, e))?;

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
    }

    pub async fn get_vod_categories(&self) -> Result<Vec<Category>, anyhow::Error> {
        let url = format!(
            "{}/player_api.php?username={}&password={}&action=get_vod_categories",
            self.base_url, self.username, self.password
        );
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
    }

    pub async fn get_vod_streams(&self, category_id: &str) -> Result<Vec<Stream>, anyhow::Error> {
        let url = format!(
            "{}/player_api.php?username={}&password={}&action=get_vod_streams&category_id={}",
            self.base_url, self.username, self.password, category_id
        );
        let resp = self.client.get(&url).send().await
            .map_err(|e| {
                let mut msg = format!("Failed to fetch VOD streams (category {}): {}", category_id, e);
                if e.is_connect() { msg = format!("Connection failed: {}", e); }
                if e.is_timeout() { msg = format!("Request timed out: {}", e); }
                if e.is_request() && e.to_string().contains("dns") { msg = format!("DNS Resolution Error: {}", e); }
                anyhow::anyhow!(msg)
            })?;
        
        let bytes = resp.bytes().await
            .map_err(|e| anyhow::anyhow!("Failed to read VOD streams body (category {}): {}", category_id, e))?;

        if bytes.is_empty() || bytes == "{}" || bytes == "null" {
            return Ok(Vec::new());
        }

        let streams = tokio::task::spawn_blocking(move || {
            serde_json::from_slice::<Vec<Stream>>(&bytes)
        }).await.map_err(|e| anyhow::anyhow!("Spawn blocking failed: {}", e))?
          .map_err(|e| anyhow::anyhow!("Failed to parse VOD streams JSON (category {}): {}", category_id, e))?;

        Ok(streams)
    }

    pub async fn get_vod_streams_all(&self) -> Result<Vec<Stream>, anyhow::Error> {
        let url = format!(
            "{}/player_api.php?username={}&password={}&action=get_vod_streams",
            self.base_url, self.username, self.password
        );
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
    }

    pub async fn get_series_categories(&self) -> Result<Vec<Category>, anyhow::Error> {
        let url = format!(
            "{}/player_api.php?username={}&password={}&action=get_series_categories",
            self.base_url, self.username, self.password
        );
        let resp = self.client.get(&url).send().await
            .map_err(|e| anyhow::anyhow!("Failed to fetch series categories: {}", e))?;
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
    }

    pub async fn get_series_all(&self) -> Result<Vec<Stream>, anyhow::Error> {
        let url = format!(
            "{}/player_api.php?username={}&password={}&action=get_series",
            self.base_url, self.username, self.password
        );
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
        let streams: Vec<Stream> = resp.json().await
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
