use pumpkin_protocol::{
    bedrock::{
        UDP_HEADER_SIZE,
        client::raknet::open_connection::{COpenConnectionReply1, COpenConnectionReply2},
        server::raknet::open_connection::{SOpenConnectionRequest1, SOpenConnectionRequest2},
    },
    codec::socket_address::SocketAddress,
};

use crate::{
    net::{Client, bedrock::BedrockClientPlatform},
    server::Server,
};

impl Client {
    pub async fn handle_open_connection_1(
        &self,
        bedrock: &BedrockClientPlatform,
        server: &Server,
        packet: SOpenConnectionRequest1,
    ) {
        bedrock
            .send_raknet_packet_now(
                self,
                &COpenConnectionReply1::new(
                    server.server_guid,
                    false,
                    0,
                    packet.mtu + UDP_HEADER_SIZE,
                ),
            )
            .await;
    }
    pub async fn handle_open_connection_2(
        &self,
        bedrock: &BedrockClientPlatform,
        server: &Server,
        packet: SOpenConnectionRequest2,
    ) {
        bedrock
            .send_raknet_packet_now(
                self,
                &COpenConnectionReply2::new(
                    server.server_guid,
                    SocketAddress(*self.address.lock().await),
                    packet.mtu,
                    false,
                ),
            )
            .await;
    }
}
