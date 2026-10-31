//! Goals cats and ocelots share

use std::sync::Weak;

use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;

use crate::entity::ai::behavior::fleeing::Flee;
use crate::entity::ai::goal::breed::BreedGoal;
use crate::entity::ai::goal::goal_selector::GoalSelector;
use crate::entity::ai::goal::leap_at_target::LeapAtTargetGoal;
use crate::entity::ai::goal::look_at_entity::LookAtEntityGoal;
use crate::entity::ai::goal::ocelot_attack::OcelotAttackGoal;
use crate::entity::ai::goal::revenge::MobFilter;
use crate::entity::ai::goal::tempt::TemptGoal;
use crate::entity::ai::goal::wander_around::WanderAroundGoal;
use crate::entity::ai::target_match::TargetMatch;
use crate::entity::mob::Mob;

pub const TEMPT_ITEMS: &[&Item] = &[&Item::COD, &Item::SALMON];

#[must_use]
pub fn tempt_goal() -> Box<TemptGoal> {
    Box::new(TemptGoal::new(0.6, TEMPT_ITEMS, true))
}

/// Shy of players while `gate` holds: untamed cats, wary ocelots.
#[must_use]
pub const fn flee_players(priority: u8, gate: MobFilter) -> Flee {
    Flee::new(
        priority,
        TargetMatch::Type(&EntityType::PLAYER),
        16.0,
        0.8,
        1.33,
    )
    .as_long_as(gate)
}

/// Leap, pounce, breed, stroll and look at players, on consecutive priorities from `first`.
pub fn apply_tail(goals: &GoalSelector, mob: Weak<dyn Mob>, first: u8) {
    goals.add_goal(first, Box::new(LeapAtTargetGoal::new(0.3)));
    goals.add_goal(first + 1, Box::new(OcelotAttackGoal::new()));
    goals.add_goal(first + 2, BreedGoal::new(0.8));
    goals.add_goal(first + 3, Box::new(WanderAroundGoal::new(0.8)));
    goals.add_goal(
        first + 4,
        LookAtEntityGoal::with_default(mob, &EntityType::PLAYER, 10.0),
    );
}
