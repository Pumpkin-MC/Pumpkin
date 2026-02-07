use pumpkin_protocol::{
    Players, StatusResponse, Version,
    java::client::status::{CPingResponse, CStatusResponse},
    java::server::status::SStatusPingRequest,
};

use crate::{
    net::java::JavaClient,
    plugin::api::events::Payload,
    plugin::api::events::server::server_list_ping::ServerListPingEvent,
    server::Server,
};

impl JavaClient {
    pub async fn handle_status_request(&self, server: &Server) {
        log::debug!("Handling status request");
        let status_lock = server.get_status();
        let cached = status_lock.lock().await;
        let response = &cached.status_response;

        let motd = response.description.clone();
        let (max_players, online_players) = response
            .players
            .as_ref()
            .map_or((0, 0), |p| (p.max, p.online));
        let (version_name, protocol_version) = response
            .version
            .as_ref()
            .map_or_else(|| (String::new(), 0), |v| (v.name.clone(), v.protocol));
        let favicon = response.favicon.clone();
        drop(cached);

        let event = server
            .plugin_manager
            .fire(ServerListPingEvent::new(
                motd,
                max_players,
                online_players,
                version_name,
                protocol_version,
            ))
            .await;

        if event.is_cancelled() {
            return;
        }

        // Rebuild status JSON with potentially modified values from plugins
        let status_json = serde_json::to_string(&StatusResponse {
            version: Some(Version {
                name: event.version_name,
                protocol: event.protocol_version,
            }),
            players: Some(Players {
                max: event.max_players,
                online: event.online_players,
                sample: vec![],
            }),
            description: event.motd,
            favicon,
            enforce_secure_chat: true,
        })
        .expect("Failed to serialize status response");

        self.send_packet_now(&CStatusResponse::new(status_json))
            .await;
    }

    pub async fn handle_ping_request(&self, ping_request: SStatusPingRequest) {
        log::debug!("Handling ping request");
        self.send_packet_now(&CPingResponse::new(ping_request.payload))
            .await;
        self.close();
    }
}
