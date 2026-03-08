/// DNS-over-HTTPS fallback utility.
///
/// Extracts the hostname from a URL, queries multiple DoH providers for an A record,
/// and attempts to reconnect using the resolved IP. Only works for plain HTTP — HTTPS
/// will fail due to TLS SNI mismatch so we skip the retry in that case.

/// Attempt to resolve DNS via DoH and retry the request.
/// Returns `Some(Response)` if a DoH provider resolves and the retry succeeds.
/// Returns `None` if DNS resolution fails, URL is HTTPS, or the retry fails.
#[cfg(not(target_arch = "wasm32"))]
pub async fn try_doh_fallback(
    client: &reqwest::Client,
    url: &str,
) -> Option<reqwest::Response> {
    // Only attempt IP substitution for plain HTTP endpoints.
    // HTTPS requires TLS SNI to match the hostname — substituting an IP will
    // cause a certificate mismatch and the handshake will always fail.
    if !url.starts_with("http://") {
        #[cfg(debug_assertions)]
        println!("DEBUG: DoH skipped for HTTPS URL (SNI mismatch would occur)");
        return None;
    }

    // Extract hostname from URL
    let host_start = url.find("://")?;
    let rest = &url[host_start + 3..];
    let host_end = rest.find(':').or_else(|| rest.find('/'))?;
    let hostname = &rest[..host_end];

    let providers = [
        "https://9.9.9.9/dns-query",
        "https://1.1.1.1/dns-query",
        "https://8.8.8.8/dns-query",
    ];

    for doh_base in providers {
        let doh_url = format!("{}?name={}", doh_base, hostname);
        #[cfg(debug_assertions)]
        println!("DEBUG: Attempting DoH via {}", doh_base);

        if let Ok(doh_resp) = client
            .get(&doh_url)
            .header("Accept", "application/dns-json")
            .send()
            .await
        {
            if let Ok(doh_json) = doh_resp.json::<serde_json::Value>().await {
                if let Some(ip) = doh_json
                    .get("Answer")
                    .and_then(|a| a.as_array())
                    .and_then(|a| {
                        a.iter()
                            .find(|ans| ans.get("type") == Some(&serde_json::Value::from(1)))
                    })
                    .and_then(|a| a.get("data"))
                    .and_then(|d| d.as_str())
                {
                    #[cfg(debug_assertions)]
                    println!("DEBUG: DoH Resolved {} to {}", hostname, ip);

                    let resolved_url = url.replace(hostname, ip);
                    if let Ok(resp) = client
                        .get(&resolved_url)
                        .header("Host", hostname)
                        .send()
                        .await
                    {
                        return Some(resp);
                    }
                }
            }
        }
    }

    None
}

/// Check if an error chain contains DNS-related failure signals.
pub fn is_dns_error(error: &dyn std::error::Error) -> bool {
    let mut full = error.to_string().to_lowercase();
    let mut source = error.source();
    while let Some(s) = source {
        full.push(' ');
        full.push_str(&s.to_string().to_lowercase());
        source = s.source();
    }

    full.contains("dns")
        || full.contains("resolution")
        || full.contains("resolve")
        || full.contains("no such host")
        || full.contains("11004")
        || full.contains("no data")
}

/// Redact credentials from a URL for safe display in error messages and logs.
pub fn redact_url(url: &str) -> String {
    // Redact password= and username= query parameters
    let mut result = url.to_string();
    if let Some(pw_start) = result.find("password=") {
        let after = &result[pw_start + 9..];
        let pw_end = after.find('&').unwrap_or(after.len());
        result.replace_range(pw_start + 9..pw_start + 9 + pw_end, "***");
    }
    if let Some(un_start) = result.find("username=") {
        let after = &result[un_start + 9..];
        let un_end = after.find('&').unwrap_or(after.len());
        result.replace_range(un_start + 9..un_start + 9 + un_end, "***");
    }
    result
}
