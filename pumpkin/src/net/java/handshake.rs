use pumpkin_data::{packet::{CURRENT_MC_VERSION, LOWEST_SUPPORTED_MC_VERSION}, translation};
use pumpkin_protocol::{ConnectionState, java::server::handshake::SHandShake};
use pumpkin_util::{text::TextComponent, version::MinecraftVersion};

use crate::net::java::JavaClient;

impl JavaClient {
    pub async fn handle_handshake(&self, handshake: SHandShake) {
        let version = handshake.protocol_version.0 as u32;
        *self.server_address.lock().await = handshake.server_address;
        self.version.store(MinecraftVersion::from_protocol(version));

        log::debug!("Handshake: next state is {:?}", &handshake.next_state);
        self.connection_state.store(handshake.next_state);
        if self.connection_state.load() != ConnectionState::Status {
            let protocol = version;
            if protocol < LOWEST_SUPPORTED_MC_VERSION.protocol_version() as u32 {
                self.kick(TextComponent::translate(
                    translation::MULTIPLAYER_DISCONNECT_OUTDATED_CLIENT,
                    [TextComponent::text(CURRENT_MC_VERSION.to_string())],
                ))
                .await;
            } else if protocol > CURRENT_MC_VERSION.protocol_version() as u32 {
                self.kick(TextComponent::translate(
                    translation::MULTIPLAYER_DISCONNECT_OUTDATED_SERVER,
                    [TextComponent::text(CURRENT_MC_VERSION.to_string())],
                ))
                .await;
            }
        }
    }
}
