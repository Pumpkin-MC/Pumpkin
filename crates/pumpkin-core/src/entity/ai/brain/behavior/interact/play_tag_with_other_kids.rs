use std::sync::Arc;

use pumpkin_util::math::position::BlockPos;
use rand::RngExt;
use rustc_hash::FxHashMap;

use super::super::super::memory::position_tracker::{
    BlockPosTracker, EntityTracker, PositionTracker,
};
use super::super::super::memory::walk_target::WalkTarget;
use super::super::super::memory::{MemoryStatus, types};
use super::super::OneShot;
use crate::entity::EntityBase;
use crate::entity::ai::random_pos::land_random_pos;

const MAX_FLEE_XZ_DIST: i32 = 20;
const MAX_FLEE_Y_DIST: i32 = 8;
const FLEE_SPEED_MODIFIER: f32 = 0.6;
const CHASE_SPEED_MODIFIER: f32 = 0.6;
const MAX_CHASERS_PER_TARGET: u32 = 5;
const AVERAGE_WAIT_TIME_BETWEEN_RUNS: i32 = 10;
const FLEE_ATTEMPTS: i32 = 10;

#[must_use]
pub fn create() -> OneShot {
    OneShot::new(
        "PlayTagWithOtherKids",
        vec![
            (
                types::VISIBLE_VILLAGER_BABIES.id(),
                MemoryStatus::ValuePresent,
            ),
            (types::WALK_TARGET.id(), MemoryStatus::ValueAbsent),
            (types::LOOK_TARGET.id(), MemoryStatus::Registered),
            (types::INTERACTION_TARGET.id(), MemoryStatus::Registered),
        ],
        |tick| {
            if tick
                .mob
                .get_random()
                .random_range(0..AVERAGE_WAIT_TIME_BETWEEN_RUNS)
                != 0
            {
                return false;
            }
            let Some(friends) = tick.state.get(types::VISIBLE_VILLAGER_BABIES).cloned() else {
                return false;
            };
            let me = tick.mob.get_entity().entity_id;
            if friends
                .iter()
                .any(|friend| is_friend_chasing_me(me, friend))
            {
                for _ in 0..FLEE_ATTEMPTS {
                    let Some(pos) = land_random_pos(tick.mob, MAX_FLEE_XZ_DIST, MAX_FLEE_Y_DIST)
                    else {
                        continue;
                    };
                    if tick.world.is_village(&BlockPos::containing_vec(pos)) {
                        tick.state.set(
                            types::WALK_TARGET,
                            WalkTarget::new(
                                Arc::new(BlockPosTracker::from_vec(pos)),
                                FLEE_SPEED_MODIFIER,
                                0,
                            ),
                        );
                        break;
                    }
                }
            } else if let Some(target) = find_someone_being_chased_among(&friends) {
                chase_kid(tick, &target);
            } else if let Some(target) = friends.first() {
                chase_kid(tick, target);
            }
            true
        },
    )
}

fn chase_kid(tick: &mut super::super::super::BrainTick<'_>, kid: &Arc<dyn EntityBase>) {
    tick.state.set(types::INTERACTION_TARGET, kid.clone());
    tick.state.set(
        types::LOOK_TARGET,
        Arc::new(EntityTracker::new(kid.clone(), true)) as Arc<dyn PositionTracker>,
    );
    tick.state.set(
        types::WALK_TARGET,
        WalkTarget::new(
            Arc::new(EntityTracker::new(kid.clone(), false)),
            CHASE_SPEED_MODIFIER,
            1,
        ),
    );
}

fn find_someone_being_chased_among(friends: &[Arc<dyn EntityBase>]) -> Option<Arc<dyn EntityBase>> {
    let mut chased_kids: FxHashMap<i32, u32> = FxHashMap::default();
    let mut chased_entities: FxHashMap<i32, Arc<dyn EntityBase>> = FxHashMap::default();
    for friend in friends {
        let Some(target) = super::interaction_target_of(friend) else {
            continue;
        };
        let id = target.get_entity().entity_id;
        *chased_kids.entry(id).or_insert(0) += 1;
        chased_entities.entry(id).or_insert(target);
    }
    let id = find_someone_being_chased(&chased_kids)?;
    chased_entities.remove(&id)
}

fn find_someone_being_chased(chased: &FxHashMap<i32, u32>) -> Option<i32> {
    let mut entries: Vec<(i32, u32)> = chased.iter().map(|(id, count)| (*id, *count)).collect();
    entries.sort_by_key(|(_, count)| *count);
    entries
        .into_iter()
        .find_map(|(id, count)| (count > 0 && count <= MAX_CHASERS_PER_TARGET).then_some(id))
}

fn is_friend_chasing_me(me: i32, friend: &Arc<dyn EntityBase>) -> bool {
    super::interaction_target_of(friend).is_some_and(|target| target.get_entity().entity_id == me)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chased_kid_selection_uses_the_one_to_five_window_and_fewest_chasers() {
        let mut chased: FxHashMap<i32, u32> = FxHashMap::default();
        chased.insert(1, 0);
        chased.insert(2, 6);
        chased.insert(3, 5);
        chased.insert(4, 2);
        chased.insert(5, 1);
        assert_eq!(find_someone_being_chased(&chased), Some(5));
        chased.remove(&5);
        assert_eq!(find_someone_being_chased(&chased), Some(4));
        chased.remove(&4);
        assert_eq!(find_someone_being_chased(&chased), Some(3));
        chased.remove(&3);
        assert_eq!(find_someone_being_chased(&chased), None);
    }
}
