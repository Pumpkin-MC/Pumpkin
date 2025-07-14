use pumpkin_protocol::bedrock::{
    client::raknet::open_connection::{COpenConnectionReply1, COpenConnectionReply2},
    server::raknet::open_connection::{SOpenConnectionRequest1, SOpenConnectionRequest2},
};

use crate::{net::bedrock::BedrockClientPlatform, server::Server};

impl BedrockClientPlatform {
    pub async fn handle_open_connection_1(
        &self,
        server: &Server,
        _packet: SOpenConnectionRequest1,
    ) {
        self.send_raknet_packet_now(&COpenConnectionReply1::new(server.server_guid, false, 1400))
            .await;
    }
    pub async fn handle_open_connection_2(&self, server: &Server, packet: SOpenConnectionRequest2) {
        self.send_raknet_packet_now(&COpenConnectionReply2::new(
            server.server_guid,
            self.address,
            packet.mtu,
            false,
        ))
        .await;
    }
}
