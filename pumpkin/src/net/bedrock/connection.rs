use std::{
    io::{Cursor, Error},
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    time::UNIX_EPOCH,
};

use pumpkin_protocol::bedrock::{
    RakReliability,
    client::raknet::connection::{CConnectedPong, CConnectionRequestAccepted},
    server::raknet::connection::{SConnectedPing, SConnectionRequest, SNewIncomingConnection},
};
use pumpkin_protocol::{codec::u24, serial::PacketRead as _};

use crate::net::bedrock::BedrockClient;

impl BedrockClient {
    pub fn is_connection_request(reader: &mut Cursor<&[u8]>) -> Result<SConnectionRequest, Error> {
        //Must be reliable and non split
        if u8::read(reader)? == 0x40 {
            u16::read_be(reader)?;
            //skip reliable seq
            u24::read(reader)?;
            SConnectionRequest::read(reader)
        } else {
            Err(Error::other(""))
        }
    }

    pub async fn handle_connection_request(&self, packet: SConnectionRequest) {
        self.send_framed_packet(
            &CConnectionRequestAccepted::new(
                self.address,
                0,
                [SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 19132)); 10],
                packet.time,
                UNIX_EPOCH.elapsed().unwrap().as_millis() as u64,
            ),
            RakReliability::Unreliable,
        )
        .await;
    }

    pub const fn handle_new_incoming_connection(&self, _packet: &SNewIncomingConnection) {
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
        // TODO Make this cleaner and handle it only with the ClientPlatform
        // This would also help with potential deadlocks by preventing to lock the player
        //self.player.lock().await.clone().map(async |player| {
        //    player.wait_for_keep_alive.store(false, Ordering::Relaxed);
        //    println!("ping procedet");
        //});
    }
}
