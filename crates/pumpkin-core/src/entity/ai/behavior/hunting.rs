use pumpkin_data::entity::EntityType;

use crate::entity::ai::goal::active_target::{ActiveTargetGoal, TargetCondition};
use crate::entity::ai::goal::goal_selector::GoalSelector;
use crate::entity::ai::goal::revenge::MobFilter;
use crate::entity::ai::target_match::TargetMatch;
use crate::entity::living::LivingEntity;
use crate::entity::mob::{Mob, MobEntity};
use crate::entity::passive::turtle::is_baby_on_land;
use crate::world::World;

/// Vanilla's target selector: hunter, candidate.
pub type TargetPredicateFn = fn(&dyn Mob, &LivingEntity, &World) -> bool;

/// One vanilla `NearestAttackableTargetGoal`, defaults as its short constructor:
/// reciprocal chance 10, no reach check.
#[derive(Clone, Copy)]
pub struct Hunt {
    priority: u8,
    target: TargetMatch,
    must_see: bool,
    reciprocal_chance: i32,
    condition: TargetCondition,
    only: Option<MobFilter>,
    gate: Option<MobFilter>,
    predicate: Option<TargetPredicateFn>,
    unseen_memory_ticks: Option<i32>,
    range_scale: f64,
}

impl Hunt {
    #[must_use]
    pub const fn new(priority: u8, target: TargetMatch, must_see: bool) -> Self {
        Self {
            priority,
            target,
            must_see,
            reciprocal_chance: 10,
            condition: TargetCondition::Always,
            only: None,
            gate: None,
            predicate: None,
            unseen_memory_ticks: None,
            range_scale: 1.0,
        }
    }

    /// Search roughly once per `chance` ticks, 0 every tick.
    #[must_use]
    pub const fn every(mut self, chance: i32) -> Self {
        self.reciprocal_chance = chance;
        self
    }

    /// See [`ActiveTargetGoal::when`].
    #[must_use]
    pub const fn when(mut self, condition: TargetCondition) -> Self {
        self.condition = condition;
        self
    }

    /// See [`ActiveTargetGoal::only_mobs`].
    #[must_use]
    pub const fn only_mobs(mut self, filter: MobFilter) -> Self {
        self.only = Some(filter);
        self
    }

    /// See [`ActiveTargetGoal::gated_by`].
    #[must_use]
    pub const fn gated_by(mut self, gate: MobFilter) -> Self {
        self.gate = Some(gate);
        self
    }

    /// Extra test on each candidate, vanilla's selector.
    #[must_use]
    pub const fn matching(mut self, predicate: TargetPredicateFn) -> Self {
        self.predicate = Some(predicate);
        self
    }

    /// Ticks an unseen target is kept, vanilla `setUnseenMemoryTicks`.
    #[must_use]
    pub const fn forget_unseen_after(mut self, ticks: i32) -> Self {
        self.unseen_memory_ticks = Some(ticks);
        self
    }

    /// Follow range factor for searching and keeping the target.
    #[must_use]
    pub const fn range_scale(mut self, scale: f64) -> Self {
        self.range_scale = scale;
        self
    }

    fn build(self, mob: &MobEntity) -> Box<ActiveTargetGoal> {
        let mut goal = ActiveTargetGoal::with_default(mob, self.target, self.must_see)
            .every(self.reciprocal_chance)
            .when(self.condition)
            .scale_range(self.range_scale);
        if let Some(predicate) = self.predicate {
            goal = goal.selecting(predicate);
        }
        if let Some(filter) = self.only {
            goal = goal.only_mobs(filter);
        }
        if let Some(gate) = self.gate {
            goal = goal.gated_by(gate);
        }
        if let Some(ticks) = self.unseen_memory_ticks {
            goal = goal.forget_unseen_after(ticks);
        }
        goal
    }
}

/// Registers every entry in order.
pub fn apply(targets: &GoalSelector, mob: &MobEntity, hunts: &[Hunt]) {
    for hunt in hunts {
        targets.add_goal(hunt.priority, hunt.build(mob));
    }
}

/// Vanilla `Turtle.BABY_ON_LAND_SELECTOR` hunts.
#[must_use]
pub const fn baby_turtles(priority: u8, must_see: bool) -> Hunt {
    Hunt::new(priority, TargetMatch::Type(&EntityType::TURTLE), must_see).only_mobs(is_baby_on_land)
}

/// Cube mobs and ghasts: target at most 4 blocks above or below.
pub fn within_four_blocks_height(hunter: &dyn Mob, target: &LivingEntity, _world: &World) -> bool {
    let hunter_y = hunter.get_mob_entity().living_entity.entity.pos.load().y;
    (target.entity.pos.load().y - hunter_y).abs() <= 4.0
}
