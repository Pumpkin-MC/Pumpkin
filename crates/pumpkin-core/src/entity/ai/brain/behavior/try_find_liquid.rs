use pumpkin_data::Block;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::tag::{Tag, Taggable};
use pumpkin_util::math::position::BlockPos;

use super::super::BrainTick;
use super::super::memory::{MemoryStatus, types};
use super::{OneShot, utils};
use crate::world::World;

const SEARCH_INTERVAL: i64 = 40;
const RETRY_DELAY: i64 = 20 + 2;
/// A liquid block this close to the mob is only a last resort (vanilla
/// `closerToCenterThan(position, 1.5)`).
const TOO_CLOSE_DISTANCE: f64 = 1.5;
const CLOSE_ENOUGH_DISTANCE: i32 = 0;

fn fluid_type_at(world: &World, pos: &BlockPos) -> &'static Fluid {
    let (family, state) = world.get_fluid_and_fluid_state(pos);
    if family.matches_type(&Fluid::WATER) {
        if state.is_still {
            &Fluid::WATER
        } else {
            &Fluid::FLOWING_WATER
        }
    } else if family.matches_type(&Fluid::LAVA) {
        if state.is_still {
            &Fluid::LAVA
        } else {
            &Fluid::FLOWING_LAVA
        }
    } else {
        &Fluid::EMPTY
    }
}

fn find_liquid(
    tick: &BrainTick<'_>,
    origin: BlockPos,
    range: i32,
    fluid_tag: &'static Tag,
) -> Option<BlockPos> {
    let world = tick.world;
    let position = tick.mob.get_entity().pos.load();
    let mut covered = None;
    for pos in BlockPos::iterate_outwards(origin, range, range, range) {
        let (block, _) = world.get_block_and_state_id(&pos);
        let is_liquid_block = block == &Block::WATER || block == &Block::LAVA;
        if (pos.0.x == origin.0.x && pos.0.z == origin.0.z)
            || !is_liquid_block
            || !fluid_type_at(world, &pos).has_tag(fluid_tag)
        {
            continue;
        }
        if world.get_block_state(&pos.up()).is_air() {
            return Some(pos);
        }
        if covered.is_none()
            && pos.dist_to_center_sqr(position.x, position.y, position.z)
                >= TOO_CLOSE_DISTANCE * TOO_CLOSE_DISTANCE
        {
            covered = Some(pos);
        }
    }
    covered
}

#[must_use]
pub fn create(range: i32, speed_modifier: f32, fluid_tag: &'static Tag) -> OneShot {
    let mut next_ok_start_time = 0i64;
    OneShot::new(
        "TryFindLiquid",
        vec![
            (types::ATTACK_TARGET.id(), MemoryStatus::ValueAbsent),
            (types::WALK_TARGET.id(), MemoryStatus::ValueAbsent),
            (types::LOOK_TARGET.id(), MemoryStatus::Registered),
        ],
        move |tick| {
            let origin = tick.mob.get_entity().block_pos.load();
            if fluid_type_at(tick.world, &origin).has_tag(fluid_tag) {
                return false;
            }
            if tick.time < next_ok_start_time {
                next_ok_start_time = tick.time + RETRY_DELAY;
                return true;
            }
            if let Some(liquid) = find_liquid(tick, origin, range, fluid_tag) {
                utils::set_walk_and_look_target_memories_to_block(
                    tick.state,
                    liquid,
                    speed_modifier,
                    CLOSE_ENOUGH_DISTANCE,
                );
            }
            next_ok_start_time = tick.time + SEARCH_INTERVAL;
            true
        },
    )
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use pumpkin_data::block_properties::WaterLikeProperties;
    use pumpkin_data::entity::EntityType;
    use pumpkin_data::environment_attribute::Activity;
    use pumpkin_data::tag;
    use pumpkin_util::math::{vector2::Vector2, vector3::Vector3};
    use pumpkin_world::{chunk::ChunkData, world::BlockFlags};

    use super::*;
    use crate::entity::Entity;
    use crate::entity::ai::brain::Brain;
    use crate::entity::ai::brain::activity::ActivityData;
    use crate::entity::ai::test_support::world;
    use crate::entity::passive::cow::CowEntity;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    const SPEED: f32 = 0.15;

    /// A test world whose chunk at the origin is loaded, so that blocks can be placed in it.
    fn world_with_chunk(folder: &tempfile::TempDir) -> Arc<World> {
        let world = world(folder);
        world
            .level
            .loaded_chunks
            .insert(Vector2::new(0, 0), ChunkData::empty_sync(0, 0));
        world
    }

    fn cow(world: &Arc<World>) -> Arc<CowEntity> {
        CowEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::COW,
        ))
    }

    fn brain() -> Brain {
        let mut brain = Brain::default();
        brain.add_activity(ActivityData::with_priority_start(
            Activity::Idle,
            0,
            vec![Box::new(create(
                6,
                SPEED,
                &tag::Fluid::MINECRAFT_AXOLOTL_TRIES_TO_FIND,
            ))],
        ));
        brain
    }

    fn place(world: &Arc<World>, x: i32, y: i32, z: i32, block: &Block) {
        let flags = BlockFlags::NOTIFY_LISTENERS
            | BlockFlags::SKIP_BLOCK_ADDED_CALLBACK
            | BlockFlags::UPDATE_KNOWN_SHAPE;
        world.set_block_state(&BlockPos::new(x, y, z), block.default_state.id, flags);
    }

    fn place_level(world: &Arc<World>, x: i32, y: i32, z: i32, level: u8) {
        let flags = BlockFlags::NOTIFY_LISTENERS
            | BlockFlags::SKIP_BLOCK_ADDED_CALLBACK
            | BlockFlags::UPDATE_KNOWN_SHAPE;
        let state = WaterLikeProperties { level }.to_state_id(&Block::WATER);
        world.set_block_state(&BlockPos::new(x, y, z), state, flags);
    }

    fn walking_to(brain: &Brain) -> Option<(BlockPos, f32, bool)> {
        let walk = brain.get(types::WALK_TARGET)?;
        let pos = walk.target().current_block_position();
        let looks = brain
            .get(types::LOOK_TARGET)
            .is_some_and(|look| look.current_block_position() == pos);
        Some((pos, walk.speed_modifier(), looks))
    }

    #[tokio::test]
    async fn walks_to_water_with_air_above_it_rather_than_to_covered_water() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world_with_chunk(&folder);
        let body = cow(&world);
        place(&world, 3, 64, 0, &Block::WATER);
        place(&world, 3, 65, 0, &Block::STONE);
        place(&world, 5, 64, 0, &Block::WATER);
        let mut brain = brain();

        brain.tick(&world, &*body, 100);
        world.level.shutdown().await;

        assert_eq!(
            walking_to(&brain),
            Some((BlockPos::new(5, 64, 0), SPEED, true))
        );
        Ok(())
    }

    #[tokio::test]
    async fn falls_back_to_the_nearest_covered_water_that_is_not_beside_the_mob() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world_with_chunk(&folder);
        let body = cow(&world);
        for x in [1, 3, 5] {
            place(&world, x, 64, 0, &Block::WATER);
            place(&world, x, 65, 0, &Block::STONE);
        }
        let mut brain = brain();

        brain.tick(&world, &*body, 100);
        world.level.shutdown().await;

        assert_eq!(
            walking_to(&brain),
            Some((BlockPos::new(3, 64, 0), SPEED, true))
        );
        Ok(())
    }

    #[tokio::test]
    async fn ignores_its_own_column_and_flowing_water() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world_with_chunk(&folder);
        let body = cow(&world);
        place(&world, 0, 62, 0, &Block::WATER);
        place_level(&world, 3, 64, 0, 3);
        place_level(&world, 0, 64, 0, 5);
        let mut brain = brain();

        brain.tick(&world, &*body, 100);
        let found_nothing = walking_to(&brain);
        place(&world, 0, 64, 4, &Block::WATER);
        brain.tick(&world, &*body, 200);
        world.level.shutdown().await;

        assert_eq!(found_nothing, None);
        assert_eq!(
            walking_to(&brain),
            Some((BlockPos::new(0, 64, 4), SPEED, true))
        );
        Ok(())
    }

    #[tokio::test]
    async fn does_nothing_while_standing_in_water() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world_with_chunk(&folder);
        let body = cow(&world);
        place(&world, 0, 64, 0, &Block::WATER);
        place(&world, 3, 64, 0, &Block::WATER);
        let mut brain = brain();

        brain.tick(&world, &*body, 100);
        world.level.shutdown().await;

        assert_eq!(walking_to(&brain), None);
        Ok(())
    }

    #[tokio::test]
    async fn searches_again_only_after_the_interval() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world_with_chunk(&folder);
        let body = cow(&world);
        place(&world, 3, 64, 0, &Block::WATER);
        let mut asked_every_tick = brain();
        let mut asked_at_39 = brain();
        let mut asked_at_40 = brain();

        asked_every_tick.tick(&world, &*body, 100);
        asked_at_39.tick(&world, &*body, 100);
        asked_at_40.tick(&world, &*body, 100);
        for brain in [&mut asked_every_tick, &mut asked_at_39, &mut asked_at_40] {
            brain.erase(types::WALK_TARGET.id());
            brain.erase(types::LOOK_TARGET.id());
        }
        for time in 101..=200 {
            asked_every_tick.tick(&world, &*body, time);
        }
        asked_at_39.tick(&world, &*body, 139);
        asked_at_40.tick(&world, &*body, 140);
        world.level.shutdown().await;

        assert_eq!(walking_to(&asked_every_tick), None);
        assert_eq!(walking_to(&asked_at_39), None);
        assert!(walking_to(&asked_at_40).is_some());
        Ok(())
    }
}
