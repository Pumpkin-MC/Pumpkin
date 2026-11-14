#[allow(clippy::wildcard_imports)]
use super::*;
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
        let Some(target) = world.get_entity_by_id(target_id) else {
            return;
        };

        let entity = target.get_entity();
        let block_pos = entity.block_pos.load().0;
        // Vanilla drops a camera target that is outside the world border.
        if !entity
            .world
            .load()
            .worldborder
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .contains(f64::from(block_pos.x), f64::from(block_pos.z))
        {
            return;
        }

        // Vanilla allows a 3 block buffer beyond the entity interaction range.
        if !player.can_interact_with_entity(target.as_ref(), 3.0)
            || entity.is_removed()
            || !target.can_hit()
        {
            return;
        }

        let target_pos = entity.pos.load();
        let target_yaw = entity.yaw.load();
        let target_pitch = entity.pitch.load();
        let target_id = entity.entity_id;

        player.set_camera_entity_id(target_id);

        player.request_teleport(target_pos, target_yaw, target_pitch);
    }
}
