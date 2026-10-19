#[allow(clippy::wildcard_imports)]
use super::*;
use pumpkin_protocol::bedrock::network_stack_latency::NetworkStackLatency;
use std::time::{Duration, Instant};

const LATENCY_INTERVAL: Duration = Duration::from_secs(1);
const MAX_PENDING: usize = 4;
const LATENCY_TIMEOUT: Duration = Duration::from_secs(5);
/// Clients have echoed the probe value as is, times 1000 (before 1.20.10) or times 10^9.
const ECHO_SCALES: [u64; 3] = [1, 1_000, 1_000_000_000];

impl BedrockClient {
    /// Probe once the client has finished loading, then once a second.
    pub fn tick_network_latency(&self, player: &Player) {
        if !player.has_client_loaded() {
            return;
        }
        if self.last_latency_send.load().elapsed() < LATENCY_INTERVAL {
            return;
        }
        self.send_latency_probe();
    }

    pub fn send_latency_probe(&self) {
        let mut pending = self
            .pending_latencies
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        while pending
            .front()
            .is_some_and(|(_, sent)| sent.elapsed() >= LATENCY_TIMEOUT)
        {
            pending.pop_front();
        }
        if pending.len() >= MAX_PENDING {
            return;
        }
        let id = self.next_latency_id.fetch_add(1, Ordering::Relaxed);
        pending.push_back((id, Instant::now()));
        drop(pending);
        self.last_latency_send.store(Instant::now());
        self.try_enqueue_client_packet(&NetworkStackLatency {
            timestamp: id,
            from_server: true,
        });
    }

    pub fn handle_network_stack_latency(&self, player: &Player, packet: NetworkStackLatency) {
        let send_time = {
            let mut pending = self
                .pending_latencies
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            // Replies arrive in order, so older unmatched probes were lost.
            pending
                .iter()
                .position(|&(id, _)| {
                    ECHO_SCALES
                        .iter()
                        .any(|&scale| id.wrapping_mul(scale) == packet.timestamp)
                })
                .and_then(|index| pending.drain(..=index).next_back())
                .map(|(_, sent)| sent)
        };
        if let Some(send_time) = send_time {
            let sample = (send_time.elapsed().as_millis() as u32).max(1);
            let previous = player.ping.load(Ordering::Relaxed);
            // 0 stays "unsampled". Integer EMA of a 1 ms first sample would round to 0.
            player
                .ping
                .store(((previous * 3 + sample) / 4).max(1), Ordering::Relaxed);
            return;
        }
        if !packet.from_server {
            self.try_enqueue_client_packet(&packet);
        }
    }
}
