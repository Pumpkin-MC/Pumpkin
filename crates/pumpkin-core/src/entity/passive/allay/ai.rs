use std::sync::Arc;

use pumpkin_data::{Block, environment_attribute::Activity};
use pumpkin_util::{
    GameMode,
    math::{position::BlockPos, vector3::Vector3},
};

use super::AllayEntity;
use crate::entity::ai::brain::{
    Brain, BrainProvider, BrainState, BrainTick,
    activity::ActivityData,
    behavior::{
        AnimalPanic, CountDownCooldownTicks, DoNothing, GoAndGiveItemsToTarget, LookAtTargetSink,
        MoveToTargetSink, Swim, Timed, go_to_wanted_item, random_stroll, run_one,
        set_entity_look_target_sometimes, set_walk_target_from_look_target, stay_close_to_target,
    },
    memory::{
        GlobalPos, MemoryModuleId, PackedMemories,
        position_tracker::{BlockPosTracker, EntityTracker, PositionTracker},
        types,
    },
    sensing::SensorType,
};
use crate::entity::{EntityBase, mob::Mob, player::Player};
use crate::world::World;

const SPEED_MULTIPLIER_WHEN_IDLING: f32 = 1.0;
const SPEED_MULTIPLIER_WHEN_FOLLOWING_DEPOSIT_TARGET: f32 = 2.25;
const SPEED_MULTIPLIER_WHEN_RETRIEVING_ITEM: f32 = 1.75;
const SPEED_MULTIPLIER_WHEN_PANICKING: f32 = 2.5;
const CLOSE_ENOUGH_TO_TARGET: i32 = 4;
const TOO_FAR_FROM_TARGET: i32 = 16;
const MAX_LOOK_DISTANCE: f32 = 6.0;
const MIN_WAIT_DURATION: i32 = 30;
const MAX_WAIT_DURATION: i32 = 60;
const TIME_TO_FORGET_NOTEBLOCK: i32 = 600;
const DISTANCE_TO_WANTED_ITEM: i32 = 32;
const GIVE_ITEM_TIMEOUT_DURATION: i32 = 20;
const THROW_VELOCITY: Vector3<f32> = Vector3::new(0.2, 0.3, 0.2);
const ITEM_PICKUP_COOLDOWN_DURATION: i32 = 60;
const LIKED_PLAYER_MAX_DISTANCE: f64 = 64.0;
pub(super) const MAX_NOTEBLOCK_DISTANCE: u32 = 1024;

const MEMORIES: [MemoryModuleId; 3] = [
    types::LIKED_PLAYER.id(),
    types::LIKED_NOTEBLOCK_POSITION.id(),
    types::LIKED_NOTEBLOCK_COOLDOWN_TICKS.id(),
];

pub(super) fn make_brain(allay: &AllayEntity, packed: &PackedMemories) -> Brain {
    BrainProvider {
        memory_types: &MEMORIES,
        sensor_types: &[
            SensorType::NearestLivingEntities,
            SensorType::NearestPlayers,
            SensorType::HurtBy,
            SensorType::NearestItems,
        ],
        activities,
    }
    .make_brain(allay, packed)
}

fn activities(_mob: &dyn Mob) -> Vec<ActivityData> {
    vec![
        ActivityData::with_priority_start(
            Activity::Core,
            0,
            vec![
                Box::new(Timed::new(Swim::new(0.8))),
                Box::new(Timed::new(AnimalPanic::new(
                    SPEED_MULTIPLIER_WHEN_PANICKING,
                ))),
                Box::new(Timed::new(LookAtTargetSink::new(45, 90))),
                Box::new(Timed::new(MoveToTargetSink::default())),
                Box::new(CountDownCooldownTicks::new(
                    types::LIKED_NOTEBLOCK_COOLDOWN_TICKS,
                )),
                Box::new(CountDownCooldownTicks::new(
                    types::ITEM_PICKUP_COOLDOWN_TICKS,
                )),
            ],
        ),
        ActivityData::with_priority_start(
            Activity::Idle,
            0,
            vec![
                Box::new(go_to_wanted_item::create(
                    SPEED_MULTIPLIER_WHEN_RETRIEVING_ITEM,
                    true,
                    DISTANCE_TO_WANTED_ITEM,
                )),
                Box::new(Timed::new(GoAndGiveItemsToTarget::new(
                    get_item_deposit_position,
                    SPEED_MULTIPLIER_WHEN_FOLLOWING_DEPOSIT_TARGET,
                    GIVE_ITEM_TIMEOUT_DURATION,
                    throw_item,
                    types::ITEM_PICKUP_COOLDOWN_TICKS,
                    ITEM_PICKUP_COOLDOWN_DURATION,
                    has_item_to_throw,
                ))),
                Box::new(stay_close_to_target::create(
                    get_item_deposit_position,
                    |tick| !has_wanted_item(tick),
                    CLOSE_ENOUGH_TO_TARGET,
                    TOO_FAR_FROM_TARGET,
                    SPEED_MULTIPLIER_WHEN_FOLLOWING_DEPOSIT_TARGET,
                )),
                Box::new(set_entity_look_target_sometimes::create(
                    MAX_LOOK_DISTANCE,
                    MIN_WAIT_DURATION,
                    MAX_WAIT_DURATION,
                )),
                Box::new(run_one(vec![
                    (
                        Box::new(random_stroll::fly(SPEED_MULTIPLIER_WHEN_IDLING)),
                        2,
                    ),
                    (
                        Box::new(set_walk_target_from_look_target::create(
                            SPEED_MULTIPLIER_WHEN_IDLING,
                            3,
                        )),
                        2,
                    ),
                    (
                        Box::new(DoNothing::new(MIN_WAIT_DURATION, MAX_WAIT_DURATION)),
                        1,
                    ),
                ])),
            ],
        ),
    ]
}

fn has_item_to_throw(tick: &BrainTick<'_>) -> bool {
    tick.mob
        .cast_any()
        .downcast_ref::<AllayEntity>()
        .is_some_and(|allay| {
            !allay
                .inventory
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .is_empty()
        })
}

pub(super) fn hear_noteblock(allay: &AllayEntity, pos: BlockPos) {
    let world = allay.get_entity().world.load();
    let Some(dimension) =
        pumpkin_data::dimension::Dimension::from_name(world.dimension.minecraft_name)
    else {
        return;
    };
    let target = GlobalPos::new(dimension, pos);
    allay.mob_entity.with_brain(allay, |brain| {
        if brain
            .get(types::LIKED_NOTEBLOCK_POSITION)
            .is_none_or(|liked| *liked == target)
        {
            brain.set(types::LIKED_NOTEBLOCK_POSITION, target);
            brain.set(
                types::LIKED_NOTEBLOCK_COOLDOWN_TICKS,
                TIME_TO_FORGET_NOTEBLOCK,
            );
        }
    });
}

fn get_item_deposit_position(tick: &mut BrainTick<'_>) -> Option<Arc<dyn PositionTracker>> {
    if let Some(liked) = tick.state.get(types::LIKED_NOTEBLOCK_POSITION).copied() {
        // Loading runs outside the brain tick; missing data is not air.
        if liked.is_close_enough(
            &tick.world.dimension,
            tick.mob.get_entity().block_pos.load(),
            MAX_NOTEBLOCK_DISTANCE,
        ) && tick.world.is_in_build_limit(liked.pos)
        {
            if let Some(allay) = tick.mob.cast_any().downcast_ref::<AllayEntity>() {
                allay.request_noteblock_chunk(tick.world, liked.pos);
            }
            if !tick.world.is_loaded(&liked.pos) {
                return None;
            }
        }
        if should_deposit_items_at_liked_noteblock(tick, &liked) {
            return Some(Arc::new(BlockPosTracker::new(liked.pos.up())));
        }
        tick.state.erase(types::LIKED_NOTEBLOCK_POSITION.id());
    }
    liked_player(tick.state, tick.world, tick.mob)
        .map(|player| Arc::new(EntityTracker::new(player, true)) as Arc<dyn PositionTracker>)
}

fn has_wanted_item(tick: &BrainTick<'_>) -> bool {
    tick.state
        .has_memory_value(types::NEAREST_VISIBLE_WANTED_ITEM.id())
}

fn should_deposit_items_at_liked_noteblock(tick: &BrainTick<'_>, liked: &GlobalPos) -> bool {
    liked.is_close_enough(
        &tick.world.dimension,
        tick.mob.get_entity().block_pos.load(),
        MAX_NOTEBLOCK_DISTANCE,
    ) && tick.world.get_block(&liked.pos) == &Block::NOTE_BLOCK
        && tick
            .state
            .has_memory_value(types::LIKED_NOTEBLOCK_COOLDOWN_TICKS.id())
}

pub(super) fn liked_player(
    state: &BrainState,
    world: &World,
    allay: &dyn Mob,
) -> Option<Arc<Player>> {
    let uuid = state.get(types::LIKED_PLAYER)?;
    world.get_player_by_uuid(*uuid).filter(|player| {
        matches!(
            player.gamemode.load(),
            GameMode::Survival | GameMode::Creative
        ) && player
            .get_entity()
            .pos
            .load()
            .squared_distance_to_vec(&allay.get_entity().pos.load())
            < LIKED_PLAYER_MAX_DISTANCE * LIKED_PLAYER_MAX_DISTANCE
    })
}

fn throw_item(tick: &mut BrainTick<'_>, target_pos: Vector3<f64>) {
    if let Some(allay) = tick.mob.cast_any().downcast_ref::<AllayEntity>() {
        allay.throw_one_item(
            target_pos,
            THROW_VELOCITY,
            liked_player(tick.state, tick.world, tick.mob),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{Entity, ai::test_support::world};
    use pumpkin_data::entity::EntityType;
    use pumpkin_world::chunk::{
        ChunkData,
        io::{Dirtiable, FileIO},
    };

    #[tokio::test]
    async fn unloaded_noteblock_is_reloaded_without_forgetting_the_target()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let pos = BlockPos::new(32, 64, 0);
        let (chunk_pos, relative) = pos.chunk_and_chunk_relative_position();
        let chunk = ChunkData::empty_sync(chunk_pos.x, chunk_pos.y);
        chunk.set_block_absolute_y(
            relative.x as usize,
            relative.y,
            relative.z as usize,
            Block::NOTE_BLOCK.default_state.id,
        );
        chunk
            .light_populated
            .store(true, std::sync::atomic::Ordering::Relaxed);
        chunk.mark_dirty(true);
        world.level.loaded_chunks.insert(chunk_pos, chunk.clone());
        world.level.write_chunks(vec![(chunk_pos, chunk)]).await;
        world.level.chunk_saver.flush_all().await;

        let allay = AllayEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::ALLAY,
        ));
        hear_noteblock(&allay, pos);
        let deposit = || {
            allay.mob_entity.with_brain(allay.as_ref(), |brain| {
                get_item_deposit_position(&mut BrainTick {
                    state: brain,
                    world: &world,
                    mob: allay.as_ref(),
                    time: 0,
                })
                .map(|target| target.current_block_position())
            })
        };
        assert_eq!(deposit(), Some(pos.up()));
        world.level.loaded_chunks.remove(&chunk_pos);
        assert!(deposit().is_none());
        assert!(allay.mob_entity.with_brain(allay.as_ref(), |brain| {
            brain.has_memory_value(types::LIKED_NOTEBLOCK_POSITION.id())
        }));

        tokio::time::timeout(std::time::Duration::from_secs(30), async {
            while !world.level.is_chunk_loaded(&chunk_pos) {
                tokio::task::yield_now().await;
            }
        })
        .await?;
        let has_ticket = || {
            world
                .level
                .chunk_loading
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .ticket
                .contains_key(&chunk_pos)
        };
        assert_eq!(deposit(), Some(pos.up()));

        world
            .level
            .set_block_state(&pos, Block::AIR.default_state.id);
        assert!(deposit().is_none());
        assert!(!allay.mob_entity.with_brain(allay.as_ref(), |brain| {
            brain.has_memory_value(types::LIKED_NOTEBLOCK_POSITION.id())
        }));
        allay.finish_noteblock_chunk();
        assert!(has_ticket());
        allay.finish_noteblock_chunk();
        assert!(
            allay
                .noteblock_chunk
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .is_none()
        );
        assert!(!has_ticket());
        world.level.shutdown().await;
        Ok(())
    }
}
