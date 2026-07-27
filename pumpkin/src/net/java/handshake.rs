use pumpkin_data::{
    packet::{CURRENT_MC_VERSION, LOWEST_SUPPORTED_MC_VERSION},
    translation,
};
use pumpkin_protocol::{ConnectionState, java::server::handshake::SHandShake};
use pumpkin_util::{text::TextComponent, version::JavaMinecraftVersion};
use tracing::debug;

use crate::net::java::JavaClient;
use crate::server::Server;

impl JavaClient {
    pub async fn handle_handshake(&self, server: &Server, handshake: SHandShake) {
        let version = handshake.protocol_version.0 as u32;
        *self.server_address.lock().await = handshake.server_address;
        let parsed_version = JavaMinecraftVersion::from_protocol(version);
        self.version.store(parsed_version);

        debug!("Handshake: next state is {:?}", &handshake.next_state);
        self.connection_state.store(handshake.next_state);
        if self.connection_state.load() != ConnectionState::Status {
            let protocol = version;
            if protocol < LOWEST_SUPPORTED_MC_VERSION.protocol_version() as u32 {
                self.kick(TextComponent::translate_cross(
                    translation::java::MULTIPLAYER_DISCONNECT_OUTDATED_CLIENT,
                    translation::java::MULTIPLAYER_DISCONNECT_OUTDATED_CLIENT,
                    [TextComponent::text(CURRENT_MC_VERSION.to_string())],
                ))
                .await;
            } else if protocol > CURRENT_MC_VERSION.protocol_version() as u32 {
                self.kick(TextComponent::translate_cross(
                    translation::java::MULTIPLAYER_DISCONNECT_OUTDATED_SERVER,
                    translation::java::MULTIPLAYER_DISCONNECT_OUTDATED_SERVER,
                    [TextComponent::text(CURRENT_MC_VERSION.to_string())],
                ))
                .await;
            } else {
                // The client's version is within the range the protocol implementation
                // supports. Now check the operator-configured allow/deny rules.
                let versions_config = &server.advanced_config.networking.java.versions;
                if !versions_config.is_allowed(parsed_version, CURRENT_MC_VERSION) {
                    let message = versions_config.disconnect_message().map_or_else(
                        || versions_config.default_disconnect_message(CURRENT_MC_VERSION),
                        str::to_string,
                    );
                    self.kick(TextComponent::text(message)).await;
                }
            }
        }
    }
}
