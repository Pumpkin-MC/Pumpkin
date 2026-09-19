#[allow(clippy::wildcard_imports)]
use super::*;
use pumpkin_data::data_component_impl::IDSetContent;
use pumpkin_data::effect::StatusEffect;
use pumpkin_inventory::beacon_screen_handler::BeaconScreenHandler;
use pumpkin_protocol::java::server::play::SSetBeacon;

impl JavaClient {
    /// Mirrors `ServerGamePacketListenerImpl.handleSetBeaconPacket`.
    pub fn handle_set_beacon(&self, player: &Arc<Player>, packet: &SSetBeacon) {
        // `None` = no effect; an id that is not a status effect is a malformed packet.
        let decode = |id: &Option<VarInt>| {
            id.as_ref().map_or(Ok(None), |id| {
                u16::try_from(id.0)
                    .ok()
                    .and_then(<StatusEffect as IDSetContent>::from_id)
                    .map(Some)
                    .ok_or(())
            })
        };
        let valid = match (
            decode(&packet.primary_effect),
            decode(&packet.secondary_effect),
        ) {
            (Ok(primary), Ok(secondary)) => {
                let handler_lock = player
                    .current_screen_handler
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .clone();
                let mut handler = handler_lock
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                let Some(beacon) = handler.as_any_mut().downcast_mut::<BeaconScreenHandler>()
                else {
                    debug!(
                        "Player {} interacted with invalid menu (expected beacon)",
                        player.gameprofile.name
                    );
                    return;
                };
                beacon.update_effects(primary, secondary)
            }
            _ => false,
        };

        if !valid {
            warn!(
                "Player {} tried to set invalid beacon effects: primary {:?}, secondary {:?}",
                player.gameprofile.name, packet.primary_effect, packet.secondary_effect
            );
            self.try_kick(&TextComponent::translate(
                "multiplayer.disconnect.generic",
                &[],
            ));
        }
    }
}
