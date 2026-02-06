use pumpkin_protocol::{
    Players,
    java::client::status::{CPingResponse, CStatusResponse},
    java::server::status::SStatusPingRequest,
};

use crate::{net::java::JavaClient, plugin::server::server_list_ping::ServerListPingEvent, server::Server};
use tracing::debug;

impl JavaClient {
    pub async fn handle_status_request(&self, server: &Server) {
        debug!("Handling status request");
        let status = server.get_status();
        let mut status_response = {
            let status_guard = status.lock().await;
            status_guard.status_response.clone()
        };

        let (max_players, num_players) = status_response
            .players
            .as_ref()
            .map_or((0, 0), |players| (players.max, players.online));

        let event = ServerListPingEvent::new(
            status_response.description.clone(),
            max_players,
            num_players,
            status_response.favicon.clone(),
        );
        let event = server.plugin_manager.fire(event).await;

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

        let status_json = serde_json::to_string(&status_response)
            .unwrap_or_else(|_| "{}".to_string());
        self.send_packet_now(&CStatusResponse::new(status_json)).await;
    }

    pub async fn handle_ping_request(&self, ping_request: SStatusPingRequest) {
        debug!("Handling ping request");
        self.send_packet_now(&CPingResponse::new(ping_request.payload))
            .await;
        self.close();
    }
}
