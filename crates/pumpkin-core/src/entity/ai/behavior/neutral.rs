use pumpkin_data::entity::EntityType;

use crate::entity::ai::goal::active_target::{ActiveTargetGoal, TargetCondition};
use crate::entity::ai::goal::goal_selector::GoalSelector;
use crate::entity::ai::goal::reset_universal_anger::ResetUniversalAngerGoal;
use crate::entity::mob::MobEntity;

/// Target pair every neutral mob shares: go for a player only while holding a grudge,
/// and swap that grudge for a universal one when the gamerule says so.
///
/// The revenge goal stays at the call site -> its priority and alert filter differ per mob.
pub fn apply_targets(
    targets: &mut GoalSelector,
    mob: &MobEntity,
    angry_priority: u8,
    reset_priority: u8,
    alert_others: bool,
) {
    targets.add_goal(
        angry_priority,
        ActiveTargetGoal::with_default(mob, &EntityType::PLAYER, true)
            .when(TargetCondition::AngryAt),
    );
    targets.add_goal(reset_priority, ResetUniversalAngerGoal::new(alert_others));
}
