use pumpkin_util::math::vector3::Vector3;

use super::super::BrainTick;
use super::super::memory::{MemoryModuleId, MemoryModuleType, MemoryStatus, types};
use super::timed::Behavior;
use super::{PositionGetter, utils};

const CLOSE_ENOUGH_DISTANCE_TO_TARGET: i32 = 3;

pub type ItemThrower = fn(&mut BrainTick<'_>, Vector3<f64>);

pub struct GoAndGiveItemsToTarget {
    target_position_getter: PositionGetter,
    speed_modifier: f32,
    timeout_duration: i32,
    item_thrower: ItemThrower,
    cooldown_memory: MemoryModuleType<i32>,
    cooldown_duration: i32,
    has_item_predicate: fn(&BrainTick<'_>) -> bool,
    entry_conditions: [(MemoryModuleId, MemoryStatus); 3],
}

impl GoAndGiveItemsToTarget {
    #[must_use]
    pub const fn new(
        target_position_getter: PositionGetter,
        speed_modifier: f32,
        timeout_duration: i32,
        item_thrower: ItemThrower,
        cooldown_memory: MemoryModuleType<i32>,
        cooldown_duration: i32,
        has_item_predicate: fn(&BrainTick<'_>) -> bool,
    ) -> Self {
        Self {
            target_position_getter,
            speed_modifier,
            timeout_duration,
            item_thrower,
            cooldown_memory,
            cooldown_duration,
            has_item_predicate,
            entry_conditions: [
                (types::LOOK_TARGET.id(), MemoryStatus::Registered),
                (types::WALK_TARGET.id(), MemoryStatus::Registered),
                (cooldown_memory.id(), MemoryStatus::Registered),
            ],
        }
    }

    fn can_throw_item_to_target(&self, tick: &mut BrainTick<'_>) -> bool {
        (self.has_item_predicate)(tick) && (self.target_position_getter)(tick).is_some()
    }
}

impl Behavior for GoAndGiveItemsToTarget {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        &self.entry_conditions
    }

    fn min_duration(&self) -> i32 {
        self.timeout_duration
    }

    fn max_duration(&self) -> i32 {
        self.timeout_duration
    }

    fn check_extra_start_conditions(&mut self, tick: &mut BrainTick<'_>) -> bool {
        self.can_throw_item_to_target(tick)
    }

    fn can_still_use(&mut self, tick: &mut BrainTick<'_>) -> bool {
        self.can_throw_item_to_target(tick)
    }

    fn start(&mut self, tick: &mut BrainTick<'_>) {
        if let Some(target) = (self.target_position_getter)(tick) {
            utils::set_walk_and_look_target_memories(
                tick.state,
                target,
                self.speed_modifier,
                CLOSE_ENOUGH_DISTANCE_TO_TARGET,
            );
        }
    }

    fn tick(&mut self, tick: &mut BrainTick<'_>) {
        let Some(target) = (self.target_position_getter)(tick) else {
            return;
        };
        let deposit_position = target.current_position();
        if deposit_position.squared_distance_to_vec(&tick.mob.get_eye_pos())
            < f64::from(CLOSE_ENOUGH_DISTANCE_TO_TARGET * CLOSE_ENOUGH_DISTANCE_TO_TARGET)
        {
            (self.item_thrower)(tick, target.current_position());
            tick.state.set(self.cooldown_memory, self.cooldown_duration);
        }
    }

    fn debug_name(&self) -> &'static str {
        "GoAndGiveItemsToTarget"
    }
}
