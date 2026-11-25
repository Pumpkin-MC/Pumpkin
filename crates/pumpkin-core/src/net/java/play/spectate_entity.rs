#[allow(clippy::wildcard_imports)]
use super::*;
use crate::plugin::api::events::player::player_teleport::TeleportCause;
use pumpkin_protocol::java::server::play::SSpectateEntity;
use pumpkin_util::GameMode;

impl JavaClient {
    pub fn handle_spectate_entity(&self, player: &Arc<Player>, packet: &SSpectateEntity) {
        if !player.has_client_loaded() {
            return;
        }
        player.update_last_action_time();

        if player.gamemode.load() != GameMode::Spectator {
            return;
        }

        let Some(target_id) = packet.target_entity_id() else {
            return;
        };

        let world = player.world();
        // TODO: vanilla setCamera checks world border, 3.0 interaction range and isPickable,
        // acts only on a camera change and keeps the player's own yaw/pitch
        if let Some(target) = world.get_entity_by_id(target_id) {
            let target_pos = target.get_entity().pos.load();
            let target_yaw = target.get_entity().yaw.load();
            let target_pitch = target.get_entity().pitch.load();
            let target_id = target.get_entity().entity_id;

            if player.request_teleport_to(
                target_pos,
                Some(target_yaw),
                Some(target_pitch),
                TeleportCause::Spectate,
            ) {
                player.set_camera_entity_id(target_id);
            }
        }
    }
}
