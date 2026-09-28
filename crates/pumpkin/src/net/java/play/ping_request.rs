#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub fn handle_play_ping_request(&self, request: &SPlayPingRequest) {
        self.try_send_packet(&CPingResponse::new(request.payload));
        // Vanilla answers on the netty thread, so the pong flushes even mid-tick
        self.flush_channel();
    }
}
