use std::time::UNIX_EPOCH;

use pumpkin_protocol::bedrock::{
    RakReliability, client::raknet::connection::CConnectedPong,
    server::raknet::connection::SConnectedPing,
};

use crate::net::bedrock::BedrockClient;

impl BedrockClient {
    pub async fn handle_connected_ping(&self, packet: SConnectedPing) {
        self.send_framed_packet(
            &CConnectedPong::new(
                packet.time,
                UNIX_EPOCH
                    .elapsed()
                    .map_or(0, |elapsed| elapsed.as_millis() as u64),
            ),
            RakReliability::Unreliable,
        )
        .await;
    }
}
