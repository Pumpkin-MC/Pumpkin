use std::sync::Arc;

use pumpkin_data::data_component_impl::EquipmentSlot;

use crate::entity::mob::Mob;

use super::super::BrainTick;
use super::super::memory::position_tracker::{EntityTracker, PositionTracker};
use super::super::memory::{MemoryStatus, types};
use super::one_shot::OneShot;

#[must_use]
pub fn melee_attack(
    can_attack: impl Fn(&BrainTick<'_>) -> bool + Send + Sync + 'static,
    cooldown_between_attacks: i32,
) -> OneShot {
    OneShot::with_required(
        "MeleeAttack",
        vec![
            (types::ATTACK_TARGET.id(), MemoryStatus::ValuePresent),
            (types::ATTACK_COOLING_DOWN.id(), MemoryStatus::ValueAbsent),
            (
                types::NEAREST_VISIBLE_LIVING_ENTITIES.id(),
                MemoryStatus::ValuePresent,
            ),
        ],
        vec![types::LOOK_TARGET.id()],
        move |tick| {
            if !can_attack(tick) || is_holding_usable_non_melee_weapon(tick.mob) {
                return false;
            }
            let target = {
                let ctx = tick.visibility();
                let Some(target) = ctx.brain.get(types::ATTACK_TARGET).map(Arc::clone) else {
                    return false;
                };
                let in_range = ctx.mob.get_mob_entity().is_in_attack_range(target.as_ref());
                let visible = ctx
                    .brain
                    .get(types::NEAREST_VISIBLE_LIVING_ENTITIES)
                    .is_some_and(|visible| visible.contains(target.as_ref(), &ctx));
                (in_range && visible).then_some(target)
            };
            let Some(target) = target else {
                return false;
            };

            tick.brain.set(
                types::LOOK_TARGET,
                Arc::new(EntityTracker::new(Arc::clone(&target), true)) as Arc<dyn PositionTracker>,
            );
            let mob_entity = tick.mob.get_mob_entity();
            mob_entity.living_entity.swing_hand();
            mob_entity.try_attack(tick.mob.get_entity(), target.as_ref());
            tick.brain.set_with_expiry(
                types::ATTACK_COOLING_DOWN,
                true,
                i64::from(cooldown_between_attacks),
            );
            true
        },
    )
}

fn is_holding_usable_non_melee_weapon(mob: &dyn Mob) -> bool {
    let Ok(equipment) = mob
        .get_mob_entity()
        .living_entity
        .entity_equipment
        .try_lock()
    else {
        return false;
    };
    let (main_hand, off_hand) = (
        equipment.get(&EquipmentSlot::MAIN_HAND),
        equipment.get(&EquipmentSlot::OFF_HAND),
    );
    drop(equipment);
    mob.can_use_non_melee_weapon(&main_hand) || mob.can_use_non_melee_weapon(&off_hand)
}
