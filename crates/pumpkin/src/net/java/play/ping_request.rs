#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub async fn handle_play_ping_request(&self, request: SPlayPingRequest) {
        self.enqueue_client_packet(&CPingResponse::new(request.payload))
            .await;
    }
}
