use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;

use super::{Controls, Goal, GoalReach, to_goal_ticks};
use crate::entity::ai::target_match::{SearchArea, TargetMatch};
use crate::entity::{EntityBase, ai::pathfinder::NavigatorGoal, mob::Mob};

const SEARCH_RADIUS: f64 = 8.0;
const MIN_DISTANCE_SQ: f64 = 9.0;
const MAX_DISTANCE_SQ: f64 = 256.0;

const SEARCH_Y_RANGE: f64 = 4.0;

pub struct FollowParentGoal {
    speed: f64,
    parent: Option<Arc<dyn EntityBase>>,
    delay: i32,
}

impl FollowParentGoal {
    #[must_use]
    pub fn new(speed: f64) -> Self {
        Self {
            speed,
            parent: None,
            delay: 0,
        }
    }

    fn find_parent(mob: &dyn Mob) -> Option<Arc<dyn EntityBase>> {
        let mob_entity = mob.get_mob_entity();
        let entity = &mob_entity.living_entity.entity;
        let pos = entity.pos.load();
        let world = entity.world.load();
        let area = SearchArea::from_feet(entity, SEARCH_RADIUS, SEARCH_Y_RANGE);

        let parent =
            TargetMatch::Type(entity.entity_type).find_nearest(&world, &area, |candidate| {
                candidate.get_entity().age.load(Relaxed) >= 0
            })?;

        // The minimum distance applies to the closest parent only: a nearby parent means "do not
        // follow", not "follow a farther one".
        let dist_sq = pos.squared_distance_to_vec(&parent.get_entity().pos.load());
        (dist_sq >= MIN_DISTANCE_SQ).then_some(parent)
    }
}

impl Goal for FollowParentGoal {
    fn tick_reach(&self, mob: &dyn Mob, _running: bool) -> Option<GoalReach> {
        let baby = mob.get_mob_entity().living_entity.entity.age.load(Relaxed) < 0;
        Some(GoalReach {
            // Adults bail out before searching.
            entities: if baby { SEARCH_RADIUS } else { 0.0 },
            speed: if baby { self.speed } else { 0.0 },
            links: self
                .parent
                .iter()
                .map(|parent| parent.get_entity().entity_id)
                .collect(),
            ..GoalReach::default()
        })
    }

    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let age = mob.get_mob_entity().living_entity.entity.age.load(Relaxed);
        if age >= 0 {
            return false;
        }
        self.parent = Self::find_parent(mob);
        self.parent.is_some()
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let age = mob.get_mob_entity().living_entity.entity.age.load(Relaxed);
        if age >= 0 {
            return false;
        }
        let Some(parent) = &self.parent else {
            return false;
        };
        let parent_entity = parent.get_entity();
        if !parent_entity.is_alive() {
            return false;
        }
        let mob_pos = mob.get_mob_entity().living_entity.entity.pos.load();
        let parent_pos = parent_entity.pos.load();
        let dist_sq = mob_pos.squared_distance_to_vec(&parent_pos);
        (MIN_DISTANCE_SQ..=MAX_DISTANCE_SQ).contains(&dist_sq)
    }

    fn start(&mut self, _mob: &dyn Mob) {
        self.delay = 0;
    }

    fn tick(&mut self, mob: &dyn Mob) {
        self.delay -= 1;
        if self.delay > 0 {
            return;
        }
        self.delay = to_goal_ticks(10);
        if let Some(parent) = &self.parent {
            let mob_pos = mob.get_mob_entity().living_entity.entity.pos.load();
            let parent_pos = parent.get_entity().pos.load();
            let steered = mob.get_mob_entity().steered();
            let mut navigator = steered.navigator();
            navigator.set_progress(NavigatorGoal::new(mob_pos, parent_pos, self.speed));
        }
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        self.parent = None;
    }

    fn controls(&self) -> Controls {
        Controls::empty()
    }
}
