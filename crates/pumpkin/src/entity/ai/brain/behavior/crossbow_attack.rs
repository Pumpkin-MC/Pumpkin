use std::sync::Arc;

use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::item::Item;
use rand::RngExt;

use crate::entity::EntityBase;
use crate::entity::ai::goal::ranged_crossbow_attack::RangedCrossbowAttackGoal;
use crate::entity::mob::Mob;

use super::super::BrainTick;
use super::super::memory::position_tracker::{EntityTracker, PositionTracker};
use super::super::memory::{MemoryModuleId, MemoryStatus, types};
use super::timed::Behavior;
use super::utils::{can_see, is_within_attack_range};

const TIMEOUT: i32 = 1200;

#[derive(Clone, Copy, PartialEq, Eq)]
enum CrossbowState {
    Uncharged,
    Charging,
    Charged,
    ReadyToAttack,
}

pub struct CrossbowAttack {
    conditions: [(MemoryModuleId, MemoryStatus); 1],
    attack_delay: i32,
    crossbow_state: CrossbowState,
}

impl Default for CrossbowAttack {
    fn default() -> Self {
        Self::new()
    }
}

impl CrossbowAttack {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            conditions: [(types::ATTACK_TARGET.id(), MemoryStatus::ValuePresent)],
            attack_delay: 0,
            crossbow_state: CrossbowState::Uncharged,
        }
    }

    fn able_to_attack(tick: &BrainTick<'_>) -> bool {
        let ctx = tick.visibility();
        let Some(target) = ctx.brain.get(types::ATTACK_TARGET) else {
            return false;
        };
        is_holding_crossbow(ctx.mob)
            && can_see(&ctx, target.as_ref())
            && is_within_attack_range(ctx.mob, target.as_ref(), 0)
    }

    fn crossbow_attack(&mut self, tick: &BrainTick<'_>, target: &Arc<dyn EntityBase>) {
        let living = &tick.mob.get_mob_entity().living_entity;
        match self.crossbow_state {
            CrossbowState::Uncharged => {
                start_using_crossbow(tick.mob);
                self.crossbow_state = CrossbowState::Charging;
                set_charging(tick.mob, true);
            }
            CrossbowState::Charging => {
                // Release while the hand is still active; a cleared hand is not a charge.
                if is_using_item(tick.mob) {
                    if living
                        .item_use_time
                        .load(std::sync::atomic::Ordering::Relaxed)
                        <= 1
                    {
                        living.clear_active_hand();
                        self.crossbow_state = CrossbowState::Charged;
                        self.attack_delay = 20 + tick.mob.get_random().random_range(0..20);
                        set_charging(tick.mob, false);
                    }
                } else {
                    self.crossbow_state = CrossbowState::Uncharged;
                }
            }
            CrossbowState::Charged => {
                self.attack_delay -= 1;
                if self.attack_delay == 0 {
                    self.crossbow_state = CrossbowState::ReadyToAttack;
                }
            }
            CrossbowState::ReadyToAttack => {
                RangedCrossbowAttackGoal::shoot(tick.mob, target);
                if let Some(crossbow_mob) = tick.mob.as_crossbow_attack_mob() {
                    crossbow_mob.on_crossbow_attack_performed();
                }
                self.crossbow_state = CrossbowState::Uncharged;
            }
        }
    }
}

impl Behavior for CrossbowAttack {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        &self.conditions
    }

    fn min_duration(&self) -> i32 {
        TIMEOUT
    }

    fn max_duration(&self) -> i32 {
        TIMEOUT
    }

    fn check_extra_start_conditions(&mut self, tick: &mut BrainTick<'_>) -> bool {
        Self::able_to_attack(tick)
    }

    fn can_still_use(&mut self, tick: &BrainTick<'_>) -> bool {
        tick.brain.has_memory_value(types::ATTACK_TARGET.id()) && Self::able_to_attack(tick)
    }

    fn tick(&mut self, tick: &mut BrainTick<'_>) {
        let Some(target) = tick.brain.get(types::ATTACK_TARGET).cloned() else {
            return;
        };
        tick.brain.set(
            types::LOOK_TARGET,
            Arc::new(EntityTracker::new(Arc::clone(&target), true)) as Arc<dyn PositionTracker>,
        );
        self.crossbow_attack(tick, &target);
    }

    fn stop(&mut self, tick: &mut BrainTick<'_>) {
        if is_using_item(tick.mob) {
            tick.mob.get_mob_entity().living_entity.clear_active_hand();
        }
        if is_holding_crossbow(tick.mob) {
            set_charging(tick.mob, false);
        }
    }

    fn debug_name(&self) -> &'static str {
        "CrossbowAttack"
    }
}

fn is_holding_crossbow(mob: &dyn Mob) -> bool {
    let equipment = mob
        .get_mob_entity()
        .living_entity
        .entity_equipment
        .try_lock();
    equipment.is_ok_and(|equipment| {
        equipment.get(&EquipmentSlot::MAIN_HAND).item.id == Item::CROSSBOW.id
            || equipment.get(&EquipmentSlot::OFF_HAND).item.id == Item::CROSSBOW.id
    })
}

fn is_using_item(mob: &dyn Mob) -> bool {
    mob.get_mob_entity()
        .living_entity
        .item_in_use
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .is_some()
}

fn set_charging(mob: &dyn Mob, charging: bool) {
    if let Some(crossbow_mob) = mob.as_crossbow_attack_mob() {
        crossbow_mob.set_charging_crossbow(charging);
    }
}

fn start_using_crossbow(mob: &dyn Mob) {
    let living = &mob.get_mob_entity().living_entity;
    let equipment = living.entity_equipment.try_lock();
    let Ok(equipment) = equipment else {
        return;
    };
    let main_hand = equipment.get(&EquipmentSlot::MAIN_HAND);
    let (hand, stack) = if main_hand.item.id == Item::CROSSBOW.id {
        (pumpkin_util::Hand::Right, main_hand)
    } else {
        (
            pumpkin_util::Hand::Left,
            equipment.get(&EquipmentSlot::OFF_HAND),
        )
    };
    drop(equipment);
    living.set_active_hand(hand, stack, RangedCrossbowAttackGoal::CHARGE_DURATION);
}
