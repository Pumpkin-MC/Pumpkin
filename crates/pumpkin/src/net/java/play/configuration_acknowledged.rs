#[allow(clippy::wildcard_imports)]
use super::*;
use pumpkin_protocol::ConnectionState;

impl JavaClient {
    pub fn handle_configuration_acknowledged(&self) {
        if !self.user.acknowledge_state(ConnectionState::Config) {
            self.try_kick(&TextComponent::text(
                "Unexpected configuration acknowledgement",
            ));
            return;
        }
        debug!(
            "Player {} acknowledged configuration switch",
            self.gameprofile.name
        );
        self.connection_state.store(ConnectionState::Config);
    }
}
