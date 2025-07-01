use std::time::UNIX_EPOCH;

use pumpkin_protocol::{
    bedrock::{
        RakReliability, client::raknet::connection::CConnectionRequestAccepted,
        server::raknet::connection::SConnectionRequest,
    },
    codec::socket_address::SocketAddress,
};

use crate::net::{Client, bedrock::BedrockClientPlatform};

impl Client {
    pub async fn handle_connection_request(
        &self,
        bedrock: &BedrockClientPlatform,
        packet: SConnectionRequest,
    ) {
        dbg!("send connection accepted");
        bedrock
            .send_framed_packet(
                self,
                &CConnectionRequestAccepted::new(
                    SocketAddress(*self.address.lock().await),
                    0,
                    vec![],
                    packet.time,
                    UNIX_EPOCH.elapsed().unwrap().as_millis() as u64,
                ),
                RakReliability::Reliable,
            )
            .await;
    }
}
