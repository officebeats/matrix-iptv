/// M3U/M3U8 playlist parser for IPTV catalog ingestion.
///
/// Parses the extended M3U format (`#EXTM3U` + `#EXTINF` lines) commonly distributed
/// by IPTV providers. This is distinct from HLS segment playlists — this module handles
/// *catalog* playlists that list channel URLs with metadata tags.
///
/// # Format example
/// ```text
/// #EXTM3U
/// #EXTINF:-1 tvg-id="cnn.us" tvg-name="CNN" tvg-logo="http://..." group-title="News",CNN
/// http://provider.example/live/user/pass/12345.ts
/// ```

/// A single entry parsed from an M3U playlist.
#[derive(Debug, Clone, PartialEq)]
pub struct M3uEntry {
    /// Display name from the comma-suffix of the `#EXTINF` line.
    pub name: String,
    /// Direct stream URL.
    pub url: String,
    /// `group-title` attribute — used as the category name.
    pub group_title: String,
    /// `tvg-id` attribute — used for EPG correlation.
    pub tvg_id: String,
    /// `tvg-logo` attribute — artwork URL.
    pub tvg_logo: String,
    /// `tvg-name` attribute — alternate display name from the EPG.
    pub tvg_name: String,
    /// Content type inferred from `tvg-type` or `group-title` heuristics.
    pub tvg_type: M3uEntryType,
}

/// Inferred content type for an M3U entry.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum M3uEntryType {
    #[default]
    Live,
    Movie,
    Series,
}

impl M3uEntryType {
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "movie" | "vod" => M3uEntryType::Movie,
            "series" => M3uEntryType::Series,
            _ => M3uEntryType::Live,
        }
    }
}

/// Extract the value of a quoted or unquoted attribute from an `#EXTINF` line.
///
/// Handles both `key="value"` and `key=value` forms.
fn extract_attr<'a>(line: &'a str, key: &str) -> &'a str {
    let search = format!("{}=", key);
    if let Some(pos) = line.find(&search) {
        let rest = &line[pos + search.len()..];
        if rest.starts_with('"') {
            // Quoted value: find the closing quote
            let inner = &rest[1..];
            if let Some(end) = inner.find('"') {
                return &inner[..end];
            }
            // Malformed — no closing quote; return to end-of-line or next space
            return inner;
        } else {
            // Unquoted value: terminate at space or comma
            let end = rest
                .find(|c: char| c == ' ' || c == ',')
                .unwrap_or(rest.len());
            return &rest[..end];
        }
    }
    ""
}

/// Parse an extended M3U playlist string into a list of [`M3uEntry`] values.
///
/// - Lines not starting with `#EXTINF` or that are blank comments are skipped.
/// - A URL line immediately following an `#EXTINF` line is treated as the stream URL.
/// - Malformed entries (no URL, no name) are silently skipped.
/// - This function is CPU-bound and suitable for offloading via `tokio::task::spawn_blocking`.
pub fn parse_m3u(content: &str) -> Vec<M3uEntry> {
    let mut entries = Vec::new();
    let mut pending_extinf: Option<String> = None;

    for raw_line in content.lines() {
        let line = raw_line.trim();

        if line.is_empty() {
            continue;
        }

        if line.starts_with("#EXTINF") {
            pending_extinf = Some(line.to_string());
            continue;
        }

        if line.starts_with('#') {
            // Any other directive or comment — skip
            continue;
        }

        // If we reach here the line should be a URL
        let url = line.to_string();
        if url.is_empty() {
            pending_extinf = None;
            continue;
        }

        if let Some(extinf_line) = pending_extinf.take() {
            let name = extinf_line
                .find(',')
                .map(|pos| extinf_line[pos + 1..].trim().to_string())
                .unwrap_or_default();

            let group_title = extract_attr(&extinf_line, "group-title").to_string();
            let tvg_id = extract_attr(&extinf_line, "tvg-id").to_string();
            let tvg_logo = extract_attr(&extinf_line, "tvg-logo").to_string();
            let tvg_name = extract_attr(&extinf_line, "tvg-name").to_string();

            // Determine content type
            let tvg_type_str = extract_attr(&extinf_line, "tvg-type");
            let tvg_type = if !tvg_type_str.is_empty() {
                M3uEntryType::from_str(tvg_type_str)
            } else {
                // Heuristic: check group-title for movie/series keywords
                let lower_group = group_title.to_lowercase();
                if lower_group.contains("movie")
                    || lower_group.contains("film")
                    || lower_group.contains("vod")
                {
                    M3uEntryType::Movie
                } else if lower_group.contains("series") || lower_group.contains("show") {
                    M3uEntryType::Series
                } else {
                    M3uEntryType::Live
                }
            };

            if name.is_empty() && url.is_empty() {
                continue;
            }

            entries.push(M3uEntry {
                name,
                url,
                group_title,
                tvg_id,
                tvg_logo,
                tvg_name,
                tvg_type,
            });
        }
        // URL line without a preceding #EXTINF — skip (plain M3U without metadata)
    }

    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_M3U: &str = r#"#EXTM3U
#EXTINF:-1 tvg-id="cnn.us" tvg-name="CNN HD" tvg-logo="http://logo.example/cnn.png" group-title="News",CNN
http://provider.example/live/user/pass/1001.ts
#EXTINF:-1 tvg-id="espn.us" tvg-name="ESPN" tvg-logo="" group-title="Sports",ESPN
http://provider.example/live/user/pass/1002.ts
#EXTINF:-1 tvg-id="" tvg-name="" tvg-logo="" group-title="Movies" tvg-type="movie",The Matrix
http://provider.example/vod/user/pass/9001.mp4
#EXTINF:-1 group-title="Series" tvg-type="series",Breaking Bad S01E01
http://provider.example/series/user/pass/5001.mkv
"#;

    #[test]
    fn test_parse_m3u_count() {
        let entries = parse_m3u(SAMPLE_M3U);
        assert_eq!(entries.len(), 4);
    }

    #[test]
    fn test_parse_m3u_live_entry() {
        let entries = parse_m3u(SAMPLE_M3U);
        let cnn = &entries[0];
        assert_eq!(cnn.name, "CNN");
        assert_eq!(cnn.url, "http://provider.example/live/user/pass/1001.ts");
        assert_eq!(cnn.group_title, "News");
        assert_eq!(cnn.tvg_id, "cnn.us");
        assert_eq!(cnn.tvg_name, "CNN HD");
        assert_eq!(cnn.tvg_logo, "http://logo.example/cnn.png");
        assert_eq!(cnn.tvg_type, M3uEntryType::Live);
    }

    #[test]
    fn test_parse_m3u_movie_tvg_type() {
        let entries = parse_m3u(SAMPLE_M3U);
        let movie = &entries[2];
        assert_eq!(movie.name, "The Matrix");
        assert_eq!(movie.tvg_type, M3uEntryType::Movie);
    }

    #[test]
    fn test_parse_m3u_series_tvg_type() {
        let entries = parse_m3u(SAMPLE_M3U);
        let series = &entries[3];
        assert_eq!(series.tvg_type, M3uEntryType::Series);
    }

    #[test]
    fn test_parse_m3u_group_heuristic_movie() {
        let content =
            "#EXTM3U\n#EXTINF:-1 group-title=\"Movies HD\",Inception\nhttp://example.com/1.mp4\n";
        let entries = parse_m3u(content);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].tvg_type, M3uEntryType::Movie);
    }

    #[test]
    fn test_parse_m3u_group_heuristic_series() {
        let content = "#EXTM3U\n#EXTINF:-1 group-title=\"TV Series\",Game of Thrones\nhttp://example.com/1.mkv\n";
        let entries = parse_m3u(content);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].tvg_type, M3uEntryType::Series);
    }

    #[test]
    fn test_parse_m3u_empty() {
        let entries = parse_m3u("#EXTM3U\n");
        assert!(entries.is_empty());
    }

    #[test]
    fn test_parse_m3u_missing_group_title() {
        let content = "#EXTM3U\n#EXTINF:-1 tvg-id=\"abc\",ABC\nhttp://example.com/abc.ts\n";
        let entries = parse_m3u(content);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].group_title, "");
        assert_eq!(entries[0].name, "ABC");
    }

    #[test]
    fn test_parse_m3u_blank_lines_and_comments() {
        let content = "#EXTM3U\n\n# This is a comment\n#EXTINF:-1 group-title=\"News\",BBC\nhttp://example.com/bbc.ts\n\n";
        let entries = parse_m3u(content);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "BBC");
    }

    #[test]
    fn test_parse_m3u_url_without_extinf_skipped() {
        let content = "#EXTM3U\nhttp://orphan.example/stream.ts\n#EXTINF:-1 group-title=\"News\",Sky\nhttp://example.com/sky.ts\n";
        let entries = parse_m3u(content);
        // Orphan URL should be skipped, only the properly-annotated entry is returned
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "Sky");
    }

    #[test]
    fn test_extract_attr_quoted() {
        assert_eq!(
            extract_attr(r#"#EXTINF:-1 group-title="News & Sports""#, "group-title"),
            "News & Sports"
        );
    }

    #[test]
    fn test_extract_attr_missing() {
        assert_eq!(
            extract_attr("#EXTINF:-1 tvg-name=\"Test\"", "group-title"),
            ""
        );
    }
}
