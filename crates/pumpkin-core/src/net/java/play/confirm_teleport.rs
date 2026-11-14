#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub fn handle_confirm_teleport(&self, player: &Player, confirm_teleport: &SConfirmTeleport) {
        // Vanilla `handleAcceptTeleportPacket`: only the id of the latest
        // teleport counts. Older ids are stale and ignored.
        if confirm_teleport.teleport_id.0 != player.teleport_id_count.load(Ordering::Relaxed) {
            return;
        }
        let invalid = {
            let mut awaiting_teleport = player
                .awaiting_teleport
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            match awaiting_teleport.take() {
                Some((_, position))
                    if !confirm_teleport.position.x.is_nan()
                        && !confirm_teleport.position.y.is_nan()
                        && !confirm_teleport.position.z.is_nan()
                        && confirm_teleport.yaw.is_finite()
                        && confirm_teleport.pitch.is_finite() =>
                {
                    player.get_entity().set_pos(position);
                    player.is_changing_dimension.store(false, Ordering::Relaxed);
                    false
                }
                _ => true,
            }
        };
        if invalid {
            self.try_kick(&TextComponent::translate_cross(
                translation::java::MULTIPLAYER_DISCONNECT_INVALID_PLAYER_MOVEMENT,
                translation::java::MULTIPLAYER_DISCONNECT_INVALID_PLAYER_MOVEMENT,
                [],
            ));
        }
    }
}
