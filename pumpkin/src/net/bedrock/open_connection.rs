use pumpkin_protocol::{
    bedrock::{
        client::open_connection::{COpenConnectionReply1, COpenConnectionReply2},
        server::open_connection::{SOpenConnectionRequest1, SOpenConnectionRequest2},
    },
    codec::socket_address::SocketAddress,
};

use crate::{net::Client, server::Server};

impl Client {
    pub async fn handle_open_connection_1(&self, server: &Server, packet: SOpenConnectionRequest1) {
        self.send_packet_now(&COpenConnectionReply1::new(
            server.server_guid,
            false,
            0,
            packet.mtu,
        ))
        .await;
    }
    pub async fn handle_open_connection_2(&self, server: &Server, packet: SOpenConnectionRequest2) {
        self.send_packet_now(&COpenConnectionReply2::new(
            server.server_guid,
            SocketAddress(*self.address.lock().await),
            packet.mtu,
            false,
        ))
        .await;
    }
}
