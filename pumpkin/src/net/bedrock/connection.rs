use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    time::UNIX_EPOCH,
};

use pumpkin_protocol::{
    bedrock::{
        RakReliability,
        client::raknet::connection::{CConnectedPong, CConnectionRequestAccepted},
        server::raknet::connection::{SConnectedPing, SConnectionRequest, SNewIncomingConnection},
    },
    codec::socket_address::SocketAddress,
};

use crate::net::bedrock::BedrockClientPlatform;

impl BedrockClientPlatform {
    pub async fn handle_connection_request(&self, packet: SConnectionRequest) {
        dbg!("send connection accepted");
        self.send_framed_packet(
            &CConnectionRequestAccepted::new(
                SocketAddress(self.address),
                0,
                [SocketAddress(SocketAddr::V4(SocketAddrV4::new(
                    Ipv4Addr::UNSPECIFIED,
                    19132,
                ))); 10],
                packet.time,
                UNIX_EPOCH.elapsed().unwrap().as_millis() as u64,
            ),
            RakReliability::Unreliable,
        )
        .await;
    }

    pub fn handle_new_incoming_connection(&self, _packet: &SNewIncomingConnection) {
        // self.connection_state.store(ConnectionState::Login);
    }

    pub async fn handle_connected_ping(&self, packet: SConnectedPing) {
        self.send_framed_packet(
            &CConnectedPong::new(
                packet.time,
                UNIX_EPOCH.elapsed().unwrap().as_millis() as u64,
            ),
            RakReliability::Unreliable,
        )
        .await;
    }
}
