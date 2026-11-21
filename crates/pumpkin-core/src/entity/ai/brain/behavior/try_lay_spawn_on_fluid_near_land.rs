use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{Block, BlockDirection, HorizontalFacingExt};
use pumpkin_world::world::BlockFlags;

use super::super::memory::{MemoryStatus, types};
use super::OneShot;
use crate::entity::EntityBase;
use crate::world::game_event::GameEventContext;

/// Places `spawn_block` on a free fluid or supporting block next to where a pregnant mob stands.
#[must_use]
pub fn create(spawn_block: &'static Block) -> OneShot {
    OneShot::new(
        "TryLaySpawnOnFluidNearLand",
        vec![
            (types::ATTACK_TARGET.id(), MemoryStatus::ValueAbsent),
            (types::WALK_TARGET.id(), MemoryStatus::ValuePresent),
            (types::IS_PREGNANT.id(), MemoryStatus::ValuePresent),
        ],
        move |tick| {
            let entity = tick.mob.get_entity();
            if entity.is_in_water() || !entity.on_ground.load(std::sync::atomic::Ordering::Relaxed)
            {
                return false;
            }
            let below = entity.block_pos.load().down();
            for facing in BlockDirection::horizontal_worldgen() {
                let side = below.offset(facing.to_block_direction().to_offset());
                let (side_block, side_state) = tick.world.get_block_and_state(&side);
                let top_face_empty = !side_state
                    .get_block_collision_shapes()
                    .any(|shape| shape.max.y >= 1.0);
                if top_face_empty
                    && (tick
                        .world
                        .fluid_state_has_tag(&side, &tag::Fluid::MINECRAFT_SUPPORTS_FROGSPAWN)
                        || side_block.has_tag(&tag::Block::MINECRAFT_SUPPORTS_FROGSPAWN))
                {
                    let spawn_pos = side.up();
                    if tick.world.get_block_state(&spawn_pos).is_air() {
                        tick.world.set_block_state(
                            &spawn_pos,
                            spawn_block.default_state.id,
                            BlockFlags::NOTIFY_ALL,
                        );
                        let source: &dyn EntityBase = tick.mob;
                        tick.world.emit_game_event_with_context(
                            pumpkin_data::game_event::GameEvent::BlockPlace.name(),
                            spawn_pos.to_f64(),
                            &GameEventContext {
                                source_entity: Some(source),
                            },
                        );
                        tick.world.play_sound(
                            Sound::EntityFrogLaySpawn,
                            SoundCategory::Blocks,
                            &entity.pos.load(),
                        );
                        tick.state.erase(types::IS_PREGNANT.id());
                        return true;
                    }
                }
            }
            true
        },
    )
}
