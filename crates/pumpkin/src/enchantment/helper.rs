use crate::enchantment::effects::EnchantmentEntityEffectExt;
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
        let world = projectile_entity.world.load_full();
        Self::run_iteration_on_item(weapon, |enchantment, level| {
            for conditional_effect in enchantment.effects.projectile_spawned {
                conditional_effect.effect.apply(
                    &world,
                    level,
                    None,
                    Some(projectile_entity),
                    projectile_entity.pos.load(),
                );
                if let EnchantmentEntityEffect::Ignite { .. } = &conditional_effect.effect
                    && let Some(arrow) = arrow
                {
                    arrow.set_flame(true);
                }
            }
        });
    }

    /// Applies post-attack enchantment effects (e.g. Fire Aspect ignites victim).
    pub fn on_post_attack(attacker: &Entity, victim: &Entity, weapon: &ItemStack) {
        Self::run_iteration_on_item(weapon, |enchantment, level| {
            for targeted_effect in enchantment.effects.post_attack {
                let target = match targeted_effect.affected {
                    Some(EnchantmentTarget::Attacker | EnchantmentTarget::DamagingEntity) => {
                        attacker
                    }
                    Some(EnchantmentTarget::Victim) | None => victim,
                };
                let world = target.world.load_full();
                targeted_effect
                    .effect
                    .apply(&world, level, None, Some(target), target.pos.load());
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

    /// Modifies durability change using data-driven item damage effects (e.g. Unbreaking).
    #[must_use]
    pub fn modify_durability_change(item: &ItemStack, base_change: f32) -> f32 {
        let mut change = base_change;
        if let Some(enchantments) = item.get_data_component::<EnchantmentsImpl>() {
            for (enchantment, level) in enchantments.enchantment.iter() {
                for effect in enchantment.effects.item_damage {
                    change = effect.effect.process(*level, change);
                }
            }
        }
        change
    }

    /// Modifies block experience using data-driven block experience effects (e.g. Fortune).
    #[must_use]
    pub fn modify_block_experience(tool: &ItemStack, base_xp: i32) -> i32 {
        let mut xp = base_xp as f32;
        if let Some(enchantments) = tool.get_data_component::<EnchantmentsImpl>() {
            for (enchantment, level) in enchantments.enchantment.iter() {
                enchantment.modify_block_experience(*level, &mut xp);
            }
        }
        (xp as i32).max(0)
    }

    /// Modifies mob experience using data-driven mob experience effects (e.g. Looting).
    #[must_use]
    pub fn modify_mob_experience(weapon: &ItemStack, base_xp: i32) -> i32 {
        let mut xp = base_xp as f32;
        if let Some(enchantments) = weapon.get_data_component::<EnchantmentsImpl>() {
            for (enchantment, level) in enchantments.enchantment.iter() {
                enchantment.modify_mob_experience(*level, &mut xp);
            }
        }
        (xp as i32).max(0)
    }

    /// Modifies durability to repair from experience using data-driven effects (e.g. Mending).
    #[must_use]
    pub fn modify_durability_to_repair_from_xp(item: &ItemStack, base_repair: f32) -> f32 {
        let mut repair = base_repair;
        if let Some(enchantments) = item.get_data_component::<EnchantmentsImpl>() {
            for (enchantment, level) in enchantments.enchantment.iter() {
                enchantment.modify_durability_to_repair_from_xp(*level, &mut repair);
            }
        }
        repair
    }

    /// Modifies trident return acceleration using data-driven effects (e.g. Loyalty).
    #[must_use]
    pub fn modify_trident_return_acceleration(trident: &ItemStack, base_accel: f32) -> f32 {
        let mut accel = base_accel;
        if let Some(enchantments) = trident.get_data_component::<EnchantmentsImpl>() {
            for (enchantment, level) in enchantments.enchantment.iter() {
                enchantment.modify_trident_return_to_owner_acceleration(*level, &mut accel);
            }
        }
        accel
    }

    /// Modifies trident spin attack strength using data-driven effects (e.g. Riptide).
    #[must_use]
    pub fn modify_trident_spin_attack_strength(trident: &ItemStack, base_strength: f32) -> f32 {
        let mut strength = base_strength;
        if let Some(enchantments) = trident.get_data_component::<EnchantmentsImpl>() {
            for (enchantment, level) in enchantments.enchantment.iter() {
                enchantment.modify_trident_spin_attack_strength(*level, &mut strength);
            }
        }
        strength
    }

    /// Modifies fishing time reduction using data-driven effects (e.g. Lure).
    #[must_use]
    pub fn modify_fishing_time_reduction(rod: &ItemStack, base_reduction: f32) -> f32 {
        let mut reduction = base_reduction;
        if let Some(enchantments) = rod.get_data_component::<EnchantmentsImpl>() {
            for (enchantment, level) in enchantments.enchantment.iter() {
                enchantment.modify_fishing_time_reduction(*level, &mut reduction);
            }
        }
        reduction
    }

    /// Modifies fishing luck bonus using data-driven effects (e.g. Luck of the Sea).
    #[must_use]
    pub fn modify_fishing_luck_bonus(rod: &ItemStack, base_luck: f32) -> f32 {
        let mut luck = base_luck;
        if let Some(enchantments) = rod.get_data_component::<EnchantmentsImpl>() {
            for (enchantment, level) in enchantments.enchantment.iter() {
                enchantment.modify_fishing_luck_bonus(*level, &mut luck);
            }
        }
        luck
    }

    /// Modifies damage protection across equipped armor items.
    #[must_use]
    pub fn modify_damage_protection<'a>(
        armor_items: impl IntoIterator<Item = &'a ItemStack>,
        base_protection: f32,
    ) -> f32 {
        let mut protection = base_protection;
        for item in armor_items {
            if let Some(enchantments) = item.get_data_component::<EnchantmentsImpl>() {
                for (enchantment, level) in enchantments.enchantment.iter() {
                    for effect in enchantment.effects.damage_protection {
                        protection = effect.effect.process(*level, protection);
                    }
                }
            }
        }
        protection
    }

    /// Applies location-changed enchantment effects (e.g. Frost Walker replacing water with frosted ice).
    pub fn on_location_changed(
        entity: &Entity,
        item: &ItemStack,
        position: pumpkin_util::math::vector3::Vector3<f64>,
    ) {
        let world = entity.world.load_full();
        Self::run_iteration_on_item(item, |enchantment, level| {
            for conditional_effect in enchantment.effects.location_changed {
                conditional_effect
                    .effect
                    .apply(&world, level, None, Some(entity), position);
            }
        });
    }

    /// Applies hit-block enchantment effects.
    pub fn on_hit_block(
        weapon: &ItemStack,
        projectile_entity: &Entity,
        position: pumpkin_util::math::vector3::Vector3<f64>,
    ) {
        let world = projectile_entity.world.load_full();
        Self::run_iteration_on_item(weapon, |enchantment, level| {
            for conditional_effect in enchantment.effects.hit_block {
                conditional_effect.effect.apply(
                    &world,
                    level,
                    None,
                    Some(projectile_entity),
                    position,
                );
            }
        });
    }
}
