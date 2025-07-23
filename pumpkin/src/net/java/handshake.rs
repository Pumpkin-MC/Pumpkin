use std::sync::atomic::Ordering;

use pumpkin_data::packet::CURRENT_MC_PROTOCOL;
use pumpkin_protocol::{ConnectionState, java::server::handshake::SHandShake};
use pumpkin_util::text::TextComponent;

use crate::{net::java::JavaClient, server::CURRENT_MC_VERSION};

impl JavaClient {
    pub async fn handle_handshake(&self, handshake: SHandShake) {
        let version = handshake.protocol_version.0;
        self.protocol_version.store(version, Ordering::Relaxed);
        *self.server_address.lock().await = handshake.server_address;

        log::debug!("Handshake: next state is {:?}", &handshake.next_state);
        self.connection_state.store(handshake.next_state);
        if self.connection_state.load() != ConnectionState::Status {
            let protocol = version;
            match protocol.cmp(&(CURRENT_MC_PROTOCOL as i32)) {
                std::cmp::Ordering::Less => {
                    self.kick(TextComponent::translate(
                        "multiplayer.disconnect.outdated_client",
                        [TextComponent::text(CURRENT_MC_VERSION.to_string())],
                    ))
                    .await;
                }
                std::cmp::Ordering::Equal => {}
                std::cmp::Ordering::Greater => {
                    self.kick(TextComponent::translate(
                        "multiplayer.disconnect.incompatible",
                        [TextComponent::text(CURRENT_MC_VERSION.to_string())],
                    ))
                    .await;
                }
            }
        }
    }
}
