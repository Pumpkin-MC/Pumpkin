//! Vibration dispatching for game events (sculk sensors and shriekers).
//!
//! Mirrors the relevant subset of vanilla `VibrationSystem`: each game event
//! has a vibration frequency, listeners (sculk sensors / shriekers) within the
//! event's 8-block notification radius receive it unless the straight line
//! between the source and the listener is blocked by blocks in the
//! `minecraft:occludes_vibration_signals` block tag (wool).

use std::sync::Arc;

use pumpkin_data::Block;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use crate::block::blocks::redstone::sculk_sensor::SculkSensorBlock;
use crate::block::blocks::sculk::sculk_shrieker::SculkShriekerBlock;
use crate::world::World;

/// The vibration frequency a game event resonates at, per vanilla
/// `VibrationSystem#VibrationListener`. Events not listed never reach
/// vibration listeners.
#[must_use]
pub const fn frequency(event: GameEvent) -> u8 {
    match event {
        GameEvent::Step | GameEvent::Swim | GameEvent::Flap => 1,
        GameEvent::ProjectileLand
        | GameEvent::HitGround
        | GameEvent::Splash
        | GameEvent::Bounce => 2,
        GameEvent::ItemInteractFinish | GameEvent::ProjectileShoot | GameEvent::InstrumentPlay => 3,
        GameEvent::EntityAction | GameEvent::ElytraGlide | GameEvent::Unequip => 4,
        GameEvent::EntityDismount | GameEvent::Equip => 5,
        GameEvent::EntityInteract | GameEvent::Shear | GameEvent::EntityMount => 6,
        GameEvent::EntityDamage => 7,
        GameEvent::Drink | GameEvent::Eat => 8,
        GameEvent::ContainerClose
        | GameEvent::BlockClose
        | GameEvent::BlockDeactivate
        | GameEvent::BlockDetach => 9,
        GameEvent::ContainerOpen
        | GameEvent::BlockOpen
        | GameEvent::BlockActivate
        | GameEvent::BlockAttach
        | GameEvent::PrimeFuse
        | GameEvent::NoteBlockPlay => 10,
        GameEvent::BlockChange => 11,
        GameEvent::BlockDestroy | GameEvent::FluidPickup => 12,
        GameEvent::BlockPlace | GameEvent::FluidPlace => 13,
        GameEvent::EntityPlace | GameEvent::LightningStrike | GameEvent::Teleport => 14,
        GameEvent::EntityDie | GameEvent::Explode => 15,
        _ => 0,
    }
}

/// Resolves an event key (`snake_case`, as used by `GameEvent::name()`) back to
/// the event, so string-keyed `World::emit_game_event` callers participate in
/// vibration dispatch.
pub(super) fn game_event_from_key(key: &str) -> Option<GameEvent> {
    // The generated enum has no name() -> variant inverse; match on the
    // well-known keys that appear in vanilla vibration dispatching.
    let event = match key {
        "step" => GameEvent::Step,
        "swim" => GameEvent::Swim,
        "flap" => GameEvent::Flap,
        "projectile_land" => GameEvent::ProjectileLand,
        "hit_ground" => GameEvent::HitGround,
        "splash" => GameEvent::Splash,
        "bounce" => GameEvent::Bounce,
        "item_interact_finish" => GameEvent::ItemInteractFinish,
        "projectile_shoot" => GameEvent::ProjectileShoot,
        "instrument_play" => GameEvent::InstrumentPlay,
        "entity_action" => GameEvent::EntityAction,
        "elytra_glide" => GameEvent::ElytraGlide,
        "unequip" => GameEvent::Unequip,
        "entity_dismount" => GameEvent::EntityDismount,
        "equip" => GameEvent::Equip,
        "entity_interact" => GameEvent::EntityInteract,
        "shear" => GameEvent::Shear,
        "entity_mount" => GameEvent::EntityMount,
        "entity_damage" => GameEvent::EntityDamage,
        "drink" => GameEvent::Drink,
        "eat" => GameEvent::Eat,
        "container_close" => GameEvent::ContainerClose,
        "block_close" => GameEvent::BlockClose,
        "block_deactivate" => GameEvent::BlockDeactivate,
        "block_detach" => GameEvent::BlockDetach,
        "container_open" => GameEvent::ContainerOpen,
        "block_open" => GameEvent::BlockOpen,
        "block_activate" => GameEvent::BlockActivate,
        "block_attach" => GameEvent::BlockAttach,
        "prime_fuse" => GameEvent::PrimeFuse,
        "note_block_play" => GameEvent::NoteBlockPlay,
        "block_change" => GameEvent::BlockChange,
        "block_destroy" => GameEvent::BlockDestroy,
        "fluid_pickup" => GameEvent::FluidPickup,
        "block_place" => GameEvent::BlockPlace,
        "fluid_place" => GameEvent::FluidPlace,
        "entity_place" => GameEvent::EntityPlace,
        "lightning_strike" => GameEvent::LightningStrike,
        "teleport" => GameEvent::Teleport,
        "entity_die" => GameEvent::EntityDie,
        "explode" => GameEvent::Explode,
        _ => return None,
    };
    Some(event)
}

/// Whether any block along the straight line between the source and the
/// destination occludes vibration signals (wool), mirroring vanilla
/// `VibrationSystem#isOccluded` closely enough for gameplay purposes.
fn is_occluded(world: &World, source: Vector3<f64>, dest: Vector3<f64>) -> bool {
    let direction = dest - source;
    let distance = direction.length();
    if distance < f64::EPSILON {
        return false;
    }

    let steps = (distance / 0.5).ceil() as i32;
    for i in 0..=steps {
        let t = f64::from(i) / f64::from(steps);
        let sample = source + direction * t;
        let pos = BlockPos::containing_vec(sample);
        let block = world.get_block(&pos);
        if block.has_tag(&tag::Block::MINECRAFT_OCCLUDES_VIBRATION_SIGNALS) {
            return true;
        }
    }
    false
}

/// Delivers a game event to vibration listeners (sculk sensors and shriekers)
/// within the 8-block notification radius, unless occluded by wool.
pub(super) fn dispatch(world: &Arc<World>, event: GameEvent, position: Vector3<f64>) {
    let freq = frequency(event);
    if freq == 0 {
        return;
    }

    let listeners: Vec<_> = {
        let set = world
            .vibration_listeners
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        set.iter().copied().collect()
    };

    for pos in listeners {
        let block = world.get_block(&pos);
        let center = Vector3::new(
            f64::from(pos.0.x) + 0.5,
            f64::from(pos.0.y) + 0.5,
            f64::from(pos.0.z) + 0.5,
        );
        if (center - position).length_squared() > 64.0 {
            continue;
        }

        if is_occluded(world, position, center) {
            continue;
        }

        if block == &Block::SCULK_SHRIEKER {
            SculkShriekerBlock::try_activate(world, &pos);
        } else if block == &Block::SCULK_SENSOR || block == &Block::CALIBRATED_SCULK_SENSOR {
            SculkSensorBlock::trigger(world, &pos, block, freq);
        }
    }
}
