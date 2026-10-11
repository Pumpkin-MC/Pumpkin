use crate::entity::ai::goal::active_target::{ActiveTargetGoal, TargetCondition};
use crate::entity::ai::goal::goal_selector::GoalSelector;
use crate::entity::ai::goal::revenge::MobFilter;
use crate::entity::ai::target_match::TargetMatch;
use crate::entity::living::LivingEntity;
use crate::entity::mob::MobEntity;
use crate::world::World;

pub type TargetPredicateFn = fn(&LivingEntity, &World) -> bool;

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
    predicate: Option<TargetPredicateFn>,
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
            predicate: None,
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

    /// Extra test on each candidate, vanilla's selector.
    #[must_use]
    pub const fn matching(mut self, predicate: TargetPredicateFn) -> Self {
        self.predicate = Some(predicate);
        self
    }

    fn build(self, mob: &MobEntity) -> Box<ActiveTargetGoal> {
        let goal = Box::new(ActiveTargetGoal::new(
            mob,
            self.target,
            self.reciprocal_chance,
            self.must_see,
            false,
            self.predicate,
        ))
        .when(self.condition);
        match self.only {
            Some(filter) => goal.only_mobs(filter),
            None => goal,
        }
    }
}

/// Registers every entry in order.
pub fn apply(targets: &mut GoalSelector, mob: &MobEntity, hunts: &[Hunt]) {
    for hunt in hunts {
        targets.add_goal(hunt.priority, hunt.build(mob));
    }
}
