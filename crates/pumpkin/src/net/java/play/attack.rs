#[allow(clippy::wildcard_imports)]
use super::*;
use pumpkin_data::attributes::Attributes;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::vector3::Vector3;

impl JavaClient {
    pub fn handle_attack(&self, player: &Arc<Player>, attack: &SAttack, server: &Arc<Server>) {
        if !player.has_client_loaded() {
            return;
        }
        player.update_last_action_time();
        let entity_id = attack.entity_id;
        let player_entity = &player.get_entity();
        let world = player_entity.world.load_full();

        let config = &server.advanced_config.pvp;
        if !config.enabled {
            return;
        }

        if entity_id.0 == player.entity_id() {
            self.try_kick(&TextComponent::translate_cross(
                translation::java::MULTIPLAYER_DISCONNECT_INVALID_ENTITY_ATTACKED,
                translation::java::MULTIPLAYER_DISCONNECT_INVALID_ENTITY_ATTACKED,
                [],
            ));
            return;
        }

        let player_target = world.get_player_by_id(entity_id.0);
        let target: Option<Arc<dyn EntityBase>> = player_target
            .as_ref()
            .map(|p| Arc::clone(p) as Arc<dyn EntityBase>)
            .or_else(|| world.get_entity_by_id(entity_id.0));
        let Some(target) = target else {
            self.try_kick(&TextComponent::translate_cross(
                translation::java::MULTIPLAYER_DISCONNECT_INVALID_ENTITY_ATTACKED,
                translation::java::MULTIPLAYER_DISCONNECT_INVALID_ENTITY_ATTACKED,
                [],
            ));
            return;
        };
        // Vanilla validates the interact packet's range server-side before
        // dispatching the attack (`ServerboundInteractPacket.isWithinRange`
        // with a buffer of 3.0); silently ignore out-of-range attacks.
        let target_bounds = target.get_entity().bounding_box.load();
        if !is_within_entity_interaction_range(player, &target_bounds, 3.0) {
            return;
        }
        if let Some(player_victim) = &player_target {
            if player_victim.living_entity.health.load() <= 0.0 {
                return;
            }
            if config.protect_creative && player_victim.gamemode.load() == GameMode::Creative {
                world.play_sound(
                    Sound::EntityPlayerAttackNodamage,
                    SoundCategory::Players,
                    &player_victim.position(),
                );
                return;
            }
        }
        player.attack(&target);
    }
}

/// Mirrors vanilla `Player.isWithinEntityInteractionRange(AABB, double)`: the
/// squared distance from the eye position to the target's bounding box must be
/// less than the squared range plus buffer. Vanilla calls this from the
/// interact-packet handler with a buffer of 3.0.
fn is_within_entity_interaction_range(
    player: &Player,
    bounding_box: &BoundingBox,
    buffer: f64,
) -> bool {
    let max_range = player
        .living_entity
        .get_attribute_value(&Attributes::ENTITY_INTERACTION_RANGE)
        + buffer;
    let entity_pos = player.living_entity.entity.pos.load();
    let eye_height = player.living_entity.entity.get_eye_height();
    bounding_box.squared_magnitude(Vector3 {
        x: entity_pos.x,
        y: entity_pos.y + eye_height,
        z: entity_pos.z,
    }) < max_range * max_range
}
