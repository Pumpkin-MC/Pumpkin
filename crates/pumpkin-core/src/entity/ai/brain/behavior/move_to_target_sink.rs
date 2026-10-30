use std::f64::consts::FRAC_PI_2;
use std::sync::Arc;

use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use super::super::BrainTick;
use super::super::memory::position_tracker::PositionTracker;
use super::super::memory::{MemoryModuleId, MemoryStatus, types};
use super::timed::Behavior;
use crate::entity::ai::pathfinder::path::Path;
use crate::entity::ai::util::default_random_pos;
use crate::entity::mob::Mob;

const MAX_COOLDOWN_BEFORE_RETRYING: i32 = 40;
const DEFAULT_MIN_TIMEOUT: i32 = 150;
const DEFAULT_MAX_TIMEOUT: i32 = 250;
const PARTIAL_STEP_HORIZONTAL_DIST: i32 = 10;
const PARTIAL_STEP_VERTICAL_DIST: i32 = 7;
const RETARGET_DISTANCE_SQR: i32 = 4;

const ENTRY_CONDITIONS: [(MemoryModuleId, MemoryStatus); 3] = [
    (
        types::CANT_REACH_WALK_TARGET_SINCE.id(),
        MemoryStatus::Registered,
    ),
    (types::PATH.id(), MemoryStatus::ValueAbsent),
    (types::WALK_TARGET.id(), MemoryStatus::ValuePresent),
];

/// The parts of `WALK_TARGET` a step needs, copied out so the brain can be written meanwhile.
struct WalkTargetView {
    target: Arc<dyn PositionTracker>,
    speed_modifier: f32,
    close_enough_dist: i32,
}

impl WalkTargetView {
    fn read(tick: &BrainTick<'_>) -> Option<Self> {
        tick.state.get(types::WALK_TARGET).map(|walk| Self {
            target: walk.target().clone(),
            speed_modifier: walk.speed_modifier(),
            close_enough_dist: walk.close_enough_dist(),
        })
    }

    fn reached_by(&self, mob: &dyn Mob) -> bool {
        self.target
            .current_block_position()
            .manhattan_distance(mob.get_entity().block_pos.load())
            <= self.close_enough_dist
    }

    fn is_spectator(&self) -> bool {
        self.target
            .tracked_entity()
            .is_some_and(|entity| entity.is_spectator())
    }
}

pub struct MoveToTargetSink {
    min_timeout: i32,
    max_timeout: i32,
    remaining_cooldown: i32,
    path: Option<Path>,
    last_target_pos: Option<BlockPos>,
    speed_modifier: f32,
}

impl Default for MoveToTargetSink {
    fn default() -> Self {
        Self::new(DEFAULT_MIN_TIMEOUT, DEFAULT_MAX_TIMEOUT)
    }
}

impl MoveToTargetSink {
    #[must_use]
    pub const fn new(min_timeout: i32, max_timeout: i32) -> Self {
        Self {
            min_timeout,
            max_timeout,
            remaining_cooldown: 0,
            path: None,
            last_target_pos: None,
            speed_modifier: 0.0,
        }
    }

    fn create_path(mob: &dyn Mob, destination: Vector3<f64>) -> Option<Path> {
        let steered = mob.get_mob_entity().steered();
        let owner = &steered.mob_entity().living_entity;
        steered.navigator().create_path(owner, destination, 0)
    }

    fn try_compute_path(&mut self, tick: &mut BrainTick<'_>, walk: &WalkTargetView) -> bool {
        let target_pos = walk.target.current_block_position();
        self.path = Self::create_path(tick.mob, target_pos.0.to_f64());
        self.speed_modifier = walk.speed_modifier;
        if walk.reached_by(tick.mob) {
            tick.state.erase(types::CANT_REACH_WALK_TARGET_SINCE.id());
            return false;
        }
        if self.path.as_ref().is_some_and(Path::can_reach) {
            tick.state.erase(types::CANT_REACH_WALK_TARGET_SINCE.id());
        } else if !tick
            .state
            .has_memory_value(types::CANT_REACH_WALK_TARGET_SINCE.id())
        {
            tick.state
                .set(types::CANT_REACH_WALK_TARGET_SINCE, tick.time);
        }
        if self.path.is_some() {
            return true;
        }
        let bottom_center = Vector3::new(
            f64::from(target_pos.0.x) + 0.5,
            f64::from(target_pos.0.y),
            f64::from(target_pos.0.z) + 0.5,
        );
        let Some(partial_step) = default_random_pos::get_pos_towards(
            tick.mob,
            PARTIAL_STEP_HORIZONTAL_DIST,
            PARTIAL_STEP_VERTICAL_DIST,
            bottom_center,
            FRAC_PI_2,
        ) else {
            return false;
        };
        self.path = Self::create_path(tick.mob, partial_step);
        self.path.is_some()
    }
}

impl Behavior for MoveToTargetSink {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        &ENTRY_CONDITIONS
    }

    fn min_duration(&self) -> i32 {
        self.min_timeout
    }

    fn max_duration(&self) -> i32 {
        self.max_timeout
    }

    fn check_extra_start_conditions(&mut self, tick: &mut BrainTick<'_>) -> bool {
        if self.remaining_cooldown > 0 {
            self.remaining_cooldown -= 1;
            return false;
        }
        let Some(walk) = WalkTargetView::read(tick) else {
            return false;
        };
        let reached = walk.reached_by(tick.mob);
        if !reached && self.try_compute_path(tick, &walk) {
            self.last_target_pos = Some(walk.target.current_block_position());
            return true;
        }
        tick.state.erase(types::WALK_TARGET.id());
        if reached {
            tick.state.erase(types::CANT_REACH_WALK_TARGET_SINCE.id());
        }
        false
    }

    fn can_still_use(&mut self, tick: &mut BrainTick<'_>) -> bool {
        if self.path.is_none() || self.last_target_pos.is_none() {
            return false;
        }
        let walk = WalkTargetView::read(tick);
        let is_spectator = walk.as_ref().is_some_and(WalkTargetView::is_spectator);
        let done = tick.mob.get_mob_entity().steered().navigator().is_done();
        !done && walk.is_some_and(|walk| !walk.reached_by(tick.mob)) && !is_spectator
    }

    fn stop(&mut self, tick: &mut BrainTick<'_>) {
        let steered = tick.mob.get_mob_entity().steered();
        if WalkTargetView::read(tick).is_some_and(|walk| !walk.reached_by(tick.mob))
            && steered.navigator().is_stuck()
        {
            self.remaining_cooldown = tick
                .mob
                .get_random()
                .random_range(0..MAX_COOLDOWN_BEFORE_RETRYING);
        }
        steered.navigator().stop();
        tick.state.erase(types::WALK_TARGET.id());
        tick.state.erase(types::PATH.id());
        self.path = None;
    }

    fn start(&mut self, tick: &mut BrainTick<'_>) {
        tick.state.set_optional(types::PATH, self.path.clone());
        let steered = tick.mob.get_mob_entity().steered();
        let owner = &steered.mob_entity().living_entity;
        steered
            .navigator()
            .move_to_path(self.path.clone(), f64::from(self.speed_modifier), owner);
    }

    fn tick(&mut self, tick: &mut BrainTick<'_>) {
        let new_path = tick
            .mob
            .get_mob_entity()
            .steered()
            .navigator()
            .get_path()
            .cloned();
        let unchanged = match (&self.path, &new_path) {
            (Some(previous), Some(current)) => previous.same_snapshot(current),
            (None, None) => true,
            _ => false,
        };
        if !unchanged {
            self.path.clone_from(&new_path);
            tick.state.set_optional(types::PATH, new_path);
        }
        let (Some(_), Some(last_target_pos)) = (&self.path, self.last_target_pos) else {
            return;
        };
        let Some(walk) = WalkTargetView::read(tick) else {
            return;
        };
        let target_pos = walk.target.current_block_position();
        if target_pos.squared_distance(&last_target_pos) > RETARGET_DISTANCE_SQR
            && self.try_compute_path(tick, &walk)
        {
            self.last_target_pos = Some(target_pos);
            self.start(tick);
        }
    }

    fn debug_name(&self) -> &'static str {
        "MoveToTargetSink"
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_data::attributes::Attributes;
    use pumpkin_data::entity::EntityType;
    use pumpkin_data::environment_attribute::Activity;

    use super::*;
    use crate::entity::ai::brain::Brain;
    use crate::entity::ai::brain::activity::ActivityData;
    use crate::entity::ai::brain::behavior::Timed;
    use crate::entity::ai::brain::memory::walk_target::WalkTarget;
    use crate::entity::ai::pathfinder::Navigator;
    use crate::entity::ai::test_support::world;
    use crate::entity::mob::MobEntity;
    use crate::world::World;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    struct ProbeMob(MobEntity);

    impl Mob for ProbeMob {
        fn get_mob_entity(&self) -> &MobEntity {
            &self.0
        }
    }

    fn flying_mob(world: Arc<World>) -> ProbeMob {
        let mob = MobEntity::new(crate::entity::Entity::new(
            world,
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::BAT,
        ));
        *mob.navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Navigator::flying();
        ProbeMob(mob)
    }

    fn walking_to(target: BlockPos) -> Brain {
        let mut brain = Brain::default();
        brain.add_activity(ActivityData::with_priority_start(
            Activity::Core,
            0,
            vec![Box::new(Timed::new(MoveToTargetSink::default()))],
        ));
        brain.set(
            types::WALK_TARGET,
            WalkTarget::from_block_pos(target, 1.0, 0),
        );
        brain
    }

    fn navigation_done(mob: &ProbeMob) -> bool {
        mob.0
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_done()
    }

    #[tokio::test]
    async fn walks_to_the_target_and_forgets_it_on_arrival() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let mob = flying_mob(world.clone());
        let mut brain = walking_to(BlockPos::new(8, 64, 0));

        brain.tick(&world, &mob, 1);
        let walking = brain.has_memory_value(types::PATH.id()) && !navigation_done(&mob);
        let cant_reach = brain.has_memory_value(types::CANT_REACH_WALK_TARGET_SINCE.id());
        let progress = {
            let steered = mob.0.steered();
            let mut navigation = steered.navigator();
            let path = navigation.get_path_mut().ok_or("missing route")?;
            path.advance();
            path.get_next_node_index()
        };
        brain.tick(&world, &mob, 2);
        let remembered_progress = brain.get(types::PATH).map(Path::get_next_node_index);
        mob.0
            .living_entity
            .entity
            .set_pos(Vector3::new(8.5, 64.0, 0.5));
        brain.tick(&world, &mob, 3);
        world.level.shutdown().await;

        assert!(walking);
        assert!(!cant_reach);
        assert_eq!(remembered_progress, Some(progress));
        assert!(!brain.has_memory_value(types::WALK_TARGET.id()));
        assert!(!brain.has_memory_value(types::PATH.id()));
        assert!(navigation_done(&mob));
        Ok(())
    }

    #[tokio::test]
    async fn an_unreachable_target_records_when_it_became_unreachable() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let mob = flying_mob(world.clone());
        mob.0
            .living_entity
            .set_attribute_base(&Attributes::FOLLOW_RANGE, 0.0);
        let mut brain = walking_to(BlockPos::new(8, 64, 0));

        brain.tick(&world, &mob, 7);
        world.level.shutdown().await;

        assert_eq!(brain.get(types::CANT_REACH_WALK_TARGET_SINCE), Some(&7));
        Ok(())
    }
}
