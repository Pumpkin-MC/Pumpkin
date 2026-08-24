#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub async fn handle_confirm_teleport(
        &self,
        player: &Player,
        confirm_teleport: SConfirmTeleport,
    ) {
        let mut awaiting_teleport = player.awaiting_teleport.lock().await;
        if let Some((id, position)) = awaiting_teleport.as_ref() {
            if id == &confirm_teleport.teleport_id {
                // We should set the position now to what we requested in the teleport packet.
                // This may fix issues when the client sends the position while being teleported.
                player.get_entity().set_pos(*position);

                *awaiting_teleport = None;
                drop(awaiting_teleport);
            } else {
                drop(awaiting_teleport);
                self.kick(TextComponent::text("Wrong teleport id")).await;
            }
        } else {
            drop(awaiting_teleport);
            self.kick(TextComponent::text(
                "Send Teleport confirm, but we did not teleport",
            ))
            .await;
        }
    }
}
