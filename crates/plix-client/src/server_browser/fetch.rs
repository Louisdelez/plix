//! HTTP client for fetching server list from master server.

use plix_common::server_browser::ServerListResponse;
use std::time::Duration;

/// Errors that can occur when fetching the server list.
#[derive(Debug, thiserror::Error)]
pub enum FetchError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Request timed out")]
    Timeout,

    #[error("Server returned error ({0}): {1}")]
    ServerError(u16, String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

/// Fetch the server list from the master server.
///
/// Uses a 5 second timeout for the request.
pub async fn fetch_servers(master_url: &str) -> Result<ServerListResponse, FetchError> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(FetchError::Network)?;

    let url = format!("{}/servers", master_url);

    let response = client.get(&url).send().await.map_err(|e| {
        if e.is_timeout() {
            FetchError::Timeout
        } else {
            FetchError::Network(e)
        }
    })?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        return Err(FetchError::ServerError(status, body));
    }

    let server_list: ServerListResponse = response
        .json()
        .await
        .map_err(|e| FetchError::InvalidResponse(e.to_string()))?;

    Ok(server_list)
}

/// Blocking version of fetch_servers for use in synchronous game loop.
///
/// Uses a 5 second timeout for the request.
pub fn fetch_servers_blocking(master_url: &str) -> Result<ServerListResponse, FetchError> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(FetchError::Network)?;

    let url = format!("{}/servers", master_url);

    let response = client.get(&url).send().map_err(|e| {
        if e.is_timeout() {
            FetchError::Timeout
        } else {
            FetchError::Network(e)
        }
    })?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response.text().unwrap_or_default();
        return Err(FetchError::ServerError(status, body));
    }

    let server_list: ServerListResponse = response
        .json()
        .map_err(|e| FetchError::InvalidResponse(e.to_string()))?;

    Ok(server_list)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_error_display() {
        let err = FetchError::Timeout;
        assert_eq!(err.to_string(), "Request timed out");

        let err = FetchError::ServerError(404, "Not found".to_string());
        assert!(err.to_string().contains("404"));
        assert!(err.to_string().contains("Not found"));
    }

    // Note: Actual HTTP tests would require a mock server or integration test
}
