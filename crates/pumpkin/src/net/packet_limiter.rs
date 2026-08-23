use std::sync::Mutex;
use std::time::Instant;

use pumpkin_config::PacketLimiterConfig;

#[derive(Debug)]
struct LimiterState {
    tokens: f64,
    last_update: Instant,
}

/// Token bucket based rate limiter for incoming client packets.
#[derive(Debug)]
pub struct PacketRateLimiter {
    enabled: bool,
    max_rate: f64,
    burst_capacity: f64,
    state: Mutex<LimiterState>,
}

impl PacketRateLimiter {
    #[must_use]
    pub fn new(enabled: bool, max_rate: f64, burst_capacity: f64) -> Self {
        Self {
            enabled,
            max_rate,
            burst_capacity,
            state: Mutex::new(LimiterState {
                tokens: burst_capacity,
                last_update: Instant::now(),
            }),
        }
    }

    #[must_use]
    pub fn from_config(config: &PacketLimiterConfig) -> Self {
        Self::new(
            config.enabled,
            config.max_packet_rate,
            config.burst_capacity,
        )
    }

    /// Checks whether an incoming packet is allowed under the rate limit.
    ///
    /// Returns `true` if the packet is within limits, or `false` if the rate limit is exceeded.
    #[must_use]
    pub fn check_packet(&self) -> bool {
        if !self.enabled || self.max_rate <= 0.0 {
            return true;
        }

        let now = Instant::now();
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let elapsed = now.duration_since(state.last_update).as_secs_f64();
        state.last_update = now;

        state.tokens = (state.tokens + elapsed * self.max_rate).min(self.burst_capacity);

        if state.tokens >= 1.0 {
            state.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    #[must_use]
    pub const fn max_rate(&self) -> f64 {
        self.max_rate
    }

    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limiter_allows_packets_within_capacity() {
        let limiter = PacketRateLimiter::new(true, 10.0, 5.0);
        for _ in 0..5 {
            assert!(limiter.check_packet());
        }
        // Burst exhausted
        assert!(!limiter.check_packet());
    }

    #[test]
    fn limiter_disabled() {
        let limiter = PacketRateLimiter::new(false, 10.0, 1.0);
        for _ in 0..100 {
            assert!(limiter.check_packet());
        }
    }

    #[test]
    fn limiter_zero_rate() {
        let limiter = PacketRateLimiter::new(true, 0.0, 1.0);
        for _ in 0..100 {
            assert!(limiter.check_packet());
        }
    }
}
