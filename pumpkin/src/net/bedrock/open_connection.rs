use std::net::SocketAddr;

use pumpkin_protocol::bedrock::{
    client::raknet::open_connection::{COpenConnectionReply1, COpenConnectionReply2},
    server::raknet::open_connection::{SOpenConnectionRequest1, SOpenConnectionRequest2},
};
use tokio::net::UdpSocket;

use crate::{net::bedrock::BedrockClient, server::Server};

impl BedrockClient {
    pub async fn handle_open_connection_1(
        server: &Server,
        _packet: SOpenConnectionRequest1,
        addr: SocketAddr,
        socket: &UdpSocket,
    ) {
        Self::send_offline_packet(
            &COpenConnectionReply1::new(server.server_guid, false, 1400),
            addr,
            socket,
        )
        .await;
    }
    pub async fn handle_open_connection_2(
        server: &Server,
        packet: SOpenConnectionRequest2,
        addr: SocketAddr,
        socket: &UdpSocket,
    ) {
        Self::send_offline_packet(
            &COpenConnectionReply2::new(server.server_guid, addr, packet.mtu, false),
            addr,
            socket,
        )
        .await;
    }
}
