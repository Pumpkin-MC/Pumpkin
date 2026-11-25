#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub fn handle_confirm_teleport(&self, player: &Player, confirm_teleport: &SConfirmTeleport) {
        let mut awaiting_teleport = player
            .awaiting_teleport
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        // Vanilla `handleAcceptTeleportPacket` acts only when the id matches and
        // ignores every other confirm. Kicking instead disconnects a player as soon
        // as two teleports overlap, because only the newest one is remembered: a
        // refused move teleports them, something teleports them again before the
        // first confirm lands, and that first confirm no longer matches. Dying while
        // moving with `immediate_respawn` on does exactly that.
        let Some((id, position)) = awaiting_teleport.as_ref() else {
            return;
        };
        if id != &confirm_teleport.teleport_id {
            return;
        }

        // We should set the position now to what we requested in the teleport packet.
        // This may fix issues when the client sends the position while being teleported.
        let position = *position;
        *awaiting_teleport = None;
        drop(awaiting_teleport);

        player.get_entity().set_pos(position);
    }
}
