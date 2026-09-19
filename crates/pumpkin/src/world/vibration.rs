//! Sculk vibration dispatch.
//!
//! Mirrors vanilla's `VibrationSystem`: a game event is offered to the sculk listeners in
//! range, gated by the listenable-event tag of each listener kind and by the conditions
//! `VibrationSystem.User.isValidVibration` checks on the source of the event.

use std::sync::Arc;
use std::sync::atomic::Ordering;

use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{Block, BlockId, BlockState, game_event::GameEvent};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;

use crate::block::blocks::redstone::sculk_sensor::SculkSensorBlock;
use crate::block::blocks::sculk::sculk_shrieker::SculkShriekerBlock;
use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::world::World;

/// Vanilla's vibration listener range for plain sculk sensors and for shriekers.
const LISTENER_RADIUS: f64 = 8.0;

/// Calibrated sculk sensors listen twice as far as every other sculk listener.
/// Mirrors vanilla's `CalibratedSculkSensorBlockEntity.getListenerRadius`.
const CALIBRATED_LISTENER_RADIUS: f64 = 16.0;

/// Redstone strength a sculk sensor emits for a vibration that travelled `distance` blocks,
/// scaled against that listener's own range. Mirrors vanilla's
/// `VibrationSystem.getRedstoneStrengthForDistance`.
fn redstone_strength_for_distance(distance: f64, listener_radius: f64) -> u8 {
    let power_scale = 15.0 / listener_radius;
    // Callers only pass distances within `listener_radius`, so this never leaves `0..=15`.
    let falloff = (power_scale * distance).floor().clamp(0.0, 15.0) as u8;
    15u8.saturating_sub(falloff).max(1)
}

/// Listening range of the sculk listener `block`, or [`None`] when it is not one.
const fn listener_radius(block: BlockId) -> Option<f64> {
    match block {
        BlockId::CALIBRATED_SCULK_SENSOR => Some(CALIBRATED_LISTENER_RADIUS),
        BlockId::SCULK_SENSOR | BlockId::SCULK_SHRIEKER => Some(LISTENER_RADIUS),
        _ => None,
    }
}

/// Vibration frequency a game event is picked up with, or [`None`] when the event does not
/// produce a vibration at all. Mirrors vanilla's `VIBRATION_FREQUENCY_FOR_EVENT` table.
const fn vibration_frequency(event: GameEvent) -> Option<u8> {
    let frequency = match event {
        GameEvent::Step | GameEvent::Swim | GameEvent::Flap | GameEvent::Resonate1 => 1,
        GameEvent::ProjectileLand
        | GameEvent::HitGround
        | GameEvent::Splash
        | GameEvent::Bounce
        | GameEvent::Resonate2 => 2,
        GameEvent::ItemInteractFinish
        | GameEvent::ProjectileShoot
        | GameEvent::InstrumentPlay
        | GameEvent::Resonate3 => 3,
        GameEvent::EntityAction
        | GameEvent::ElytraGlide
        | GameEvent::Unequip
        | GameEvent::Resonate4 => 4,
        GameEvent::EntityDismount | GameEvent::Equip | GameEvent::Resonate5 => 5,
        GameEvent::EntityInteract
        | GameEvent::Shear
        | GameEvent::EntityMount
        | GameEvent::Resonate6 => 6,
        GameEvent::EntityDamage | GameEvent::Resonate7 => 7,
        GameEvent::Drink | GameEvent::Eat | GameEvent::Resonate8 => 8,
        GameEvent::ContainerClose
        | GameEvent::BlockClose
        | GameEvent::BlockDeactivate
        | GameEvent::BlockDetach
        | GameEvent::Resonate9 => 9,
        GameEvent::ContainerOpen
        | GameEvent::BlockOpen
        | GameEvent::BlockActivate
        | GameEvent::BlockAttach
        | GameEvent::PrimeFuse
        | GameEvent::NoteBlockPlay
        | GameEvent::Resonate10 => 10,
        GameEvent::BlockChange | GameEvent::Resonate11 => 11,
        GameEvent::BlockDestroy | GameEvent::FluidPickup | GameEvent::Resonate12 => 12,
        GameEvent::BlockPlace | GameEvent::FluidPlace | GameEvent::Resonate13 => 13,
        GameEvent::EntityPlace
        | GameEvent::LightningStrike
        | GameEvent::Teleport
        | GameEvent::Resonate14 => 14,
        GameEvent::EntityDie | GameEvent::Explode | GameEvent::Resonate15 => 15,
        GameEvent::ItemInteractStart
        | GameEvent::JukeboxPlay
        | GameEvent::JukeboxStopPlay
        | GameEvent::SculkSensorTendrilsClicking
        | GameEvent::Shriek => return None,
    };
    Some(frequency)
}

/// Whether an event coming from this source reaches listeners at all, regardless of which
/// listener it is. Mirrors the source-side half of vanilla's
/// `VibrationSystem.User.isValidVibration`.
///
/// `affected_state` is the block the event happened against -- for a footstep, the block
/// being stood on. A carpet laid over stone is itself what dampens, so a caller sampling
/// the block below the carpet would wrongly let the step be heard.
fn source_can_be_heard(
    event: GameEvent,
    sneaking: bool,
    affected_state: Option<&'static BlockState>,
) -> bool {
    // A crouching source hides the events that can be hidden; the rest still carry.
    if sneaking && event.has_tag(&tag::GameEvent::MINECRAFT_IGNORE_VIBRATIONS_SNEAKING) {
        return false;
    }

    // Wool and carpets swallow the vibration outright.
    !affected_state.is_some_and(|state| {
        Block::from_state_id(state.id).has_tag(&tag::Block::MINECRAFT_DAMPENS_VIBRATIONS)
    })
}

/// Delivers a game event to the sculk listeners around `position`.
///
/// `source_player` is the player the event can be attributed to, if any, and
/// `affected_state` the block state the event happened against -- for a footstep, the block
/// being stood on. Both are what vanilla carries in `GameEvent.Context`.
///
/// Simplified against vanilla: vibrations arrive immediately instead of travelling one
/// block per tick, and signal occlusion is not evaluated yet.
pub(super) fn dispatch(
    world: &Arc<World>,
    event_key: &str,
    position: Vector3<f64>,
    source_player: Option<&Player>,
    affected_state: Option<&'static BlockState>,
) {
    let Some(event) = GameEvent::from_name(event_key) else {
        return;
    };

    // Each listener kind declares the events it can hear through a tag, the way vanilla's
    // `VibrationSystem.User.getListenableEvents` does. Nothing can hear an event in neither.
    let sensors_listen = event.has_tag(&tag::GameEvent::MINECRAFT_VIBRATIONS);
    let shriekers_listen = event.has_tag(&tag::GameEvent::MINECRAFT_SHRIEKER_CAN_LISTEN);
    if !sensors_listen && !shriekers_listen {
        return;
    }

    // The remaining checks in `isValidVibration` are about the source rather than the
    // listener, so they hold for every listener alike and are made once here.
    let sneaking =
        source_player.is_some_and(|player| player.get_entity().sneaking.load(Ordering::Relaxed));
    if !source_can_be_heard(event, sneaking, affected_state) {
        return;
    }

    let origin_chunk = BlockPos::containing_vec(position).chunk_position();
    // The widest listener reaches 16 blocks. Measured from a source inside this chunk,
    // that only ever spans the directly neighbouring chunks: a source at block `s` in
    // `0..=15` reaches blocks `s - 16 ..= s + 16`, which stays within `-16..=31`.
    let mut candidates = Vec::new();
    for offset_x in -1..=1 {
        for offset_z in -1..=1 {
            let chunk = Vector2::new(origin_chunk.x + offset_x, origin_chunk.y + offset_z);
            let Some(block_entities) = world.block_entities.get(&chunk) else {
                continue;
            };
            candidates.extend(block_entities.keys().filter_map(|pos| {
                let squared = pos.to_centered_f64().squared_distance_to_vec(&position);
                (squared <= CALIBRATED_LISTENER_RADIUS * CALIBRATED_LISTENER_RADIUS)
                    .then(|| (*pos, squared.sqrt()))
            }));
        }
    }

    for (pos, distance) in candidates {
        let block = world.get_block(&pos);
        // Each listener type has its own range, so filter against that rather than the
        // radius the candidates were gathered with.
        let Some(radius) = listener_radius(block.id).filter(|radius| distance <= *radius) else {
            continue;
        };

        match block.id {
            BlockId::SCULK_SENSOR | BlockId::CALIBRATED_SCULK_SENSOR => {
                let Some(frequency) = sensors_listen.then(|| vibration_frequency(event)).flatten()
                else {
                    continue;
                };
                SculkSensorBlock::trigger(
                    world,
                    &pos,
                    block,
                    frequency,
                    redstone_strength_for_distance(distance, radius),
                    source_player,
                );
            }
            // Vanilla only lets shriekers respond to vibrations caused by a player, and the
            // response itself -- warning level, reply sound, Darkness, the warden -- is the
            // same flow a player standing on one goes through.
            BlockId::SCULK_SHRIEKER => {
                if !shriekers_listen {
                    continue;
                }
                if let Some(player) =
                    source_player.and_then(|player| world.get_player_by_id(player.entity_id()))
                {
                    SculkShriekerBlock::try_shriek(world, &pos, &player);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_data::{BlockId, game_event::GameEvent};

    use pumpkin_data::Block;

    use super::{
        CALIBRATED_LISTENER_RADIUS, LISTENER_RADIUS, listener_radius,
        redstone_strength_for_distance, source_can_be_heard, vibration_frequency,
    };

    #[test]
    fn dampening_blocks_swallow_the_vibration_they_are_stood_on() {
        // Vanilla checks `GameEvent.Context.affectedState` against `DAMPENS_VIBRATIONS`,
        // and for a footstep that state is the supporting block. A carpet over stone is
        // itself the supporting block, so it is the carpet that has to be seen here.
        for block in [&Block::WHITE_CARPET, &Block::WHITE_WOOL] {
            assert!(
                !source_can_be_heard(GameEvent::Step, false, Some(block.default_state)),
                "{} should dampen",
                block.name
            );
        }
        assert!(source_can_be_heard(
            GameEvent::Step,
            false,
            Some(Block::STONE.default_state)
        ));
        assert!(source_can_be_heard(GameEvent::Step, false, None));
    }

    #[test]
    fn crouching_hides_only_the_events_vanilla_lets_it_hide() {
        // `ignore_vibrations_sneaking` covers movement and interaction starts, not the
        // events a crouching player still gives away by acting on the world.
        assert!(!source_can_be_heard(GameEvent::Step, true, None));
        assert!(!source_can_be_heard(GameEvent::Swim, true, None));
        assert!(!source_can_be_heard(GameEvent::ProjectileShoot, true, None));
        assert!(source_can_be_heard(GameEvent::BlockPlace, true, None));
        assert!(source_can_be_heard(GameEvent::BlockDestroy, true, None));
    }

    #[test]
    fn vibration_frequencies_match_vanilla_table() {
        // Keyed by registry name rather than enum variant so that a wrongly grouped match arm in
        // `vibration_frequency` is caught. Mirrors vanilla's
        // `VibrationSystem.VIBRATION_FREQUENCY_FOR_EVENT`, in the order vanilla fills it.
        const VANILLA_TABLE: &[(&str, u8)] = &[
            ("step", 1),
            ("swim", 1),
            ("flap", 1),
            ("projectile_land", 2),
            ("hit_ground", 2),
            ("splash", 2),
            ("bounce", 2),
            ("item_interact_finish", 3),
            ("projectile_shoot", 3),
            ("instrument_play", 3),
            ("entity_action", 4),
            ("elytra_glide", 4),
            ("unequip", 4),
            ("entity_dismount", 5),
            ("equip", 5),
            ("entity_interact", 6),
            ("shear", 6),
            ("entity_mount", 6),
            ("entity_damage", 7),
            ("drink", 8),
            ("eat", 8),
            ("container_close", 9),
            ("block_close", 9),
            ("block_deactivate", 9),
            ("block_detach", 9),
            ("container_open", 10),
            ("block_open", 10),
            ("block_activate", 10),
            ("block_attach", 10),
            ("prime_fuse", 10),
            ("note_block_play", 10),
            ("block_change", 11),
            ("block_destroy", 12),
            ("fluid_pickup", 12),
            ("block_place", 13),
            ("fluid_place", 13),
            ("entity_place", 14),
            ("lightning_strike", 14),
            ("teleport", 14),
            ("entity_die", 15),
            ("explode", 15),
        ];

        for (name, expected) in VANILLA_TABLE {
            assert_eq!(
                GameEvent::from_name(name).and_then(vibration_frequency),
                Some(*expected),
                "{name} should be picked up at frequency {expected}"
            );
        }

        // Vanilla fills the resonance events with their own frequency in a loop.
        for frequency in 1..=15u8 {
            let name = format!("resonate_{frequency}");
            assert_eq!(
                GameEvent::from_name(&name).and_then(vibration_frequency),
                Some(frequency),
                "{name} should resonate at its own frequency"
            );
        }

        // Events vanilla leaves out of the table default to 0, meaning no vibration. The two
        // sculk-emitted ones matter most: giving them a frequency would let shriekers and
        // sensors set each other off in a loop.
        for name in [
            "item_interact_start",
            "jukebox_play",
            "jukebox_stop_play",
            "sculk_sensor_tendrils_clicking",
            "shriek",
        ] {
            assert!(
                GameEvent::from_name(name).is_some(),
                "{name} should be a known game event"
            );
            assert_eq!(
                GameEvent::from_name(name).and_then(vibration_frequency),
                None,
                "{name} should not produce a vibration"
            );
        }
    }

    #[test]
    fn redstone_strength_follows_vanilla_falloff() {
        // Vanilla: `max(1, 15 - floor(15 / listenerRadius * distance))`. Calibrated sensors
        // listen twice as far, so they fall off half as fast over the same distance.
        for (distance, radius, expected) in [
            (0.0, LISTENER_RADIUS, 15),
            (0.5, LISTENER_RADIUS, 15),
            (1.0, LISTENER_RADIUS, 14),
            (4.0, LISTENER_RADIUS, 8),
            (7.5, LISTENER_RADIUS, 1),
            (LISTENER_RADIUS, LISTENER_RADIUS, 1),
            (0.0, CALIBRATED_LISTENER_RADIUS, 15),
            (1.0, CALIBRATED_LISTENER_RADIUS, 15),
            (8.0, CALIBRATED_LISTENER_RADIUS, 8),
            (CALIBRATED_LISTENER_RADIUS, CALIBRATED_LISTENER_RADIUS, 1),
        ] {
            assert_eq!(
                redstone_strength_for_distance(distance, radius),
                expected,
                "distance {distance} within radius {radius}"
            );
        }
    }

    #[test]
    fn calibrated_sensors_listen_twice_as_far() {
        for (block, expected) in [
            (BlockId::CALIBRATED_SCULK_SENSOR, Some(16.0)),
            (BlockId::SCULK_SENSOR, Some(8.0)),
            (BlockId::SCULK_SHRIEKER, Some(8.0)),
            (BlockId::STONE, None),
        ] {
            assert_eq!(listener_radius(block), expected);
        }
    }
}
