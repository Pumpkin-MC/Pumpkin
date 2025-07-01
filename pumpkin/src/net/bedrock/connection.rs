use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    time::UNIX_EPOCH,
};

use pumpkin_protocol::{
    ConnectionState,
    bedrock::{
        RakReliability,
        client::raknet::connection::CConnectionRequestAccepted,
        server::raknet::connection::{SConnectionRequest, SNewIncomingConnection},
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
                    [SocketAddress(SocketAddr::V4(SocketAddrV4::new(
                        Ipv4Addr::new(0, 0, 0, 0),
                        19132,
                    ))); 10],
                    packet.time,
                    UNIX_EPOCH.elapsed().unwrap().as_millis() as u64,
                ),
                RakReliability::Unreliable,
            )
            .await;
    }

    pub fn handle_new_incoming_connection(&self, packet: &SNewIncomingConnection) {
        dbg!(packet.pong_time);
        self.connection_state.store(ConnectionState::Login);
    }
}
