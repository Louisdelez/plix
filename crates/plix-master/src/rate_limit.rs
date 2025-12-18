//! IP-based rate limiting for heartbeat requests.
//!
//! Implements a simple sliding window rate limiter that tracks
//! request counts per IP address.

use std::collections::HashMap;
use std::net::IpAddr;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Rate limiter for IP-based request limiting.
///
/// Uses a sliding window approach: tracks request timestamps
/// and counts requests within the last minute.
pub struct RateLimiter {
    /// Map of IP -> list of request timestamps
    requests: RwLock<HashMap<IpAddr, Vec<Instant>>>,
    /// Maximum requests allowed per minute
    max_requests_per_minute: u32,
    /// Window duration (1 minute)
    window: Duration,
}

impl RateLimiter {
    /// Create a new rate limiter with the specified limit.
    pub fn new(max_requests_per_minute: u32) -> Self {
        Self {
            requests: RwLock::new(HashMap::new()),
            max_requests_per_minute,
            window: Duration::from_secs(60),
        }
    }

    /// Check if a request from the given IP is allowed.
    ///
    /// Returns true if the request is within rate limits, false otherwise.
    /// This also records the request if allowed.
    pub async fn check(&self, ip: IpAddr) -> bool {
        let now = Instant::now();
        let cutoff = now - self.window;

        let mut requests = self.requests.write().await;
        let timestamps = requests.entry(ip).or_insert_with(Vec::new);

        // Remove expired timestamps
        timestamps.retain(|&t| t > cutoff);

        // Check if under limit
        if timestamps.len() >= self.max_requests_per_minute as usize {
            return false;
        }

        // Record this request
        timestamps.push(now);
        true
    }

    /// Get the current request count for an IP (for testing/debugging).
    #[cfg(test)]
    pub async fn get_count(&self, ip: IpAddr) -> usize {
        let now = Instant::now();
        let cutoff = now - self.window;

        let requests = self.requests.read().await;
        requests
            .get(&ip)
            .map(|timestamps| timestamps.iter().filter(|&&t| t > cutoff).count())
            .unwrap_or(0)
    }

    /// Clean up expired entries from all IPs (optional maintenance).
    pub async fn cleanup(&self) {
        let now = Instant::now();
        let cutoff = now - self.window;

        let mut requests = self.requests.write().await;
        requests.retain(|_, timestamps| {
            timestamps.retain(|&t| t > cutoff);
            !timestamps.is_empty()
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    fn test_ip() -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100))
    }

    fn other_ip() -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 101))
    }

    #[tokio::test]
    async fn test_allows_requests_under_limit() {
        let limiter = RateLimiter::new(10);
        let ip = test_ip();

        for _ in 0..10 {
            assert!(limiter.check(ip).await, "Should allow requests under limit");
        }
    }

    #[tokio::test]
    async fn test_blocks_11th_request() {
        let limiter = RateLimiter::new(10);
        let ip = test_ip();

        // First 10 requests should succeed
        for i in 0..10 {
            assert!(
                limiter.check(ip).await,
                "Request {} should be allowed",
                i + 1
            );
        }

        // 11th request should be blocked
        assert!(
            !limiter.check(ip).await,
            "11th request should be rate limited"
        );
    }

    #[tokio::test]
    async fn test_different_ips_independent() {
        let limiter = RateLimiter::new(5);
        let ip1 = test_ip();
        let ip2 = other_ip();

        // Use up all requests for ip1
        for _ in 0..5 {
            limiter.check(ip1).await;
        }

        // ip1 should be blocked
        assert!(!limiter.check(ip1).await, "ip1 should be rate limited");

        // ip2 should still be allowed
        assert!(limiter.check(ip2).await, "ip2 should be allowed");
    }

    #[tokio::test]
    async fn test_get_count() {
        let limiter = RateLimiter::new(10);
        let ip = test_ip();

        assert_eq!(limiter.get_count(ip).await, 0);

        limiter.check(ip).await;
        assert_eq!(limiter.get_count(ip).await, 1);

        limiter.check(ip).await;
        limiter.check(ip).await;
        assert_eq!(limiter.get_count(ip).await, 3);
    }

    #[tokio::test]
    async fn test_cleanup_removes_empty_entries() {
        let limiter = RateLimiter::new(10);
        let ip = test_ip();

        // Add a request
        limiter.check(ip).await;

        // Cleanup shouldn't remove recent entries
        limiter.cleanup().await;

        let requests = limiter.requests.read().await;
        assert!(requests.contains_key(&ip), "Recent entry should remain");
    }

    #[tokio::test]
    async fn test_zero_limit_blocks_all() {
        let limiter = RateLimiter::new(0);
        let ip = test_ip();

        assert!(
            !limiter.check(ip).await,
            "Zero limit should block all requests"
        );
    }
}
