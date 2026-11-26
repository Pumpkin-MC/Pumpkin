#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub fn handle_confirm_teleport(&self, player: &Player, confirm_teleport: &SConfirmTeleport) {
        let confirm_id = confirm_teleport.teleport_id;
        {
            let mut awaiting_teleport = player
                .awaiting_teleport
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            match *awaiting_teleport {
                Some((id, position)) if id == confirm_id => {
                    player.get_entity().set_pos(position);
                    *awaiting_teleport = None;
                    return;
                }
                Some(_) => return,
                None if confirm_id.0 != player.teleport_id_count.load(Ordering::Relaxed) => return,
                None => {}
            }
        }
        self.try_kick(&TextComponent::text(
            "Send Teleport confirm, but we did not teleport",
        ));
    }
}
