//! Rate limiting module for protecting against brute force and DoS attacks.
//!
//! Provides a generic rate limiter that can be used for:
//! - RCON authentication attempts
//! - Packet rate limiting
//! - Connection rate limiting
//! - Command rate limiting

use std::collections::HashMap;
use std::net::IpAddr;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Entry tracking requests from a specific IP
#[derive(Debug, Clone)]
struct RateLimitEntry {
    /// Number of requests in current window
    count: u32,
    /// Start of the current time window
    window_start: Instant,
}

/// A thread-safe rate limiter that tracks requests per IP address.
///
/// Features:
/// - Configurable request limit per time window
/// - Automatic IP blocking after exceeding limits
/// - Automatic cleanup of stale entries
pub struct RateLimiter {
    /// Request counts per IP
    requests: RwLock<HashMap<IpAddr, RateLimitEntry>>,
    /// Blocked IPs with unblock time
    blocked: RwLock<HashMap<IpAddr, Instant>>,
    /// Maximum requests allowed in the time window
    max_requests: u32,
    /// Time window duration
    window: Duration,
    /// How long to block an IP after exceeding limits
    block_duration: Duration,
}

impl RateLimiter {
    /// Creates a new rate limiter.
    ///
    /// # Arguments
    /// * `max_requests` - Maximum number of requests allowed per window
    /// * `window_secs` - Time window in seconds
    /// * `block_secs` - How long to block IPs that exceed the limit
    #[must_use]
    pub fn new(max_requests: u32, window_secs: u64, block_secs: u64) -> Self {
        Self {
            requests: RwLock::new(HashMap::new()),
            blocked: RwLock::new(HashMap::new()),
            max_requests,
            window: Duration::from_secs(window_secs),
            block_duration: Duration::from_secs(block_secs),
        }
    }

    /// Checks if an IP is allowed to make a request.
    ///
    /// Returns `true` if the request is allowed, `false` if blocked or rate limited.
    pub async fn check(&self, ip: &IpAddr) -> bool {
        // First check if IP is blocked
        if self.is_blocked(ip).await {
            return false;
        }

        // Check current request count
        let requests = self.requests.read().await;
        if let Some(entry) = requests.get(ip) {
            let now = Instant::now();
            // If within the same window, check count
            if now.duration_since(entry.window_start) < self.window {
                return entry.count < self.max_requests;
            }
        }
        true
    }

    /// Records a request from an IP address.
    ///
    /// If the IP exceeds the rate limit, it will be automatically blocked.
    pub async fn record(&self, ip: &IpAddr) {
        let now = Instant::now();
        let mut requests = self.requests.write().await;

        let entry = requests.entry(*ip).or_insert(RateLimitEntry {
            count: 0,
            window_start: now,
        });

        // Reset window if expired
        if now.duration_since(entry.window_start) >= self.window {
            entry.count = 0;
            entry.window_start = now;
        }

        entry.count += 1;

        // Block if exceeded
        if entry.count >= self.max_requests {
            drop(requests); // Release lock before acquiring another
            self.block(ip).await;
        }
    }

    /// Checks if an IP is currently blocked.
    pub async fn is_blocked(&self, ip: &IpAddr) -> bool {
        let blocked = self.blocked.read().await;
        if let Some(unblock_time) = blocked.get(ip) {
            if Instant::now() < *unblock_time {
                return true;
            }
        }
        false
    }

    /// Blocks an IP address for the configured block duration.
    pub async fn block(&self, ip: &IpAddr) {
        let unblock_time = Instant::now() + self.block_duration;
        let mut blocked = self.blocked.write().await;
        blocked.insert(*ip, unblock_time);
        log::warn!(
            "Rate limiter: Blocked IP {} for {:?}",
            ip,
            self.block_duration
        );
    }

    /// Cleans up expired entries to prevent memory growth.
    ///
    /// Should be called periodically (e.g., every minute).
    pub async fn cleanup(&self) {
        let now = Instant::now();

        // Cleanup expired request entries
        {
            let mut requests = self.requests.write().await;
            requests.retain(|_, entry| now.duration_since(entry.window_start) < self.window * 2);
        }

        // Cleanup expired blocks
        {
            let mut blocked = self.blocked.write().await;
            blocked.retain(|_, unblock_time| now < *unblock_time);
        }
    }

    /// Resets the rate limiter state for an IP (useful for testing).
    #[cfg(test)]
    pub async fn reset(&self, ip: &IpAddr) {
        self.requests.write().await.remove(ip);
        self.blocked.write().await.remove(ip);
    }

    /// Gets the current request count for an IP (useful for testing/monitoring).
    pub async fn get_count(&self, ip: &IpAddr) -> u32 {
        let requests = self.requests.read().await;
        requests.get(ip).map(|e| e.count).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[tokio::test]
    async fn test_basic_rate_limiting() {
        let limiter = RateLimiter::new(3, 60, 300);
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));

        // First 3 requests should be allowed
        assert!(limiter.check(&ip).await);
        limiter.record(&ip).await;
        assert!(limiter.check(&ip).await);
        limiter.record(&ip).await;
        assert!(limiter.check(&ip).await);
        limiter.record(&ip).await;

        // 4th request should be blocked
        assert!(!limiter.check(&ip).await);
    }

    #[tokio::test]
    async fn test_different_ips_independent() {
        let limiter = RateLimiter::new(2, 60, 300);
        let ip1 = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        let ip2 = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2));

        // Exhaust ip1's limit
        limiter.record(&ip1).await;
        limiter.record(&ip1).await;

        // ip2 should still be allowed
        assert!(limiter.check(&ip2).await);
    }

    #[tokio::test]
    async fn test_blocking() {
        let limiter = RateLimiter::new(1, 60, 1); // 1 second block
        let ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));

        // Exceed limit
        limiter.record(&ip).await;

        // Should be blocked
        assert!(limiter.is_blocked(&ip).await);
        assert!(!limiter.check(&ip).await);

        // Wait for block to expire
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Should be unblocked
        assert!(!limiter.is_blocked(&ip).await);
    }

    #[tokio::test]
    async fn test_cleanup() {
        let limiter = RateLimiter::new(10, 1, 1); // 1 second window
        let ip = IpAddr::V4(Ipv4Addr::new(172, 16, 0, 1));

        limiter.record(&ip).await;
        assert_eq!(limiter.get_count(&ip).await, 1);

        // Wait for window to expire
        tokio::time::sleep(Duration::from_secs(3)).await;
        limiter.cleanup().await;

        // Entry should be cleaned up
        assert_eq!(limiter.get_count(&ip).await, 0);
    }

    /// Property test: For any IP that exceeds max_requests, it should be blocked
    /// **Feature: security-hardening, Property 2: RCON Rate Limiting**
    /// **Validates: Requirements 2.3, 2.5**
    #[tokio::test]
    async fn test_property_exceeding_limit_blocks_ip() {
        // Test with various max_requests values
        for max_requests in [1u32, 3, 5, 10] {
            let limiter = RateLimiter::new(max_requests, 60, 300);

            // Generate random IP
            let ip = IpAddr::V4(Ipv4Addr::new(
                rand::random(),
                rand::random(),
                rand::random(),
                rand::random(),
            ));

            // Make exactly max_requests
            for _ in 0..max_requests {
                limiter.record(&ip).await;
            }

            // IP should now be blocked
            assert!(
                limiter.is_blocked(&ip).await,
                "IP should be blocked after {} requests",
                max_requests
            );
            assert!(
                !limiter.check(&ip).await,
                "check() should return false for blocked IP"
            );
        }
    }

    /// Property test: Different IPs should have independent rate limits
    #[tokio::test]
    async fn test_property_ip_independence() {
        let limiter = RateLimiter::new(5, 60, 300);

        // Generate 10 random IPs
        let ips: Vec<IpAddr> = (0..10)
            .map(|_| {
                IpAddr::V4(Ipv4Addr::new(
                    rand::random(),
                    rand::random(),
                    rand::random(),
                    rand::random(),
                ))
            })
            .collect();

        // Exhaust first IP's limit
        for _ in 0..5 {
            limiter.record(&ips[0]).await;
        }

        // All other IPs should still be allowed
        for ip in &ips[1..] {
            assert!(
                limiter.check(ip).await,
                "Other IPs should not be affected by first IP's rate limit"
            );
        }
    }

    /// Property test: Requests under limit should always be allowed
    #[tokio::test]
    async fn test_property_under_limit_allowed() {
        let max_requests = 10u32;
        let limiter = RateLimiter::new(max_requests, 60, 300);
        let ip = IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4));

        // Make requests up to but not exceeding limit
        for i in 0..(max_requests - 1) {
            assert!(
                limiter.check(&ip).await,
                "Request {} should be allowed (under limit)",
                i
            );
            limiter.record(&ip).await;
        }

        // Last request before limit should still be allowed
        assert!(
            limiter.check(&ip).await,
            "Request at limit-1 should still be allowed"
        );
    }

    /// Property test: For any IP exceeding 10 connections per second, the IP SHALL be temporarily blocked.
    /// **Feature: security-hardening, Property 4: Connection Rate Limit**
    /// **Validates: Requirements 4.4, 4.5**
    #[tokio::test]
    async fn test_property_connection_rate_limit() {
        // Use the same configuration as the server's connection rate limiter
        // CONNECTION_RATE_LIMIT_MAX = 10, window = 1 second, block = 60 seconds
        let max_connections_per_second = 10u32;
        let window_secs = 1u64;
        let block_secs = 60u64;

        // Test with 100 random IPs
        for _ in 0..100 {
            let limiter = RateLimiter::new(max_connections_per_second, window_secs, block_secs);

            // Generate random IP
            let ip = IpAddr::V4(Ipv4Addr::new(
                rand::random(),
                rand::random(),
                rand::random(),
                rand::random(),
            ));

            // First 10 connections should be allowed
            for i in 0..max_connections_per_second {
                assert!(
                    limiter.check(&ip).await,
                    "Connection {} should be allowed (under limit)",
                    i
                );
                limiter.record(&ip).await;
            }

            // 11th connection should be blocked
            assert!(
                !limiter.check(&ip).await,
                "Connection after limit should be rejected"
            );

            // IP should be blocked
            assert!(
                limiter.is_blocked(&ip).await,
                "IP should be blocked after exceeding connection rate limit"
            );
        }
    }

    /// Property test: For any IP under the connection rate limit, connections SHALL be allowed.
    /// **Feature: security-hardening, Property 4: Connection Rate Limit**
    /// **Validates: Requirements 4.4**
    #[tokio::test]
    async fn test_property_connection_under_limit_allowed() {
        let max_connections_per_second = 10u32;
        let limiter = RateLimiter::new(max_connections_per_second, 1, 60);

        // Test with 50 random IPs, each making fewer than max connections
        for _ in 0..50 {
            let ip = IpAddr::V4(Ipv4Addr::new(
                rand::random(),
                rand::random(),
                rand::random(),
                rand::random(),
            ));

            // Random number of connections under the limit
            let num_connections = rand::random::<u32>() % max_connections_per_second;

            for _ in 0..num_connections {
                assert!(
                    limiter.check(&ip).await,
                    "Connection should be allowed when under limit"
                );
                limiter.record(&ip).await;
            }

            // Should still be allowed (not blocked)
            assert!(
                !limiter.is_blocked(&ip).await,
                "IP should not be blocked when under connection rate limit"
            );
        }
    }

    /// Property test: Blocked IPs SHALL remain blocked for the configured duration.
    /// **Feature: security-hardening, Property 4: Connection Rate Limit**
    /// **Validates: Requirements 4.5**
    #[tokio::test]
    async fn test_property_connection_block_duration() {
        // Use a short block duration for testing
        let block_secs = 1u64;
        let limiter = RateLimiter::new(1, 1, block_secs);

        let ip = IpAddr::V4(Ipv4Addr::new(
            rand::random(),
            rand::random(),
            rand::random(),
            rand::random(),
        ));

        // Exceed the limit to trigger blocking
        limiter.record(&ip).await;

        // Should be blocked immediately
        assert!(
            limiter.is_blocked(&ip).await,
            "IP should be blocked after exceeding limit"
        );
        assert!(!limiter.check(&ip).await, "Blocked IP should fail check");

        // Wait for block to expire
        tokio::time::sleep(Duration::from_secs(block_secs + 1)).await;

        // Should be unblocked after duration
        assert!(
            !limiter.is_blocked(&ip).await,
            "IP should be unblocked after block duration expires"
        );
    }
}
