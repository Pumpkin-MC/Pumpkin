#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub fn handle_confirm_teleport(&self, player: &Player, confirm_teleport: &SConfirmTeleport) {
        enum TeleportResult {
            Success,
            /// Confirms an older teleport than the last one sent.
            Stale,
            NotTeleporting,
        }

        let result = {
            let mut awaiting_teleport = player
                .awaiting_teleport
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some((id, position)) = awaiting_teleport.as_ref() {
                if id == &confirm_teleport.teleport_id {
                    // We should set the position now to what we requested in the teleport packet.
                    // This may fix issues when the client sends the position while being teleported.
                    player.get_entity().set_pos(*position);
                    *awaiting_teleport = None;
                    TeleportResult::Success
                } else {
                    TeleportResult::Stale
                }
            } else if confirm_teleport.teleport_id.0
                == player.teleport_id_count.load(Ordering::Relaxed)
            {
                TeleportResult::NotTeleporting
            } else {
                TeleportResult::Stale
            }
        };

        match result {
            // Vanilla `handleAcceptTeleportPacket` only acts on the id of the last teleport sent.
            // The client confirms every teleport, so a confirm for an older one arrives whenever a
            // new teleport went out before the client answered.
            TeleportResult::Success | TeleportResult::Stale => {}
            TeleportResult::NotTeleporting => {
                self.try_kick(&TextComponent::text(
                    "Send Teleport confirm, but we did not teleport",
                ));
            }
        }
    }
}
