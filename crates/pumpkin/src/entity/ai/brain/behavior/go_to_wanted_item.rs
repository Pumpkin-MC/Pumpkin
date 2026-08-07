//! Port of `behavior/GoToWantedItem.java` (declarative, so a `OneShot` here).

use crate::entity::ai::brain::Brain;
use crate::entity::ai::brain::behavior::{Behavior, OneShot, OneShotTrigger};
use crate::entity::ai::brain::memory::{
    ItemPickupCooldownTicksMemory, LookTargetMemory, MemoryKeyId, MemoryStatus,
    NearestVisibleWantedItemMemory, PositionTracker, WalkTarget, WalkTargetMemory,
};
use crate::entity::mob::Mob;

pub struct GoToWantedItem {
    speed_modifier: f32,
    max_dist_to_walk: f64,
}

impl GoToWantedItem {
    /// `GoToWantedItem.create(predicate, speedModifier, interruptOngoingWalk, maxDistToWalk)`
    /// (`GoToWantedItem.java:17-51`). Allay passes `interruptOngoingWalk = true`
    /// (`AllayAi.java:81`), which makes the `WALK_TARGET` condition `REGISTERED` rather than
    /// `VALUE_ABSENT` -- i.e. this behavior is allowed to stomp an in-flight walk target.
    ///
    /// The `predicate` parameter is not modelled: Allay passes `mob -> true`.
    // Returns a boxed trait object, not Self by name -- constructor pattern for this behavior/sensor family.
    #[allow(clippy::new_ret_no_self)]
    #[must_use]
    pub fn new(
        speed_modifier: f32,
        interrupt_ongoing_walk: bool,
        max_dist_to_walk: f64,
    ) -> Box<dyn Behavior> {
        let walk_condition = if interrupt_ongoing_walk {
            MemoryStatus::Registered
        } else {
            MemoryStatus::ValueAbsent
        };
        Box::new(OneShot::new(
            Self {
                speed_modifier,
                max_dist_to_walk,
            },
            vec![
                (MemoryKeyId::LookTarget, MemoryStatus::Registered),
                (MemoryKeyId::WalkTarget, walk_condition),
                (
                    MemoryKeyId::NearestVisibleWantedItem,
                    MemoryStatus::ValuePresent,
                ),
                (
                    MemoryKeyId::ItemPickupCooldownTicks,
                    MemoryStatus::Registered,
                ),
            ],
        ))
    }
}

impl OneShotTrigger for GoToWantedItem {
    fn debug_name(&self) -> &'static str {
        "GoToWantedItem"
    }

    /// `GoToWantedItem.java:33-46`.
    ///
    /// DEVIATION: the world-border containment check
    /// (`body.level().getWorldBorder().isWithinBounds(item.blockPosition())`) is not ported.
    fn trigger(&mut self, mob: &dyn Mob, brain: &Brain, _game_time: i64) -> bool {
        if brain.has_value::<ItemPickupCooldownTicksMemory>() {
            return false;
        }
        if !mob.can_pick_up_loot() {
            return false;
        }
        let Some(item) = brain
            .get::<NearestVisibleWantedItemMemory>()
            .and_then(|weak| weak.upgrade())
        else {
            return false;
        };

        let mob_pos = mob.get_mob_entity().living_entity.entity.pos.load();
        let item_pos = item.get_entity().pos.load();
        if item_pos.squared_distance_to_vec(&mob_pos)
            > self.max_dist_to_walk * self.max_dist_to_walk
        {
            return false;
        }

        brain.set::<LookTargetMemory>(PositionTracker::of_entity(&item, true));
        brain.set::<WalkTargetMemory>(WalkTarget::new(
            PositionTracker::of_entity(&item, false),
            self.speed_modifier,
            0,
        ));
        true
    }
}
