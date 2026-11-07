use std::sync::Arc;

use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::world::WorldEvent;
use pumpkin_util::math::position::BlockPos;
use rand::{Rng, RngExt};

use super::super::super::BrainTick;
use super::super::super::memory::position_tracker::{BlockPosTracker, PositionTracker};
use super::super::super::memory::walk_target::WalkTarget;
use super::super::super::memory::{MemoryModuleId, MemoryStatus, types};
use super::super::timed::Behavior;
use super::harvest_farmland::{is_crop_block, is_max_age};
use crate::entity::passive::villager::VillagerEntity;
use crate::item::items::bone_meal::BoneMealItem;
use crate::world::World;

const BONEMEALING_DURATION: i32 = 80;
const WORK_CYCLE_INTERVAL: i64 = 40;
const BONEMEALING_COOLDOWN: i32 = 160;
const SPEED_MODIFIER: f32 = 0.5;

const ENTRY_CONDITIONS: [(MemoryModuleId, MemoryStatus); 2] = [
    (types::LOOK_TARGET.id(), MemoryStatus::ValueAbsent),
    (types::WALK_TARGET.id(), MemoryStatus::ValueAbsent),
];

pub struct UseBonemeal {
    next_work_cycle_time: i64,
    last_bonemealing_session: i64,
    time_worked_so_far: i32,
    crop_pos: Option<BlockPos>,
}

impl UseBonemeal {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            next_work_cycle_time: 0,
            last_bonemealing_session: 0,
            time_worked_so_far: 0,
            crop_pos: None,
        }
    }

    fn pick_next_target(tick: &BrainTick<'_>) -> Option<BlockPos> {
        let center = tick.mob.get_entity().block_pos.load();
        let mut candidates = Vec::new();
        for x in -1..=1 {
            for y in -1..=1 {
                for z in -1..=1 {
                    let pos = center.add(x, y, z);
                    if valid_pos(pos, tick.world) {
                        candidates.push(pos);
                    }
                }
            }
        }
        reservoir_pick(&candidates, &mut tick.mob.get_random())
    }

    fn set_current_crop_as_target(&self, tick: &mut BrainTick<'_>) {
        if let Some(pos) = self.crop_pos {
            let tracker: Arc<dyn PositionTracker> = Arc::new(BlockPosTracker::new(pos));
            tick.state.set(types::LOOK_TARGET, Arc::clone(&tracker));
            tick.state.set(
                types::WALK_TARGET,
                WalkTarget::new(tracker, SPEED_MODIFIER, 1),
            );
        }
    }
}

impl Behavior for UseBonemeal {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        &ENTRY_CONDITIONS
    }

    fn check_extra_start_conditions(&mut self, tick: &mut BrainTick<'_>) -> bool {
        let tick_count = i64::from(
            tick.mob
                .get_entity()
                .tick_count
                .load(std::sync::atomic::Ordering::Relaxed),
        );
        if tick_count % 10 != 0
            || (self.last_bonemealing_session != 0
                && self.last_bonemealing_session + i64::from(BONEMEALING_COOLDOWN) > tick_count)
        {
            return false;
        }
        let Some(villager) = tick.mob.cast_any().downcast_ref::<VillagerEntity>() else {
            return false;
        };
        if villager.inventory_count(&Item::BONE_MEAL) <= 0 {
            return false;
        }
        self.crop_pos = Self::pick_next_target(tick);
        self.crop_pos.is_some()
    }

    fn can_still_use(&mut self, _tick: &mut BrainTick<'_>) -> bool {
        self.time_worked_so_far < BONEMEALING_DURATION && self.crop_pos.is_some()
    }

    fn start(&mut self, tick: &mut BrainTick<'_>) {
        self.set_current_crop_as_target(tick);
        tick.mob.get_mob_entity().set_item_slot(
            &EquipmentSlot::MAIN_HAND,
            ItemStack::new(1, &Item::BONE_MEAL),
        );
        self.next_work_cycle_time = tick.time;
        self.time_worked_so_far = 0;
    }

    fn stop(&mut self, tick: &mut BrainTick<'_>) {
        tick.mob
            .get_mob_entity()
            .set_item_slot(&EquipmentSlot::MAIN_HAND, ItemStack::EMPTY.clone());
        self.last_bonemealing_session = i64::from(
            tick.mob
                .get_entity()
                .tick_count
                .load(std::sync::atomic::Ordering::Relaxed),
        );
    }

    fn tick(&mut self, tick: &mut BrainTick<'_>) {
        let Some(target_pos) = self.crop_pos else {
            return;
        };
        let entity_pos = tick.mob.get_entity().pos.load();
        if tick.time < self.next_work_cycle_time
            || target_pos.dist_to_center_sqr(entity_pos.x, entity_pos.y, entity_pos.z) >= 1.0
        {
            return;
        }
        let Some(villager) = tick.mob.cast_any().downcast_ref::<VillagerEntity>() else {
            return;
        };
        let mut inventory = villager
            .inventory
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(index) = inventory
            .iter()
            .position(|stack| stack.item.id == Item::BONE_MEAL.id)
            && BoneMealItem::grow_crop(&mut inventory[index], tick.world, &target_pos)
        {
            tick.world
                .sync_world_event(WorldEvent::ParticlesAndSoundPlantGrowth, target_pos, 15);
            self.crop_pos = Self::pick_next_target(tick);
            self.set_current_crop_as_target(tick);
            self.next_work_cycle_time = tick.time + WORK_CYCLE_INTERVAL;
        }
        self.time_worked_so_far += 1;
    }

    fn debug_name(&self) -> &'static str {
        "UseBonemeal"
    }
}

impl Default for UseBonemeal {
    fn default() -> Self {
        Self::new()
    }
}

fn valid_pos(block_pos: BlockPos, world: &World) -> bool {
    let (block, state) = world.get_block_and_state(&block_pos);
    is_crop_block(block) && !is_max_age(block, state.id)
}

fn reservoir_pick<T: Copy>(candidates: &[T], random: &mut impl Rng) -> Option<T> {
    let mut result = None;
    for (index, candidate) in candidates.iter().enumerate() {
        let count = index + 1;
        if random.random_range(0..count) == 0 {
            result = Some(*candidate);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use rand::{SeedableRng, rngs::StdRng};

    use super::*;

    #[test]
    fn a_single_candidate_is_always_picked() {
        for seed in 0..16 {
            let mut random = StdRng::seed_from_u64(seed);
            assert_eq!(reservoir_pick(&[7u8], &mut random), Some(7));
        }
    }

    #[test]
    fn reservoir_sampling_reaches_every_candidate() {
        let candidates = [1u8, 2, 3];
        let mut seen = [false; 3];
        for seed in 0..64 {
            let mut random = StdRng::seed_from_u64(seed);
            if let Some(picked) = reservoir_pick(&candidates, &mut random) {
                seen[usize::from(picked - 1)] = true;
            }
        }
        assert!(seen.iter().all(|seen| *seen));
    }
}
