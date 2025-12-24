use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Category {
    pub category_id: String,
    pub category_name: String,
    pub parent_id: ::serde_json::Value, // frequent null or 0
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Stream {
    pub num: Option<serde_json::Value>, // Sometimes int, sometimes string, sometimes missing
    pub name: String,
    pub stream_display_name: Option<String>,

    #[serde(default)]
    pub stream_type: String, // Live/Movie usually have this. Series might not.

    #[serde(alias = "series_id")]
    pub stream_id: serde_json::Value, // Can be int or string

    #[serde(alias = "cover")]
    pub stream_icon: Option<String>,

    pub epg_channel_id: Option<String>,
    pub added: Option<String>,
    pub category_id: Option<String>,
    pub container_extension: Option<String>, // Series might not have this
    pub rating: Option<serde_json::Value>,
    pub rating_5: Option<serde_json::Value>,
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

#[derive(Debug, Clone)]
pub struct XtreamClient {
    pub base_url: String,
    pub username: String,
    pub password: String,
    client: reqwest::Client,
}

impl XtreamClient {
    pub fn new(base_url: String, username: String, password: String) -> Self {
        let base_url = if base_url.ends_with('/') {
            base_url[..base_url.len() - 1].to_string()
        } else {
            base_url
        };

        // Build client with User-Agent (DoH resolver is applied per-request if needed)
        let client = reqwest::Client::builder()
            .user_agent("IPTV Smarters Pro")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

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
    ) -> Result<Self, anyhow::Error> {
        use reqwest_hickory_resolver::HickoryResolver;
        use std::sync::Arc;

        let base_url = if base_url.ends_with('/') {
            base_url[..base_url.len() - 1].to_string()
        } else {
            base_url
        };

        // Create DNS resolver using HickoryResolver (uses system DNS by default)
        let resolver = HickoryResolver::default();

        let client = reqwest::Client::builder()
            .user_agent("IPTV Smarters Pro")
            .dns_resolver(Arc::new(resolver))
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
        let resp = self.client.get(&url).send().await?;

        #[derive(Deserialize)]
        struct AuthResponse {
            user_info: Option<UserInfo>,
            server_info: Option<ServerInfo>,
        }

        if let Ok(json) = resp.json::<AuthResponse>().await {
            if let Some(info) = json.user_info {
                return Ok((info.auth == 1, Some(info), json.server_info));
            }
        }
        Ok((false, None, None))
    }

    pub async fn get_live_categories(&self) -> Result<Vec<Category>, anyhow::Error> {
        let url = format!(
            "{}/player_api.php?username={}&password={}&action=get_live_categories",
            self.base_url, self.username, self.password
        );
        let resp = self.client.get(&url).send().await?;
        let categories: Vec<Category> = resp.json().await?;
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
        let resp = self.client.get(&url).send().await?;
        let streams: Vec<Stream> = resp.json().await?;
        Ok(streams)
    }

    pub async fn get_vod_categories(&self) -> Result<Vec<Category>, anyhow::Error> {
        let url = format!(
            "{}/player_api.php?username={}&password={}&action=get_vod_categories",
            self.base_url, self.username, self.password
        );
        let resp = self.client.get(&url).send().await?;
        let categories: Vec<Category> = resp.json().await?;
        Ok(categories)
    }

    pub async fn get_vod_streams(&self, category_id: &str) -> Result<Vec<Stream>, anyhow::Error> {
        let url = format!(
            "{}/player_api.php?username={}&password={}&action=get_vod_streams&category_id={}",
            self.base_url, self.username, self.password, category_id
        );
        let resp = self.client.get(&url).send().await?;
        let streams: Vec<Stream> = resp.json().await?;
        Ok(streams)
    }

    pub async fn get_vod_streams_all(&self) -> Result<Vec<Stream>, anyhow::Error> {
        let url = format!(
            "{}/player_api.php?username={}&password={}&action=get_vod_streams",
            self.base_url, self.username, self.password
        );
        let resp = self.client.get(&url).send().await?;
        let streams: Vec<Stream> = resp.json().await?;
        Ok(streams)
    }

    pub async fn get_series_categories(&self) -> Result<Vec<Category>, anyhow::Error> {
        let url = format!(
            "{}/player_api.php?username={}&password={}&action=get_series_categories",
            self.base_url, self.username, self.password
        );
        let resp = self.client.get(&url).send().await?;
        let categories: Vec<Category> = resp.json().await?;
        Ok(categories)
    }

    pub async fn get_series_all(&self) -> Result<Vec<serde_json::Value>, anyhow::Error> {
        let url = format!(
            "{}/player_api.php?username={}&password={}&action=get_series",
            self.base_url, self.username, self.password
        );
        let resp = self.client.get(&url).send().await?;
        let series: Vec<serde_json::Value> = resp.json().await?;
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
        let resp = self.client.get(&url).send().await?;
        // Series response often mimics Stream structure, or slightly different.
        // Using Stream struct for now as best-effort compatibility.
        // If it fails, we might need a dedicated Series struct.
        // However, standard Xtream often returns series objects.
        let streams: Vec<Stream> = resp.json().await?;
        Ok(streams)
    }

    pub async fn get_series_info(&self, series_id: &str) -> Result<SeriesInfo, anyhow::Error> {
        let url = format!(
            "{}/player_api.php?username={}&password={}&action=get_series_info&series_id={}",
            self.base_url, self.username, self.password, series_id
        );
        let resp = self.client.get(&url).send().await?;
        let info: SeriesInfo = resp.json().await?;
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
}
