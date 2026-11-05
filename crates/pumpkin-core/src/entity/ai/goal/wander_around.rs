use super::{Controls, Goal, GoalReach, to_goal_ticks};
use crate::entity::ai::util::{default_random_pos, land_random_pos};
use crate::entity::{ai::pathfinder::NavigatorGoal, mob::Mob};
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

type PositionProvider = fn(&dyn Mob) -> Option<Vector3<f64>>;

pub const DEFAULT_INTERVAL: i32 = 120;
pub const PROBABILITY: f32 = 0.001;
const NO_ACTION_TIME_LIMIT: i32 = 100;

pub struct WanderAroundGoal {
    goal_control: Controls,
    speed: f64,
    target: Option<Vector3<f64>>,
    interval: i32,
    check_no_action_time: bool,
    force_trigger: bool,
    water_avoiding_probability: Option<f32>,
    position: Option<PositionProvider>,
    start_condition: Option<fn(&dyn Mob) -> bool>,
    entity_reach: f64,
}

impl WanderAroundGoal {
    #[must_use]
    pub const fn new(speed: f64) -> Self {
        Self::with_interval(speed, DEFAULT_INTERVAL)
    }

    #[must_use]
    pub const fn with_interval(speed: f64, interval: i32) -> Self {
        Self {
            goal_control: Controls::MOVE,
            speed,
            target: None,
            interval,
            check_no_action_time: true,
            force_trigger: false,
            water_avoiding_probability: None,
            position: None,
            start_condition: None,
            entity_reach: 0.0,
        }
    }

    #[must_use]
    pub const fn water_avoiding(speed: f64) -> Self {
        let mut goal = Self::new(speed);
        goal.water_avoiding_probability = Some(PROBABILITY);
        goal
    }

    #[must_use]
    pub const fn with_position(mut self, position: PositionProvider, entity_reach: f64) -> Self {
        self.position = Some(position);
        self.entity_reach = entity_reach;
        self
    }

    #[must_use]
    pub const fn with_start_condition(mut self, start_condition: fn(&dyn Mob) -> bool) -> Self {
        self.start_condition = Some(start_condition);
        self
    }

    #[must_use]
    pub const fn with_check_no_action_time(mut self, check_no_action_time: bool) -> Self {
        self.check_no_action_time = check_no_action_time;
        self
    }

    /// Run once on the next check, ignoring the interval.
    pub const fn trigger(&mut self) {
        self.force_trigger = true;
    }

    pub const fn set_interval(&mut self, interval: i32) {
        self.interval = interval;
    }

    fn get_position(&self, mob: &dyn Mob) -> Option<Vector3<f64>> {
        if let Some(position) = self.position {
            return position(mob);
        }
        let Some(probability) = self.water_avoiding_probability else {
            return default_random_pos::get_pos(mob, 10, 7);
        };
        if mob.get_entity().is_in_water() {
            land_random_pos::get_pos(mob, 15, 7).or_else(|| default_random_pos::get_pos(mob, 10, 7))
        } else if mob.get_random().random::<f32>() >= probability {
            land_random_pos::get_pos(mob, 10, 7)
        } else {
            default_random_pos::get_pos(mob, 10, 7)
        }
    }
}

impl Goal for WanderAroundGoal {
    fn tick_reach(&self, _mob: &dyn Mob, _running: bool) -> Option<GoalReach> {
        Some(GoalReach {
            speed: self.speed,
            entities: self.entity_reach,
            ..GoalReach::default()
        })
    }

    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if let Some(start_condition) = self.start_condition
            && !start_condition(mob)
        {
            return false;
        }

        // Every mob that carries a passenger is controlled by it.
        if mob.get_entity().has_passengers() {
            return false;
        }

        if !self.force_trigger {
            if self.check_no_action_time
                && mob
                    .get_mob_entity()
                    .living_entity
                    .no_action_time
                    .load(std::sync::atomic::Ordering::Relaxed)
                    >= NO_ACTION_TIME_LIMIT
            {
                return false;
            }

            if mob
                .get_random()
                .random_range(0..to_goal_ticks(self.interval))
                != 0
            {
                return false;
            }
        }

        self.target = self.get_position(mob);
        if self.target.is_none() {
            return false;
        }
        self.force_trigger = false;
        true
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let idle = mob.is_navigator_idle();
        !idle && !mob.get_entity().has_passengers()
    }

    fn start(&mut self, mob: &dyn Mob) {
        if let Some(target) = self.target {
            let pos = mob.get_mob_entity().living_entity.entity.pos.load();
            mob.get_mob_entity()
                .steered()
                .navigator()
                .set_progress(NavigatorGoal::new(pos, target, self.speed));
        }
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.target = None;
        mob.get_mob_entity().steered().navigator().stop();
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{Entity, ai::test_support::world, passive::cow::CowEntity};
    use pumpkin_data::entity::EntityType;
    use std::sync::atomic::Ordering::Relaxed;

    #[tokio::test]
    async fn idle_mobs_only_stroll_when_forced() -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let cow = CowEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.0, 64.0, 0.0),
            &EntityType::COW,
        ));
        let mut goal = WanderAroundGoal::with_interval(1.0, 1)
            .with_position(|_| Some(Vector3::new(1.0, 64.0, 0.0)), 0.0);
        let idle = &cow.get_mob_entity().living_entity.no_action_time;
        idle.store(99, Relaxed);
        assert!(goal.can_start(cow.as_ref()));
        idle.store(100, Relaxed);
        assert!(!goal.can_start(cow.as_ref()));
        goal.trigger();
        assert!(goal.can_start(cow.as_ref()));
        assert!(!goal.can_start(cow.as_ref()));
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn idle_golems_can_start_village_goals() -> Result<(), Box<dyn std::error::Error>> {
        use crate::entity::ai::goal::{
            golem_random_stroll_in_village::golem_random_stroll_in_village,
            move_back_to_village::move_back_to_village,
        };
        use crate::entity::passive::iron_golem::IronGolemEntity;

        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let golem = IronGolemEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.0, 64.0, 0.0),
            &EntityType::IRON_GOLEM,
        ));
        golem
            .get_mob_entity()
            .living_entity
            .no_action_time
            .store(100, Relaxed);
        for goal in [
            golem_random_stroll_in_village(0.6),
            move_back_to_village(0.6),
        ] {
            let mut goal = goal
                .with_start_condition(|_| true)
                .with_position(|_| Some(Vector3::new(1.0, 64.0, 0.0)), 0.0);
            goal.set_interval(1);
            assert!(goal.can_start(golem.as_ref()));
        }
        world.level.shutdown().await;
        Ok(())
    }
}
