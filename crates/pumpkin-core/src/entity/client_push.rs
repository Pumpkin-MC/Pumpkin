//! Entity pushes a player's own client predicts.
//!
//! Server side both editions are the same: vanilla `pushEntities` and `Entity.push`, sent to
//! watchers only (`needsSync`), never to the pushed player.
//!
//! - Java: `ClientLevel.getPushableEntities` lets every living entity push the local player at
//!   the end of its tick, so the client moves itself.
//! - Bedrock: the player definition has no `minecraft:pushable_by_entity`, so the client never
//!   does. The server runs that Java client push per `PlayerAuthInput` and sends it on top of
//!   the client's end of tick velocity.

use std::sync::atomic::Ordering;

use pumpkin_util::math::vector3::Vector3;

use super::{EntityBase, player::Player, velocity};

/// Velocity the Java client adds to the local player at the end of its tick.
#[must_use]
pub fn java_client_push(player: &Player) -> Vector3<f64> {
    let mut push = Vector3::default();
    let entity = player.get_entity();
    if entity.no_physics.load(Ordering::Relaxed)
        || entity.has_vehicle()
        || entity.has_passengers()
        || !player.is_pushable()
    {
        return push;
    }

    let pos = entity.pos.load();
    let world = entity.world.load();
    for pusher in world.get_all_at_box(&entity.bounding_box.load()) {
        let pusher_entity = pusher.get_entity();
        if pusher_entity.entity_id == entity.entity_id
            || pusher.get_living_entity().is_none()
            || pusher_entity.no_physics.load(Ordering::Relaxed)
            || entity.is_passenger_of_same_vehicle(pusher_entity)
            || !velocity::pushable_by(pusher.as_ref(), player)
        {
            continue;
        }

        // Vanilla `Entity.push(Entity)`, the local player's side.
        let pusher_pos = pusher_entity.pos.load();
        let mut dx = pusher_pos.x - pos.x;
        let mut dz = pusher_pos.z - pos.z;
        let mut d = dx.abs().max(dz.abs());
        if d < f64::from(0.01f32) {
            continue;
        }
        d = d.sqrt();
        dx /= d;
        dz /= d;
        let scale = (1.0 / d).min(1.0) * f64::from(0.05f32);
        push.x -= dx * scale;
        push.z -= dz * scale;
    }
    push
}

/// Bedrock: the Java client push on top of the client's end of tick velocity.
pub fn send_bedrock_client_push(player: &Player, client_velocity: Vector3<f64>) {
    let push = java_client_push(player);
    if push != Vector3::default() {
        player.send_own_velocity(client_velocity + push);
    }
}
