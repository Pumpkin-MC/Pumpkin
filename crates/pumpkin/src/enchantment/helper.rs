use crate::enchantment::effects::Ignite;
use crate::entity::Entity;
use crate::entity::projectile::arrow::ArrowEntity;
use pumpkin_data::data_component_impl::EnchantmentsImpl;
use pumpkin_data::enchantment::{Enchantment, EnchantmentEntityEffect, EnchantmentTarget};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;

/// Data-driven helper for enchantment effects matching vanilla `EnchantmentHelper`.
pub struct EnchantmentHelper;

impl EnchantmentHelper {
    /// Iterates through enchantments on an item stack, matching vanilla `runIterationOnItem`.
    pub fn run_iteration_on_item<F>(item_stack: &ItemStack, mut visitor: F)
    where
        F: FnMut(&'static Enchantment, i32),
    {
        if let Some(enchantments) = item_stack.get_data_component::<EnchantmentsImpl>() {
            for (enchantment, level) in enchantments.enchantment.iter() {
                visitor(enchantment, *level);
            }
        }
    }

    /// Applies projectile-spawned enchantment effects (e.g. Flame ignites the projectile for 100s).
    pub fn on_projectile_spawned(
        weapon: &ItemStack,
        projectile_entity: &Entity,
        arrow: Option<&ArrowEntity>,
    ) {
        Self::run_iteration_on_item(weapon, |enchantment, level| {
            for conditional_effect in enchantment.effects.projectile_spawned {
                if let EnchantmentEntityEffect::Ignite { duration } = &conditional_effect.effect {
                    let ignite = Ignite::new(duration.clone());
                    ignite.apply(level, projectile_entity);
                    if let Some(arrow) = arrow {
                        arrow.set_flame(true);
                    }
                }
            }
        });
    }

    /// Applies post-attack enchantment effects (e.g. Fire Aspect ignites victim).
    pub fn on_post_attack(_attacker: &Entity, victim: &Entity, weapon: &ItemStack) {
        Self::run_iteration_on_item(weapon, |enchantment, level| {
            for targeted_effect in enchantment.effects.post_attack {
                if targeted_effect.affected == Some(EnchantmentTarget::Victim)
                    && let EnchantmentEntityEffect::Ignite { duration } = &targeted_effect.effect
                {
                    let ignite = Ignite::new(duration.clone());
                    ignite.apply(level, victim);
                }
            }
        });
    }

    /// Computes projectile spread angle using data-driven projectile spread effects.
    #[must_use]
    pub fn process_projectile_spread(weapon: &ItemStack, base_spread: f32) -> f32 {
        let mut spread = base_spread;
        if let Some(enchantments) = weapon.get_data_component::<EnchantmentsImpl>() {
            for (enchantment, level) in enchantments.enchantment.iter() {
                enchantment.modify_projectile_spread(*level, &mut spread);
            }
        }
        spread
    }

    /// Computes projectile count using data-driven projectile count effects.
    #[must_use]
    pub fn process_projectile_count(weapon: &ItemStack, base_count: usize) -> usize {
        let mut count = base_count as f32;
        if let Some(enchantments) = weapon.get_data_component::<EnchantmentsImpl>() {
            for (enchantment, level) in enchantments.enchantment.iter() {
                enchantment.modify_projectile_count(*level, &mut count);
            }
        }
        count as usize
    }

    /// Computes projectile piercing level using data-driven piercing effects.
    #[must_use]
    pub fn process_projectile_piercing(weapon: &ItemStack, base_piercing: u8) -> u8 {
        let mut piercing = f32::from(base_piercing);
        if let Some(enchantments) = weapon.get_data_component::<EnchantmentsImpl>() {
            for (enchantment, level) in enchantments.enchantment.iter() {
                enchantment.modify_piercing_count(*level, &mut piercing);
            }
        }
        piercing as u8
    }

    /// Computes ammo use using data-driven ammo use effects (e.g. Infinity).
    #[must_use]
    pub fn process_ammo_use(weapon: &ItemStack, projectile: &ItemStack, base_ammo: i32) -> i32 {
        let mut ammo = base_ammo as f32;
        if projectile.item.id == Item::ARROW.id
            && let Some(enchantments) = weapon.get_data_component::<EnchantmentsImpl>()
        {
            for (enchantment, level) in enchantments.enchantment.iter() {
                enchantment.modify_ammo_count(*level, &mut ammo);
            }
        }
        ammo as i32
    }

    /// Modifies damage using data-driven damage effects (e.g. Power / Sharpness).
    #[must_use]
    pub fn modify_damage(weapon: &ItemStack, base_damage: f64) -> f64 {
        let mut damage = base_damage;
        if let Some(enchantments) = weapon.get_data_component::<EnchantmentsImpl>() {
            for (enchantment, level) in enchantments.enchantment.iter() {
                enchantment.modify_damage(*level, &mut damage);
            }
        }
        damage
    }

    /// Modifies smash / fall-based damage using data-driven effects (e.g. Density).
    #[must_use]
    pub fn modify_fall_based_damage(weapon: &ItemStack, base_damage: f64) -> f64 {
        let mut damage = base_damage;
        if let Some(enchantments) = weapon.get_data_component::<EnchantmentsImpl>() {
            for (enchantment, level) in enchantments.enchantment.iter() {
                enchantment.modify_fall_based_damage(*level, &mut damage);
            }
        }
        damage
    }

    /// Modifies knockback using data-driven knockback effects (e.g. Punch / Knockback).
    #[must_use]
    pub fn modify_knockback(weapon: &ItemStack, base_knockback: f32) -> f32 {
        let mut knockback = base_knockback;
        if let Some(enchantments) = weapon.get_data_component::<EnchantmentsImpl>() {
            for (enchantment, level) in enchantments.enchantment.iter() {
                enchantment.modify_knockback(*level, &mut knockback);
            }
        }
        knockback
    }

    /// Modifies armor effectiveness using data-driven effects (e.g. Breach).
    #[must_use]
    pub fn modify_armor_effectiveness(weapon: &ItemStack, base_effectiveness: f32) -> f32 {
        let mut effectiveness = base_effectiveness;
        if let Some(enchantments) = weapon.get_data_component::<EnchantmentsImpl>() {
            for (enchantment, level) in enchantments.enchantment.iter() {
                enchantment.modify_armor_effectiveness(*level, &mut effectiveness);
            }
        }
        effectiveness
    }

    /// Modifies crossbow charge time using data-driven charge time effects (e.g. Quick Charge).
    #[must_use]
    pub fn modify_crossbow_charge_time(weapon: &ItemStack, base_ticks: i32) -> i32 {
        let mut charge_time = base_ticks as f32;
        if let Some(enchantments) = weapon.get_data_component::<EnchantmentsImpl>() {
            for (enchantment, level) in enchantments.enchantment.iter() {
                let mut change_sec = 0.0f32;
                enchantment.modify_crossbow_charge_time(*level, &mut change_sec);
                charge_time += change_sec * 20.0;
            }
        }
        (charge_time as i32).max(0)
    }
}
