//! In-memory ban list for anti-cheat system.
//!
//! Stores temporary bans by IP address. No persistence - bans are cleared on server restart.
//! This is intentional for MVP to keep the system simple.

use std::collections::HashMap;
use std::net::IpAddr;
use std::time::{Duration, Instant};

/// A single ban entry.
#[derive(Debug, Clone)]
pub struct BanEntry {
    /// IP address that is banned
    pub ip: IpAddr,
    /// Reason for the ban
    pub reason: String,
    /// When this ban expires
    pub expires_at: Instant,
}

impl BanEntry {
    /// Create a new ban entry
    pub fn new(ip: IpAddr, reason: String, duration: Duration) -> Self {
        Self {
            ip,
            reason,
            expires_at: Instant::now() + duration,
        }
    }

    /// Check if this ban has expired
    pub fn is_expired(&self) -> bool {
        Instant::now() >= self.expires_at
    }

    /// Get remaining ban duration (or zero if expired)
    pub fn remaining(&self) -> Duration {
        self.expires_at.saturating_duration_since(Instant::now())
    }
}

/// In-memory ban list storing temporary bans by IP.
#[derive(Debug, Default)]
pub struct BanList {
    /// Banned IPs mapped to their ban entries
    bans: HashMap<IpAddr, BanEntry>,
}

impl BanList {
    /// Create a new empty ban list
    pub fn new() -> Self {
        Self {
            bans: HashMap::new(),
        }
    }

    /// Check if an IP is currently banned.
    /// Returns the ban entry if banned (and not expired), None otherwise.
    pub fn is_banned(&self, ip: IpAddr) -> Option<&BanEntry> {
        self.bans.get(&ip).filter(|entry| !entry.is_expired())
    }

    /// Add a ban for an IP address.
    pub fn add_ban(&mut self, ip: IpAddr, reason: String, duration: Duration) {
        let entry = BanEntry::new(ip, reason, duration);
        self.bans.insert(ip, entry);
    }

    /// Remove expired bans from the list.
    /// Call periodically (e.g., once per minute) to clean up memory.
    pub fn cleanup_expired(&mut self) {
        self.bans.retain(|_, entry| !entry.is_expired());
    }

    /// Get the number of active (non-expired) bans
    pub fn active_count(&self) -> usize {
        self.bans.values().filter(|e| !e.is_expired()).count()
    }

    /// Remove a specific ban (for admin unbans)
    pub fn remove_ban(&mut self, ip: IpAddr) -> Option<BanEntry> {
        self.bans.remove(&ip)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    fn test_ip() -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
    }

    #[test]
    fn test_new_ban_list() {
        let list = BanList::new();
        assert_eq!(list.active_count(), 0);
    }

    #[test]
    fn test_add_and_check_ban() {
        let mut list = BanList::new();
        let ip = test_ip();

        assert!(list.is_banned(ip).is_none());

        list.add_ban(ip, "Test reason".to_string(), Duration::from_secs(3600));

        let entry = list.is_banned(ip).expect("Should be banned");
        assert_eq!(entry.reason, "Test reason");
        assert!(!entry.is_expired());
    }

    #[test]
    fn test_expired_ban_not_returned() {
        let mut list = BanList::new();
        let ip = test_ip();

        // Add a ban that expires immediately
        list.add_ban(ip, "Test".to_string(), Duration::from_secs(0));

        // Should not be considered banned
        assert!(list.is_banned(ip).is_none());
    }

    #[test]
    fn test_cleanup_expired() {
        let mut list = BanList::new();
        let ip1 = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));
        let ip2 = IpAddr::V4(Ipv4Addr::new(2, 2, 2, 2));

        // Add one expired ban and one active ban
        list.add_ban(ip1, "Expired".to_string(), Duration::from_secs(0));
        list.add_ban(ip2, "Active".to_string(), Duration::from_secs(3600));

        // Before cleanup, both entries exist in the map
        assert_eq!(list.bans.len(), 2);

        list.cleanup_expired();

        // After cleanup, only active ban remains
        assert_eq!(list.bans.len(), 1);
        assert!(list.is_banned(ip2).is_some());
    }

    #[test]
    fn test_remove_ban() {
        let mut list = BanList::new();
        let ip = test_ip();

        list.add_ban(ip, "Test".to_string(), Duration::from_secs(3600));
        assert!(list.is_banned(ip).is_some());

        list.remove_ban(ip);
        assert!(list.is_banned(ip).is_none());
    }
}
