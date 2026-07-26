use pumpkin_data::Enchantment;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use rand::RngExt;

use super::{MOB_SPAWN_ENCHANT_COST_SPAN, MOB_SPAWN_ENCHANT_MIN_COST};

// ══════════════════════════════════════════════════════════════════
// Enchantment pools — curated per item category, filtered by
// equipment slot and exclusive set at application time
// ══════════════════════════════════════════════════════════════════

static MELEE_WEAPON_ENCHANTS: [&Enchantment; 9] = [
    &Enchantment::SHARPNESS,
    &Enchantment::SMITE,
    &Enchantment::BANE_OF_ARTHROPODS,
    &Enchantment::KNOCKBACK,
    &Enchantment::FIRE_ASPECT,
    &Enchantment::LOOTING,
    &Enchantment::SWEEPING_EDGE,
    &Enchantment::UNBREAKING,
    &Enchantment::MENDING,
];

static TRIDENT_ENCHANTS: [&Enchantment; 6] = [
    &Enchantment::IMPALING,
    &Enchantment::CHANNELING,
    &Enchantment::RIPTIDE,
    &Enchantment::LOYALTY,
    &Enchantment::UNBREAKING,
    &Enchantment::MENDING,
];

static BOW_ENCHANTS: [&Enchantment; 6] = [
    &Enchantment::POWER,
    &Enchantment::PUNCH,
    &Enchantment::FLAME,
    &Enchantment::INFINITY,
    &Enchantment::UNBREAKING,
    &Enchantment::MENDING,
];

static CROSSBOW_ENCHANTS: [&Enchantment; 5] = [
    &Enchantment::QUICK_CHARGE,
    &Enchantment::MULTISHOT,
    &Enchantment::PIERCING,
    &Enchantment::UNBREAKING,
    &Enchantment::MENDING,
];

static FISHING_ROD_ENCHANTS: [&Enchantment; 4] = [
    &Enchantment::LUCK_OF_THE_SEA,
    &Enchantment::LURE,
    &Enchantment::UNBREAKING,
    &Enchantment::MENDING,
];

static HEAD_ARMOR_ENCHANTS: [&Enchantment; 9] = [
    &Enchantment::PROTECTION,
    &Enchantment::FIRE_PROTECTION,
    &Enchantment::BLAST_PROTECTION,
    &Enchantment::PROJECTILE_PROTECTION,
    &Enchantment::RESPIRATION,
    &Enchantment::AQUA_AFFINITY,
    &Enchantment::THORNS,
    &Enchantment::UNBREAKING,
    &Enchantment::MENDING,
];

static CHEST_ARMOR_ENCHANTS: [&Enchantment; 7] = [
    &Enchantment::PROTECTION,
    &Enchantment::FIRE_PROTECTION,
    &Enchantment::BLAST_PROTECTION,
    &Enchantment::PROJECTILE_PROTECTION,
    &Enchantment::THORNS,
    &Enchantment::UNBREAKING,
    &Enchantment::MENDING,
];

static LEGS_ARMOR_ENCHANTS: [&Enchantment; 8] = [
    &Enchantment::PROTECTION,
    &Enchantment::FIRE_PROTECTION,
    &Enchantment::BLAST_PROTECTION,
    &Enchantment::PROJECTILE_PROTECTION,
    &Enchantment::THORNS,
    &Enchantment::UNBREAKING,
    &Enchantment::MENDING,
    &Enchantment::SWIFT_SNEAK,
];

static FEET_ARMOR_ENCHANTS: [&Enchantment; 11] = [
    &Enchantment::PROTECTION,
    &Enchantment::FIRE_PROTECTION,
    &Enchantment::BLAST_PROTECTION,
    &Enchantment::PROJECTILE_PROTECTION,
    &Enchantment::FEATHER_FALLING,
    &Enchantment::DEPTH_STRIDER,
    &Enchantment::FROST_WALKER,
    &Enchantment::SOUL_SPEED,
    &Enchantment::THORNS,
    &Enchantment::UNBREAKING,
    &Enchantment::MENDING,
];

// ══════════════════════════════════════════════════════════════════
// Enchantment system — mimics vanilla EnchantmentsByCostWithDifficulty
//
// Weighted selection with exclusive-set conflict resolution and cost-based
// level calculation. Uses curated flat pools per equipment category instead
// of the datapack-based enchantment_provider/mob_spawn_equipment.json.
// ══════════════════════════════════════════════════════════════════

/// Random enchantment cost in `[min, min + specialMultiplier * span]`.
#[must_use]
fn spawn_enchant_cost(special_multiplier: f32) -> i32 {
    let min = MOB_SPAWN_ENCHANT_MIN_COST;
    let max = min + (special_multiplier * MOB_SPAWN_ENCHANT_COST_SPAN as f32).round() as i32;
    let mut rng = rand::rng();
    rng.random_range(min..=max)
}

/// Returns the highest enchantment level whose cost is affordable.
#[must_use]
fn enchantment_level_from_cost(enchant: &Enchantment, cost: i32) -> i32 {
    for lvl in (1..=enchant.max_level).rev() {
        if cost >= enchant.min_cost.calculate(lvl) {
            return lvl;
        }
    }
    1
}

/// Selects the enchantment pool for an item/slot combination.
///
/// Uses a curated flat pool per equipment category (melee, trident, bow,
/// crossbow, fishing rod, and per-armor-slot). This is an approximation of
/// vanilla's data-driven `mob_spawn_equipment` enchantment provider which
/// filters by `supported_items` tags.
#[must_use]
fn enchant_pool_for(item: &Item, slot: &EquipmentSlot) -> &'static [&'static Enchantment] {
    let key = item.registry_key;
    if key.contains("sword")
        || key.contains("spear")
        || key.contains("axe")
        || key.contains("shovel")
    {
        &MELEE_WEAPON_ENCHANTS
    } else if key.contains("trident") {
        &TRIDENT_ENCHANTS
    } else if key.contains("bow") {
        &BOW_ENCHANTS
    } else if key.contains("crossbow") {
        &CROSSBOW_ENCHANTS
    } else if key.contains("fishing_rod") {
        &FISHING_ROD_ENCHANTS
    } else if *slot == EquipmentSlot::HEAD {
        &HEAD_ARMOR_ENCHANTS
    } else if *slot == EquipmentSlot::CHEST {
        &CHEST_ARMOR_ENCHANTS
    } else if *slot == EquipmentSlot::LEGS {
        &LEGS_ARMOR_ENCHANTS
    } else if *slot == EquipmentSlot::FEET {
        &FEET_ARMOR_ENCHANTS
    } else {
        &[]
    }
}

/// Checks whether `candidate` conflicts with any already-applied enchantment
/// via vanilla exclusive sets (e.g. `exclusive_set_damage`).
#[must_use]
fn conflicts_with(candidate: &Enchantment, applied: &[&Enchantment]) -> bool {
    if let Some(excl) = candidate.exclusive_set {
        let excl_keys = excl.0;
        for existing in applied {
            if excl_keys.contains(&existing.registry_key) {
                return true;
            }
        }
    }
    false
}

/// Applies multiple enchantments to a stack using weighted pool selection.
///
/// Starts with a random cost (scaled by `special_multiplier`), picks
/// enchantments by weight, resolves exclusive-set conflicts, and determines
/// the level from the remaining cost. Cost is halved each iteration so
/// later enchantments receive lower levels.
pub(super) fn apply_vanilla_enchantments(
    stack: &mut ItemStack,
    slot: &EquipmentSlot,
    special_multiplier: f32,
) {
    let pool = enchant_pool_for(stack.item, slot);
    if pool.is_empty() {
        return;
    }

    let mut cost = spawn_enchant_cost(special_multiplier);
    let mut applied: Vec<&Enchantment> = Vec::new();
    let mut rng = rand::rng();

    loop {
        let candidates: Vec<&Enchantment> = pool
            .iter()
            .copied()
            .filter(|e| !applied.contains(e) && !conflicts_with(e, &applied))
            .collect();

        if candidates.is_empty() {
            break;
        }

        let total_weight: f32 = candidates.iter().map(|e| e.weight as f32).sum();
        if total_weight <= 0.0 {
            break;
        }

        let mut roll = rng.random_range(0.0..total_weight);
        let mut selected: Option<&Enchantment> = None;
        for e in &candidates {
            roll -= e.weight as f32;
            if roll <= 0.0 {
                selected = Some(e);
                break;
            }
        }
        let selected = selected.unwrap_or(candidates.last().copied().unwrap());

        let level = enchantment_level_from_cost(selected, cost);
        stack.add_enchantment(selected, level.clamp(1, selected.max_level) as u16);
        applied.push(selected);

        cost /= 2;
        if cost < 1 {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enchantment_level_tracks_affordable_cost() {
        // A huge budget always affords the maximum level.
        assert_eq!(
            enchantment_level_from_cost(&Enchantment::SMITE, 10_000),
            Enchantment::SMITE.max_level
        );
        // A zero budget falls back to level 1.
        assert_eq!(enchantment_level_from_cost(&Enchantment::SMITE, 0), 1);
    }

    #[test]
    fn pools_are_selected_by_item_category_and_slot() {
        let melee = enchant_pool_for(&Item::IRON_SWORD, &EquipmentSlot::MAIN_HAND);
        assert_eq!(melee.as_ptr(), MELEE_WEAPON_ENCHANTS.as_ptr());

        let trident = enchant_pool_for(&Item::TRIDENT, &EquipmentSlot::MAIN_HAND);
        assert_eq!(trident.as_ptr(), TRIDENT_ENCHANTS.as_ptr());

        let rod = enchant_pool_for(&Item::FISHING_ROD, &EquipmentSlot::MAIN_HAND);
        assert_eq!(rod.as_ptr(), FISHING_ROD_ENCHANTS.as_ptr());

        let helmet = enchant_pool_for(&Item::IRON_HELMET, &EquipmentSlot::HEAD);
        assert_eq!(helmet.as_ptr(), HEAD_ARMOR_ENCHANTS.as_ptr());

        let boots = enchant_pool_for(&Item::IRON_BOOTS, &EquipmentSlot::FEET);
        assert_eq!(boots.as_ptr(), FEET_ARMOR_ENCHANTS.as_ptr());
    }

    #[test]
    fn exclusive_sets_reject_conflicting_enchantments() {
        // Smite and Sharpness share the damage exclusive set.
        assert!(conflicts_with(
            &Enchantment::SMITE,
            &[&Enchantment::SHARPNESS],
        ));
        // Unbreaking has no exclusive set at all.
        assert!(!conflicts_with(
            &Enchantment::UNBREAKING,
            &[&Enchantment::SHARPNESS],
        ));
        assert!(!conflicts_with(&Enchantment::SMITE, &[]));
    }

    #[test]
    fn spawn_cost_stays_within_the_configured_bounds() {
        for _ in 0..64 {
            let cost = spawn_enchant_cost(1.0);
            assert!(cost >= MOB_SPAWN_ENCHANT_MIN_COST);
            assert!(cost <= MOB_SPAWN_ENCHANT_MIN_COST + MOB_SPAWN_ENCHANT_COST_SPAN);
        }
        // A zero multiplier pins the cost to the minimum.
        assert_eq!(spawn_enchant_cost(0.0), MOB_SPAWN_ENCHANT_MIN_COST);
    }
}
