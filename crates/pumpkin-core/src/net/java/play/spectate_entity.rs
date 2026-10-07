#[allow(clippy::wildcard_imports)]
use super::*;
use pumpkin_data::attributes::Attributes;
use pumpkin_protocol::java::{client::play::CSetCamera, server::play::SSpectateEntity};
use pumpkin_util::GameMode;

impl JavaClient {
    pub fn handle_spectate_entity(
        &self,
        player: &Arc<Player>,
        server: &Server,
        packet: &SSpectateEntity,
    ) {
        if !player.has_client_loaded() {
            return;
        }
        player.update_last_action_time();

        if player.gamemode.load() != GameMode::Spectator {
            return;
        }

        let world = player.world();
        let target: Arc<dyn EntityBase> =
            if let Some(target) = world.get_entity_by_uuid(packet.target) {
                target
            } else if let Some(target) = server.get_player_by_uuid(packet.target) {
                target
            } else {
                return;
            };

        let entity = target.get_entity();
        let block_pos = entity.block_pos.load().0;
        // Vanilla drops a camera target that is outside the world border.
        if !world
            .worldborder
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .contains(f64::from(block_pos.x), f64::from(block_pos.z))
        {
            return;
        }

        // Vanilla allows a 3 block buffer beyond the entity interaction range.
        let max_range = player
            .living_entity
            .get_attribute_value(&Attributes::ENTITY_INTERACTION_RANGE)
            + 3.0;
        if entity
            .bounding_box
            .load()
            .squared_magnitude(player.eye_position())
            >= max_range * max_range
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
        player.try_send_client_packet(&CSetCamera::new(target_id.into()));

        player.request_teleport(target_pos, target_yaw, target_pitch);
    }
}
