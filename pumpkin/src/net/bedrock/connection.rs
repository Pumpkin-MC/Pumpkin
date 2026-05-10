use std::time::UNIX_EPOCH;

use pumpkin_protocol::bedrock::{
    RakReliability,
    client::raknet::connection::CConnectedPong,
    server::raknet::connection::{SConnectedPing, SNewIncomingConnection},
};

use crate::net::bedrock::BedrockClient;

impl BedrockClient {
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
