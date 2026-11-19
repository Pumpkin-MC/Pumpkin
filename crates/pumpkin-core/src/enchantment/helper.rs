use std::sync::Arc;

use crate::enchantment::effects::{DamageEntity, EnchantmentEntityEffectExt};
use crate::entity::living::{LivingEntity, attribute_modifier_slot_matches};
use crate::entity::player::Player;
use crate::entity::projectile::arrow::ArrowEntity;
use crate::entity::{Entity, EntityBase};
use crate::world::World;
use crate::world::loot::LootContextParameters;
use pumpkin_data::damage::DamageType;
use pumpkin_data::data_component_impl::{EnchantmentsImpl, EquipmentSlot};
use pumpkin_data::enchantment::{Enchantment, EnchantmentEntityEffect, EnchantmentTarget};
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_inventory::Inventory;
use pumpkin_util::loot_table::LootCondition;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::random::{RandomGenerator, get_seed, xoroshiro128::Xoroshiro};

/// Data-driven helper for enchantment effects matching vanilla `EnchantmentHelper`.
pub struct EnchantmentHelper;

impl EnchantmentHelper {
    /// Returns the highest level on equipment slots allowed by the enchantment.
    #[must_use]
    pub fn get_enchantment_level(
        enchantment: &'static Enchantment,
        caller: &dyn crate::entity::EntityBase,
    ) -> i32 {
        use crate::entity::living::attribute_modifier_slot_matches;
        use pumpkin_data::data_component_impl::EquipmentSlot;
        use pumpkin_inventory::Inventory;
        let Some(living) = caller.get_living_entity() else {
            return 0;
        };
        let level = |slot: &EquipmentSlot, stack: &ItemStack| {
            if !stack.is_empty()
                && enchantment
                    .slots
                    .iter()
                    .any(|group| attribute_modifier_slot_matches(group, slot))
            {
                stack.get_enchantment_level(enchantment)
            } else {
                0
            }
        };
        caller.get_player().map_or_else(
            || {
                living
                    .entity_equipment
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .equipment
                    .iter()
                    .map(|(slot, stack)| level(slot, stack))
                    .max()
                    .unwrap_or(0)
            },
            |player| {
                let main_hand = level(&EquipmentSlot::MAIN_HAND, &living.held_item(caller));
                player
                    .inventory
                    .equipment_slots
                    .iter()
                    .map(|(index, slot)| level(slot, &player.inventory.get_stack(*index)))
                    .max()
                    .unwrap_or(0)
                    .max(main_hand)
            },
        )
    }

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

    /// Applies the post-attack enchantment effects of the victim's equipment and the attacker's
    /// weapon, mirroring vanilla `doPostAttackEffectsWithItemSource`.
    pub fn on_post_attack(
        attacker: &dyn EntityBase,
        victim: &dyn EntityBase,
        weapon: &ItemStack,
        damage_type: DamageType,
    ) {
        let world = victim.get_entity().world.load_full();
        Self::apply_post_attack_effects(
            &world,
            victim,
            Some(weapon),
            Some(attacker),
            Some(attacker),
            damage_type,
        );
    }

    fn equipped_stacks(
        living: &LivingEntity,
        player: Option<&Player>,
    ) -> Vec<(EquipmentSlot, ItemStack)> {
        player.map_or_else(
            || {
                living
                    .entity_equipment
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .equipment
                    .iter()
                    .map(|(slot, stack)| (slot.clone(), stack.clone()))
                    .collect()
            },
            |player| {
                let mut stacks = Vec::with_capacity(player.inventory.equipment_slots.len() + 1);
                stacks.push((EquipmentSlot::MAIN_HAND, player.inventory.held_item()));
                for (index, slot) in player.inventory.equipment_slots.iter() {
                    stacks.push((slot.clone(), player.inventory.get_stack(*index)));
                }
                stacks
            },
        )
    }

    fn requirements_match(
        requirements: Option<LootCondition>,
        params: &LootContextParameters<'_>,
        level: i32,
    ) -> bool {
        requirements.is_none_or(|requirements| {
            let params = LootContextParameters {
                enchantment_level: Some(level),
                ..params.clone()
            };
            crate::world::loot::test_loot_condition(requirements, &params)
        })
    }

    fn post_attack_target<'a>(
        affected: Option<EnchantmentTarget>,
        victim: &'a dyn EntityBase,
        attacker: Option<&'a dyn EntityBase>,
        direct_entity: Option<&'a dyn EntityBase>,
    ) -> Option<&'a Entity> {
        match affected? {
            EnchantmentTarget::Attacker => attacker.map(EntityBase::get_entity),
            EnchantmentTarget::DamagingEntity => direct_entity.map(EntityBase::get_entity),
            EnchantmentTarget::Victim => Some(victim.get_entity()),
        }
    }

    fn apply_post_attack_effect(
        world: &Arc<World>,
        level: i32,
        effect: &EnchantmentEntityEffect,
        owner: Option<&Arc<Player>>,
        target: &Entity,
        holder: &dyn EntityBase,
        slot: &EquipmentSlot,
    ) {
        match effect {
            EnchantmentEntityEffect::AllOf(effects) => {
                for effect in *effects {
                    Self::apply_post_attack_effect(
                        world, level, effect, owner, target, holder, slot,
                    );
                }
            }
            EnchantmentEntityEffect::ChangeItemDamage { amount } => {
                if let Some(living) = holder.get_living_entity() {
                    living.damage_equipped_item(world, slot, amount.calculate(level) as i32);
                }
            }
            EnchantmentEntityEffect::DamageEntity {
                min_damage,
                max_damage,
                damage_type,
            } => {
                let mut rng = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(get_seed()));
                let damage =
                    DamageEntity::new(min_damage.clone(), max_damage.clone(), *damage_type)
                        .calculate_damage(level, &mut rng);
                let damage_type = damage_type.copied().unwrap_or(DamageType::GENERIC);
                let target_entity = world
                    .get_player_by_id(target.entity_id)
                    .map(|p| p as Arc<dyn EntityBase>)
                    .or_else(|| world.get_entity_by_id(target.entity_id));
                if let Some(target_entity) = target_entity {
                    target_entity.damage_with_context(
                        target_entity.as_ref(),
                        damage,
                        damage_type,
                        None,
                        Some(holder),
                        Some(holder),
                    );
                }
            }
            _ => effect.apply(world, level, owner, Some(target), target.pos.load()),
        }
    }

    fn apply_post_attack_effects(
        world: &Arc<World>,
        victim: &dyn EntityBase,
        weapon: Option<&ItemStack>,
        attacker: Option<&dyn EntityBase>,
        direct_entity: Option<&dyn EntityBase>,
        damage_type: DamageType,
    ) {
        let victim_entity = victim.get_entity();
        let params = LootContextParameters {
            world: Some(world.as_ref()),
            this_entity: Some(victim),
            killer_entity: attacker,
            direct_killer_entity: direct_entity,
            position: Some(victim_entity.pos.load()),
            damage_type: Some(damage_type),
            ..Default::default()
        };

        if let Some(living) = victim.get_living_entity() {
            let owner = world.get_player_by_id(victim_entity.entity_id);
            for (slot, stack) in Self::equipped_stacks(living, owner.as_deref()) {
                Self::run_iteration_on_item(&stack, |enchantment, level| {
                    if !enchantment
                        .slots
                        .iter()
                        .any(|group| attribute_modifier_slot_matches(group, &slot))
                    {
                        return;
                    }
                    for targeted_effect in enchantment.get_post_attack_effects() {
                        if targeted_effect.enchanted != Some(EnchantmentTarget::Victim)
                            || !Self::requirements_match(
                                targeted_effect.requirements,
                                &params,
                                level,
                            )
                        {
                            continue;
                        }
                        if let Some(target) = Self::post_attack_target(
                            targeted_effect.affected,
                            victim,
                            attacker,
                            direct_entity,
                        ) {
                            Self::apply_post_attack_effect(
                                world,
                                level,
                                &targeted_effect.effect,
                                owner.as_ref(),
                                target,
                                victim,
                                &slot,
                            );
                        }
                    }
                });
            }
        }

        if let Some(weapon) = weapon
            && let Some(attacker) = attacker
        {
            let owner = world.get_player_by_id(attacker.get_entity().entity_id);
            Self::run_iteration_on_item(weapon, |enchantment, level| {
                if !enchantment
                    .slots
                    .iter()
                    .any(|group| attribute_modifier_slot_matches(group, &EquipmentSlot::MAIN_HAND))
                {
                    return;
                }
                for targeted_effect in enchantment.get_post_attack_effects() {
                    if targeted_effect.enchanted != Some(EnchantmentTarget::Attacker)
                        || !Self::requirements_match(targeted_effect.requirements, &params, level)
                    {
                        continue;
                    }
                    if let Some(target) = Self::post_attack_target(
                        targeted_effect.affected,
                        victim,
                        Some(attacker),
                        direct_entity,
                    ) {
                        Self::apply_post_attack_effect(
                            world,
                            level,
                            &targeted_effect.effect,
                            owner.as_ref(),
                            target,
                            attacker,
                            &EquipmentSlot::MAIN_HAND,
                        );
                    }
                }
            });
        }
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

    /// Modifies damage against a specific target type (e.g. Impaling, Smite, Bane of Arthropods, Sharpness).
    #[must_use]
    pub fn modify_damage_against(
        weapon: &ItemStack,
        base_damage: f64,
        victim_type: Option<&'static EntityType>,
    ) -> f64 {
        let mut damage = base_damage;
        if let Some(enchantments) = weapon.get_data_component::<EnchantmentsImpl>() {
            for (enchantment, level) in enchantments.enchantment.iter() {
                enchantment.modify_damage_against(*level, &mut damage, victim_type);
            }
        }
        damage
    }

    /// Applies post-attack enchantment effects for a thrown trident hitting `victim`.
    pub fn on_trident_post_attack(
        world: &Arc<World>,
        trident: &Entity,
        owner: Option<&dyn EntityBase>,
        victim: &dyn EntityBase,
        weapon: &ItemStack,
    ) {
        let attacker = owner.unwrap_or(trident);
        Self::apply_post_attack_effects(
            world,
            victim,
            Some(weapon),
            Some(attacker),
            Some(trident),
            DamageType::TRIDENT,
        );
    }

    /// Applies `hit_block` enchantment effects for a thrown trident hitting `block_pos`.
    pub fn on_trident_hit_block(
        world: &Arc<World>,
        trident: &Entity,
        owner: Option<&Arc<Player>>,
        block_pos: &BlockPos,
        hit_pos: Vector3<f64>,
        weapon: &ItemStack,
    ) {
        const CLAMP_EPSILON: f64 = 1.0e-5f32 as f64;
        let bx = f64::from(block_pos.0.x);
        let by = f64::from(block_pos.0.y);
        let bz = f64::from(block_pos.0.z);
        let compensated_pos = Vector3::new(
            hit_pos
                .x
                .clamp(bx + CLAMP_EPSILON, bx + 1.0 - CLAMP_EPSILON),
            hit_pos
                .y
                .clamp(by + CLAMP_EPSILON, by + 1.0 - CLAMP_EPSILON),
            hit_pos
                .z
                .clamp(bz + CLAMP_EPSILON, bz + 1.0 - CLAMP_EPSILON),
        );
        let clamped_block_pos = BlockPos::floored_v(compensated_pos);

        Self::run_iteration_on_item(weapon, |enchantment, level| {
            for conditional_effect in enchantment.effects.hit_block {
                if *enchantment == Enchantment::CHANNELING {
                    if world.is_thunderstorm()
                        && world
                            .get_block(&clamped_block_pos)
                            .has_tag(&tag::Block::MINECRAFT_LIGHTNING_RODS)
                        && world.can_see_sky(&clamped_block_pos)
                    {
                        conditional_effect.effect.apply(
                            world,
                            level,
                            owner,
                            Some(trident),
                            compensated_pos,
                        );
                    }
                } else {
                    conditional_effect.effect.apply(
                        world,
                        level,
                        owner,
                        Some(trident),
                        compensated_pos,
                    );
                }
            }
        });
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
