use std::sync::Arc;

use pumpkin_data::dimension::Dimension;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_util::math::vector3::Vector3;
use uuid::Uuid;

use crate::entity::ai::pathfinder::node::PathComputationType;
use crate::entity::ai::util::default_random_pos;
use crate::entity::item::ItemEntity;
use crate::entity::mob::Mob;
use crate::entity::{Entity, EntityBase};
use crate::world::World;

use super::super::memory::position_tracker::{BlockPosTracker, EntityTracker, PositionTracker};
use super::super::memory::walk_target::WalkTarget;
use super::super::memory::{MemoryModuleType, types};
use super::super::{BrainState, BrainTick, VisibilityContext};

#[must_use]
pub fn world_for_dimension(world: &Arc<World>, dimension: &Dimension) -> Option<Arc<World>> {
    if world.dimension.id == dimension.id {
        return Some(Arc::clone(world));
    }
    let server = world.server.upgrade()?;
    let worlds = server.worlds.load();
    worlds
        .iter()
        .find(|candidate| candidate.dimension.id == dimension.id)
        .cloned()
}

const MAX_SWIMMABLE_POS_ATTEMPTS: i32 = 10;

#[must_use]
pub fn get_random_swimmable_pos(
    mob: &dyn Mob,
    max_horizontal_distance: i32,
    max_vertical_distance: i32,
) -> Option<Vector3<f64>> {
    let world = mob.get_entity().world.load();
    let mut target =
        default_random_pos::get_pos(mob, max_horizontal_distance, max_vertical_distance);
    let mut count = 0;
    while let Some(pos) = target {
        let (block, state) = world.get_block_and_state(&pos.to_block_pos());
        if world
            .block_registry
            .is_pathfindable(block, state, PathComputationType::Water)
            || count >= MAX_SWIMMABLE_POS_ATTEMPTS
        {
            break;
        }
        count += 1;
        target = default_random_pos::get_pos(mob, max_horizontal_distance, max_vertical_distance);
    }
    target
}

#[must_use]
pub fn entity_is_visible(ctx: &VisibilityContext<'_>, target: &dyn EntityBase) -> bool {
    ctx.state
        .get(types::NEAREST_VISIBLE_LIVING_ENTITIES)
        .is_some_and(|visible| visible.contains(target, ctx))
}

#[must_use]
pub fn can_see(ctx: &VisibilityContext<'_>, target: &dyn EntityBase) -> bool {
    entity_is_visible(ctx, target)
}

#[must_use]
pub fn target_is_valid(
    ctx: &VisibilityContext<'_>,
    memory: MemoryModuleType<Arc<dyn EntityBase>>,
    predicate: impl Fn(&Arc<dyn EntityBase>) -> bool,
) -> bool {
    ctx.state.get(memory).is_some_and(|target| {
        predicate(target)
            && target.get_entity().is_alive()
            && entity_is_visible(ctx, target.as_ref())
    })
}

pub fn look_at_entity(brain: &mut BrainState, target: Arc<dyn EntityBase>) {
    brain.set(
        types::LOOK_TARGET,
        Arc::new(EntityTracker::new(target, true)) as Arc<dyn PositionTracker>,
    );
}

pub fn set_walk_and_look_target_memories(
    brain: &mut BrainState,
    target: Arc<dyn PositionTracker>,
    speed_modifier: f32,
    close_enough_dist: i32,
) {
    brain.set(types::LOOK_TARGET, Arc::clone(&target));
    brain.set(
        types::WALK_TARGET,
        WalkTarget::new(target, speed_modifier, close_enough_dist),
    );
}

pub fn set_walk_and_look_target_memories_to_entity(
    brain: &mut BrainState,
    target: Arc<dyn EntityBase>,
    speed_modifier: f32,
    close_enough_dist: i32,
) {
    set_walk_and_look_target_memories(
        brain,
        Arc::new(EntityTracker::new(target, true)),
        speed_modifier,
        close_enough_dist,
    );
}

pub fn set_walk_and_look_target_memories_to_block(
    brain: &mut BrainState,
    target: pumpkin_util::math::position::BlockPos,
    speed_modifier: f32,
    close_enough_dist: i32,
) {
    set_walk_and_look_target_memories(
        brain,
        Arc::new(BlockPosTracker::new(target)),
        speed_modifier,
        close_enough_dist,
    );
}

#[must_use]
pub fn get_target_nearest_me(
    body: &dyn EntityBase,
    first: Arc<dyn EntityBase>,
    second: Arc<dyn EntityBase>,
) -> Arc<dyn EntityBase> {
    let pos = body.get_entity().pos.load();
    let first_dist = pos.squared_distance_to_vec(&first.get_entity().pos.load());
    let second_dist = pos.squared_distance_to_vec(&second.get_entity().pos.load());
    if first_dist < second_dist {
        first
    } else {
        second
    }
}

#[must_use]
pub fn get_nearest_target(
    body: &dyn EntityBase,
    first: Option<Arc<dyn EntityBase>>,
    second: Arc<dyn EntityBase>,
) -> Arc<dyn EntityBase> {
    match first {
        Some(first) => get_target_nearest_me(body, first, second),
        None => second,
    }
}

#[must_use]
pub fn get_living_entity_from_uuid_memory(
    brain: &BrainState,
    world: &World,
    memory: MemoryModuleType<Uuid>,
) -> Option<Arc<dyn EntityBase>> {
    let uuid = *brain.get(memory)?;
    let entity = world.get_entity_by_uuid(uuid)?;
    entity.get_living_entity().is_some().then_some(entity)
}

#[must_use]
pub fn is_other_target_much_further_away_than_current_attack_target(
    brain: &BrainState,
    body: &dyn EntityBase,
    other_target: &dyn EntityBase,
    how_much_further_away: f64,
) -> bool {
    let Some(current) = brain.get(types::ATTACK_TARGET) else {
        return false;
    };
    let pos = body.get_entity().pos.load();
    let dist_to_current = pos.squared_distance_to_vec(&current.get_entity().pos.load());
    let dist_to_other = pos.squared_distance_to_vec(&other_target.get_entity().pos.load());
    dist_to_other > dist_to_current + how_much_further_away * how_much_further_away
}

#[must_use]
pub fn is_breeding(brain: &BrainState) -> bool {
    brain.has_memory_value(types::BREED_TARGET.id())
}

const THROW_SPEED: f64 = 0.3;
const HAND_Y_DISTANCE_FROM_EYE: f64 = 0.3;

pub fn lock_gaze_and_walk_to_each_other(
    tick: &mut BrainTick<'_>,
    partner: &Arc<dyn EntityBase>,
    speed_modifier: f32,
    close_enough_distance: i32,
) {
    set_walk_and_look_target_memories_to_entity(
        tick.state,
        partner.clone(),
        speed_modifier,
        close_enough_distance,
    );
    let Some(partner_mob) = partner.get_mob() else {
        return;
    };
    let Some(body) = tick.world.get_entity_by_id(tick.mob.get_entity().entity_id) else {
        return;
    };
    partner_mob
        .get_mob_entity()
        .with_brain(partner_mob, |brain| {
            set_walk_and_look_target_memories_to_entity(
                brain,
                body,
                speed_modifier,
                close_enough_distance,
            );
        });
}

pub fn throw_item(thrower: &dyn Mob, stack: ItemStack, target_pos: Vector3<f64>) {
    if stack.is_empty() {
        return;
    }
    let entity = thrower.get_entity();
    let world = entity.world.load();
    let pos = entity.pos.load();
    let hand_pos = Vector3::new(pos.x, entity.get_eye_y() - HAND_Y_DISTANCE_FROM_EYE, pos.z);
    let velocity = (target_pos - pos)
        .normalize()
        .multiply(THROW_SPEED, THROW_SPEED, THROW_SPEED);
    let item_entity = ItemEntity::new_with_velocity(
        Entity::new(world.clone(), hand_pos, &EntityType::ITEM),
        stack,
        velocity,
        ItemEntity::DEFAULT_PICKUP_DELAY,
    );
    item_entity.set_thrower(entity.entity_uuid);
    world.spawn_entity(Arc::new(item_entity));
}
