use pumpkin_protocol::{
    Players,
    java::client::status::{CPingResponse, CStatusResponse},
    java::server::status::SStatusPingRequest,
};

use std::sync::Arc;

use crate::{
    net::java::pending::PendingConnection, plugin::server::list_ping::ServerListPingEvent,
    server::Server,
};
use tracing::debug;

impl PendingConnection {
    pub async fn handle_status_request(&mut self, server: &Arc<Server>) {
        debug!("Handling status request");
        let mut status_response = {
            let status = server.get_status();
            status
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get_status_response(self.version.load().protocol_version())
        };

        let (max_players, num_players) = status_response
            .players
            .as_ref()
            .map_or((0, 0), |players| (players.max, players.online));

        let mut event = ServerListPingEvent::new(
            self.server_address.clone(),
            self.address,
            status_response.description.clone(),
            max_players,
            num_players,
            status_response.favicon.clone(),
        );
        server.plugin_manager.fire(server, &mut event).await;

        status_response.description = event.motd;
        status_response.favicon = event.favicon;
        if let Some(players) = &mut status_response.players {
            players.max = event.max_players;
            players.online = event.num_players;
        } else {
            status_response.players = Some(Players {
                max: event.max_players,
                online: event.num_players,
                sample: vec![],
            });
        }

        let status_json = serde_json::to_string(&status_response).unwrap_or_default();
        self.send_packet_now(&CStatusResponse::new(status_json))
            .await;
    }

    pub async fn handle_ping_request(&mut self, ping_request: SStatusPingRequest) {
        debug!("Handling ping request");
        self.send_packet_now(&CPingResponse::new(ping_request.payload))
            .await;
        self.close();
    }
}
