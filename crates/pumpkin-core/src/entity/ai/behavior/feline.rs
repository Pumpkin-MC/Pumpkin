//! Goals cats and ocelots share

use std::sync::Weak;

use pumpkin_data::entity::{EntityPose, EntityType};

use crate::entity::ai::behavior::fleeing::Flee;
use crate::entity::ai::goal::breed::BreedGoal;
use crate::entity::ai::goal::goal_selector::GoalSelector;
use crate::entity::ai::goal::leap_at_target::LeapAtTargetGoal;
use crate::entity::ai::goal::look_at_entity::LookAtEntityGoal;
use crate::entity::ai::goal::ocelot_attack::OcelotAttackGoal;
use crate::entity::ai::goal::revenge::MobFilter;
use crate::entity::ai::goal::wander_around::WanderAroundGoal;
use crate::entity::ai::target_match::TargetMatch;
use crate::entity::mob::Mob;

pub const TEMPT_SPEED: f64 = 0.6;
pub const SPRINT_SPEED: f64 = 1.33;
const STROLL_PROBABILITY: f32 = 1.000_000_1E-5;

/// Shy of players while `gate` holds: untamed cats, wary ocelots.
#[must_use]
pub const fn flee_players(priority: u8, gate: MobFilter) -> Flee {
    Flee::new(
        priority,
        TargetMatch::Type(&EntityType::PLAYER),
        16.0,
        0.8,
        SPRINT_SPEED,
    )
    .as_long_as(gate)
}

/// Leap, pounce, breed, stroll and look at players, on consecutive priorities from `first`.
pub fn apply_tail(goals: &GoalSelector, mob: Weak<dyn Mob>, first: u8) {
    goals.add_goal(first, Box::new(LeapAtTargetGoal::new(0.3)));
    goals.add_goal(first + 1, Box::new(OcelotAttackGoal::new()));
    goals.add_goal(first + 2, BreedGoal::new(0.8));
    goals.add_goal(
        first + 3,
        Box::new(WanderAroundGoal::water_avoiding_with_probability(
            0.8,
            STROLL_PROBABILITY,
        )),
    );
    goals.add_goal(
        first + 4,
        LookAtEntityGoal::with_default(mob, &EntityType::PLAYER, 10.0),
    );
}

/// Crouches at the tempt speed and sprints at the flee speed the move control was given.
pub fn update_pose_from_speed(mob: &dyn Mob) {
    let (pose, sprinting) = {
        let steered = mob.get_mob_entity().steered();
        let move_control = steered.move_control();
        if move_control.has_wanted() {
            let speed = move_control.get_speed_modifier();
            if speed == TEMPT_SPEED {
                (EntityPose::Crouching, false)
            } else if speed == SPRINT_SPEED {
                (EntityPose::Standing, true)
            } else {
                (EntityPose::Standing, false)
            }
        } else {
            (EntityPose::Standing, false)
        }
    };
    let living = &mob.get_mob_entity().living_entity;
    living.entity.set_pose(pose);
    if living.entity.is_sprinting() != sprinting {
        living.set_sprinting(sprinting);
    }
}

/// A crouching feline steps carefully, as does a sneaking one.
#[must_use]
pub fn is_stepping_carefully(mob: &dyn Mob) -> bool {
    let entity = mob.get_entity();
    entity.pose.load() == EntityPose::Crouching || entity.is_sneaking()
}
