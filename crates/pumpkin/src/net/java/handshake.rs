use pumpkin_data::{
    packet::{CURRENT_MC_VERSION, LOWEST_SUPPORTED_MC_VERSION},
    translation,
};
use pumpkin_protocol::{ConnectionState, java::server::handshake::SHandShake};
use pumpkin_util::{text::TextComponent, version::JavaMinecraftVersion};
use tracing::debug;

use crate::{net::java::pending::PendingConnection, server::Server};
use std::sync::Arc;

/// Known older Java versions are admitted when the multiversion plugin is loaded.
pub(super) fn multiversion_admits(server: &Server, version: JavaMinecraftVersion) -> bool {
    version != JavaMinecraftVersion::Unknown
        && server
            .plugin_manager
            .has_handlers::<crate::plugin::server::packet::ConnectionPacketReceivedEvent>()
}

impl PendingConnection {
    pub async fn handle_handshake(&mut self, server: &Arc<Server>, handshake: SHandShake) {
        let version = handshake.protocol_version.0 as u32;
        self.server_address = handshake.server_address.to_string();
        self.version
            .store(JavaMinecraftVersion::from_protocol(version));

        debug!("Handshake: next state is {:?}", &handshake.next_state);
        self.connection_state.store(handshake.next_state);
        if handshake.next_state == ConnectionState::Transfer
            && !server.basic_config.accepts_transfers
        {
            self.kick(TextComponent::translate_cross(
                translation::java::MULTIPLAYER_DISCONNECT_TRANSFERS_DISABLED,
                translation::java::MULTIPLAYER_DISCONNECT_TRANSFERS_DISABLED,
                [],
            ))
            .await;
            return;
        }
        if self.connection_state.load() != ConnectionState::Status {
            let protocol = version;
            let allows_client =
                multiversion_admits(server, JavaMinecraftVersion::from_protocol(version));

            if !allows_client && protocol < LOWEST_SUPPORTED_MC_VERSION.protocol_version() as u32 {
                self.kick(TextComponent::translate_cross(
                    translation::java::MULTIPLAYER_DISCONNECT_OUTDATED_CLIENT,
                    translation::bedrock::DISCONNECTIONSCREEN_OUTDATEDCLIENT,
                    [TextComponent::text(CURRENT_MC_VERSION.to_string())],
                ))
                .await;
            } else if protocol > CURRENT_MC_VERSION.protocol_version() as u32 {
                self.kick(TextComponent::translate_cross(
                    translation::java::MULTIPLAYER_DISCONNECT_OUTDATED_SERVER,
                    translation::bedrock::DISCONNECTIONSCREEN_OUTDATEDSERVER,
                    [TextComponent::text(CURRENT_MC_VERSION.to_string())],
                ))
                .await;
            }
        }
    }
}
