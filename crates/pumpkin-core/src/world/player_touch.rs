use std::sync::Arc;

use pumpkin_data::entity::EntityType;
use pumpkin_util::math::boundingbox::BoundingBox;
use rand::RngExt;

use crate::entity::{EntityBase, player::Player};

use super::World;

/// Player box, joined with a live vehicle's box; `true` when mounted.
pub(super) fn pickup_body(player: &Player) -> (BoundingBox, bool) {
    let entity = player.get_entity();
    let hitbox = entity.bounding_box.load();
    match entity.get_vehicle() {
        Some(vehicle) if !vehicle.get_entity().is_removed() => (
            hitbox.minmax(&vehicle.get_entity().bounding_box.load()),
            true,
        ),
        _ => (hitbox, false),
    }
}

/// Vanilla `Player.aiStep` pickup: touches every entity in the area, then one random orb.
/// Runs after the player moved and before entities tick, like `doTick` after the level tick.
pub(super) fn touch_entities(world: &World, player: &Arc<Player>) {
    if player.living_entity.health.load() <= 0.0 || player.is_spectator() {
        return;
    }
    let (body, mounted) = pickup_body(player);
    let area = if mounted {
        body.expand(1.0, 0.0, 1.0)
    } else {
        body.expand(1.0, 0.5, 1.0)
    };
    let mut orbs = Vec::new();
    // Removed entities may still be indexed; vanilla's section lookup no longer has them
    for entity in world.get_entities_at_box(&area) {
        if !entity.receives_player_touch() || entity.get_entity().is_removed() {
            continue;
        }
        if entity.get_entity().entity_type == &EntityType::EXPERIENCE_ORB {
            orbs.push(entity);
        } else {
            entity.on_player_collision(player);
        }
    }
    if !orbs.is_empty() {
        let orb = &orbs[rand::rng().random_range(0..orbs.len())];
        orb.on_player_collision(player);
    }
}
