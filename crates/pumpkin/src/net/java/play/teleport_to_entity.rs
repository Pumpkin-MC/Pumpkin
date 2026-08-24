#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub async fn handle_teleport_to_entity(
        &self,
        player: &Arc<Player>,
        packet: STeleportToEntity,
        server: &Server,
    ) {
        if !player.has_client_loaded() {
            return;
        }
        player.update_last_action_time();

        if player.gamemode.load() != GameMode::Spectator {
            return;
        }

        if let Some(target_player) = server.get_player_by_uuid(packet.target) {
            let target_pos = target_player.living_entity.entity.pos.load();
            let target_yaw = target_player.living_entity.entity.yaw.load();
            let target_pitch = target_player.living_entity.entity.pitch.load();

            let target_id = target_player.living_entity.entity.entity_id;
            player.camera_target_id.store(Some(target_id));
            player
                .send_client_packet(&CSetCamera::new(target_id.into()))
                .await;

            player
                .request_teleport(target_pos, target_yaw, target_pitch)
                .await;
        }
    }
}
