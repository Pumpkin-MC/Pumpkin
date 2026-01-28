use pumpkin_data::packet::CURRENT_MC_PROTOCOL;
use pumpkin_protocol::{ConnectionState, java::server::handshake::SHandShake};
use pumpkin_util::{text::TextComponent, version::MinecraftVersion};

use pumpkin_world::{CURRENT_MC_VERSION, LOWEST_SUPPRORTED_PROTOCOL_VERSION};

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
            if protocol < LOWEST_SUPPRORTED_PROTOCOL_VERSION {
                self.kick(TextComponent::translate(
                    "multiplayer.disconnect.outdated_client",
                    [TextComponent::text(CURRENT_MC_VERSION.to_string())],
                ))
                .await;
            } else if protocol > CURRENT_MC_PROTOCOL {
                self.kick(TextComponent::translate(
                    "multiplayer.disconnect.incompatible",
                    [TextComponent::text(CURRENT_MC_VERSION.to_string())],
                ))
                .await;
            }
        }
    }
}
