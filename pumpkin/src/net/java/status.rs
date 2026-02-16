use pumpkin_protocol::{
    java::client::status::CPingResponse, java::server::status::SStatusPingRequest,
};

use crate::{net::java::JavaClient, server::Server};
use tracing::debug;

impl JavaClient {
    pub async fn handle_status_request(&self, server: &Server) {
        debug!("Handling status request");
        let status = server.get_status();
        self.send_packet_now(&status.lock().await.get_status())
            .await;
    }

    pub async fn handle_ping_request(&self, ping_request: SStatusPingRequest) {
        debug!("Handling ping request");
        self.send_packet_now(&CPingResponse::new(ping_request.payload))
            .await;
        self.close();
    }
}
