//! Manifest fetching from remote server

use crate::error::LauncherError;
use crate::manifest::Manifest;
use std::time::Duration;

/// Fetch manifest from a remote URL
pub fn fetch_manifest(url: &str, timeout_seconds: u64) -> Result<Manifest, LauncherError> {
    tracing::debug!("Fetching manifest from {}", url);

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(timeout_seconds))
        .user_agent(format!("plix-launcher/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| LauncherError::Network(e))?;

    let response = client.get(url).send().map_err(|e| {
        tracing::warn!("Failed to fetch manifest: {}", e);
        LauncherError::Network(e)
    })?;

    if !response.status().is_success() {
        return Err(LauncherError::Manifest(format!(
            "HTTP {} fetching manifest",
            response.status()
        )));
    }

    let content = response.text().map_err(|e| LauncherError::Network(e))?;
    tracing::debug!("Manifest fetched, {} bytes", content.len());

    parse_manifest(&content)
}

/// Parse manifest from TOML string
pub fn parse_manifest(content: &str) -> Result<Manifest, LauncherError> {
    let manifest: Manifest = toml::from_str(content)?;
    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_manifest_valid() {
        let content = r#"
manifest_version = 1
version = "1.0.0"
release_date = 1734480000

[[files]]
path = "plix-client"
url = "https://example.com/plix-client"
size = 1000
sha256 = "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
"#;
        let manifest = parse_manifest(content).unwrap();
        assert_eq!(manifest.version, "1.0.0");
    }

    #[test]
    fn test_parse_manifest_invalid() {
        let content = "invalid toml {{";
        let result = parse_manifest(content);
        assert!(result.is_err());
    }
}
