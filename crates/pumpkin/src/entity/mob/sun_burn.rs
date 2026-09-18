use std::sync::atomic::Ordering::Relaxed;

use pumpkin_data::item_stack::ItemStack;

use crate::entity::mob::Mob;
use crate::entity::{Entity, EntityBase, equipment_break_status};
use crate::world::brightness::DAYLIGHT_BRIGHTNESS;

/// Fire an unprotected mob is set alight for.
const BURN_SECONDS: f32 = 8.0;

/// Vanilla `isSunBurnTick` dice: rarer the dimmer the sun. `roll` is 0..1.
#[must_use]
pub fn passes_burn_roll(sunlight: f32, roll: f32) -> bool {
    sunlight > DAYLIGHT_BRIGHTNESS && roll * 30.0 < (sunlight - 0.4) * 2.0
}

/// Sun reaches the mob this tick and the dice agree.
fn is_sun_burn_tick(entity: &Entity) -> bool {
    let world = entity.world.load();
    let eye = entity.get_eye_pos().to_block_pos();

    if !world.monsters_burn(&eye) {
        return false;
    }
    // One sky light read serves the brightness and the open-sky check.
    let sky_light = world.get_sky_light_level(&eye);
    // Nearly every tick ends at the roll: the sky and wet checks stay behind it.
    if !passes_burn_roll(world.sunlight_from_sky_light(sky_light), rand::random()) {
        return false;
    }
    if !world.can_see_sky_with_light(&eye, sky_light) {
        return false;
    }

    !(entity.touching_water.load(Relaxed)
        || entity.is_in_powder_snow()
        || entity.was_in_powder_snow.load(Relaxed)
        || world.is_raining_at(&eye))
}

/// An item in the protection slot takes the sun instead of the mob.
#[must_use]
pub fn is_protected(mob: &dyn Mob) -> bool {
    !mob.get_mob_entity()
        .living_entity
        .entity_equipment
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get(&mob.sun_protection_slot())
        .is_empty()
}

/// Wears the protection item down, or sets an unprotected mob on fire.
fn burn(mob: &dyn Mob) {
    let mob_entity = mob.get_mob_entity();
    let entity = &mob_entity.living_entity.entity;
    let slot = mob.sun_protection_slot();

    let mut stack = mob_entity
        .living_entity
        .entity_equipment
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get(&slot);

    if stack.is_empty() {
        entity.set_on_fire_for(BURN_SECONDS);
        return;
    }
    if !stack.is_damageable() || stack.is_unbreakable() {
        return;
    }

    // Vanilla wears mob gear by 0 or 1, unaffected by Unbreaking.
    let worn = stack.get_damage() + rand::random_range(0..2);
    if worn >= stack.get_max_damage().unwrap_or(i32::MAX) {
        mob_entity.set_item_slot(&slot, ItemStack::EMPTY.clone());
        entity
            .world
            .load()
            .send_entity_status(entity, equipment_break_status(&slot), None);
    } else if worn != stack.get_damage() {
        stack.set_damage(worn);
        mob_entity.set_item_slot(&slot, stack);
    }
}

/// Daylight burning, once per tick for every mob.
pub fn tick(mob: &dyn Mob) {
    let entity = &mob.get_mob_entity().living_entity.entity;
    if !entity.is_alive() || !mob.burns_in_daylight() {
        return;
    }
    if is_sun_burn_tick(entity) {
        burn(mob);
    }
}

#[cfg(test)]
mod tests {
    use super::passes_burn_roll;

    #[test]
    fn dim_light_never_burns() {
        for roll in [0.0, 0.5, 0.99] {
            assert!(!passes_burn_roll(0.5, roll));
            assert!(!passes_burn_roll(0.1, roll));
        }
    }

    #[test]
    fn full_sun_burns_on_a_low_roll_only() {
        // (1.0 - 0.4) * 2 = 1.2, so the roll has to land under 1.2 / 30 = 0.04.
        assert!(passes_burn_roll(1.0, 0.0));
        assert!(passes_burn_roll(1.0, 0.039));
        assert!(!passes_burn_roll(1.0, 0.041));
        assert!(!passes_burn_roll(1.0, 0.99));
    }

    #[test]
    fn dimmer_sun_burns_less_often() {
        assert!(passes_burn_roll(1.0, 0.03));
        assert!(!passes_burn_roll(0.6, 0.03));
    }
}
