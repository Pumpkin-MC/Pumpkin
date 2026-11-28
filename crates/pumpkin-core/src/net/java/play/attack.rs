#[allow(clippy::wildcard_imports)]
use super::*;

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

        let player_target = world.get_player_by_id(entity_id.0);
        let target: Option<Arc<dyn EntityBase>> = player_target
            .as_ref()
            .map(|p| Arc::clone(p) as Arc<dyn EntityBase>)
            .or_else(|| world.get_entity_by_id(entity_id.0));
        // Vanilla ignores a target that is gone, like a cart broken by the previous hit.
        let Some(target) = target else {
            return;
        };
        // TODO: vanilla `handleAttack` ignores spectators, targets outside the world border and
        // `PIERCING_WEAPON` main hands, and checks `isWithinAttackRange` (the weapon's
        // `ATTACK_RANGE`) with a 3.0 buffer.
        if !player.can_interact_with_entity(target.as_ref(), 1.0) {
            return;
        }
        if is_invalid_attack_target(player, target.as_ref()) {
            self.try_kick(&TextComponent::translate_cross(
                translation::java::MULTIPLAYER_DISCONNECT_INVALID_ENTITY_ATTACKED,
                translation::java::MULTIPLAYER_DISCONNECT_INVALID_ENTITY_ATTACKED,
                [],
            ));
            warn!(
                "Player {} tried to attack an invalid entity",
                player.gameprofile.name
            );
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

/// Vanilla `handleAttack`: items, experience orbs, the player itself and arrows that can't be
/// hit are never valid targets.
fn is_invalid_attack_target(player: &Player, target: &dyn EntityBase) -> bool {
    let target_entity = target.get_entity();
    let entity_type = target_entity.entity_type;
    target_entity.entity_id == player.entity_id()
        || entity_type == &pumpkin_data::entity::EntityType::ITEM
        || entity_type == &pumpkin_data::entity::EntityType::EXPERIENCE_ORB
        || (crate::entity::projectile::abstract_arrow::is_abstract_arrow(entity_type)
            && !pumpkin_data::tag::Taggable::has_tag(
                entity_type,
                &pumpkin_data::tag::EntityType::MINECRAFT_REDIRECTABLE_PROJECTILE,
            ))
}
