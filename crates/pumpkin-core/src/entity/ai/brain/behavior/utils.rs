use std::sync::Arc;

use uuid::Uuid;

use crate::entity::EntityBase;
use crate::world::World;

use super::super::memory::position_tracker::{BlockPosTracker, EntityTracker, PositionTracker};
use super::super::memory::walk_target::WalkTarget;
use super::super::memory::{MemoryModuleType, types};
use super::super::{Brain, VisibilityContext};

#[must_use]
pub fn entity_is_visible(ctx: &VisibilityContext<'_>, target: &dyn EntityBase) -> bool {
    ctx.brain
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
    ctx.brain.get(memory).is_some_and(|target| {
        predicate(target)
            && target.get_entity().is_alive()
            && entity_is_visible(ctx, target.as_ref())
    })
}

pub fn look_at_entity(brain: &mut Brain, target: Arc<dyn EntityBase>) {
    brain.set(
        types::LOOK_TARGET,
        Arc::new(EntityTracker::new(target, true)) as Arc<dyn PositionTracker>,
    );
}

pub fn set_walk_and_look_target_memories(
    brain: &mut Brain,
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
    brain: &mut Brain,
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
    brain: &mut Brain,
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
    brain: &Brain,
    world: &World,
    memory: MemoryModuleType<Uuid>,
) -> Option<Arc<dyn EntityBase>> {
    let uuid = *brain.get(memory)?;
    let entity = world.get_entity_by_uuid(uuid)?;
    entity.get_living_entity().is_some().then_some(entity)
}

#[must_use]
pub fn is_other_target_much_further_away_than_current_attack_target(
    brain: &Brain,
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
pub fn is_breeding(brain: &Brain) -> bool {
    brain.has_memory_value(types::BREED_TARGET.id())
}
