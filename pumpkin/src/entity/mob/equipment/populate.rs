use std::sync::Arc;
use std::sync::LazyLock;

use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_util::difficulty::Difficulty;
use rand::RngExt;

use crate::entity::EntityBase;

use super::enchant::apply_vanilla_enchantments;
use super::{
    ARMOR_ENCHANT_CHANCE, ARMOR_UPGRADE_MATERIAL_ATTEMPTS, ARMOR_UPGRADE_MATERIAL_CHANCE,
    ArmorConfig, DEFAULT_EQUIPMENT_DROP_CHANCE, EQUIPMENT_REGISTRY, MobEquipmentDef,
    RegionalDifficulty, WEAPON_ENCHANT_CHANCE, WEARING_ARMOR_CHANCE, WeaponConfig, WeaponEntry,
};

// ══════════════════════════════════════════════════════════════════
// Armor tiers — exact match to vanilla Mob.getEquipmentForSlot()
// Vanilla approximation: armor type selection (base 0-2 + 3 upgrade
// attempts at 10.87% per vanilla Mob.populateDefaultEquipmentSlots) and
// partial armor chance (0.1 on Hard / 0.25 otherwise).
// Type 0=Leather, 1=Copper, 2=Gold, 3=Chainmail, 4=Iron, 5=Diamond
// Slot order: HEAD, CHEST, LEGS, FEET
// ══════════════════════════════════════════════════════════════════

static ARMOR_TIERS: LazyLock<[[&'static Item; 4]; 6]> = LazyLock::new(|| {
    [
        [
            &Item::LEATHER_HELMET,
            &Item::LEATHER_CHESTPLATE,
            &Item::LEATHER_LEGGINGS,
            &Item::LEATHER_BOOTS,
        ],
        [
            &Item::COPPER_HELMET,
            &Item::COPPER_CHESTPLATE,
            &Item::COPPER_LEGGINGS,
            &Item::COPPER_BOOTS,
        ],
        [
            &Item::GOLDEN_HELMET,
            &Item::GOLDEN_CHESTPLATE,
            &Item::GOLDEN_LEGGINGS,
            &Item::GOLDEN_BOOTS,
        ],
        [
            &Item::CHAINMAIL_HELMET,
            &Item::CHAINMAIL_CHESTPLATE,
            &Item::CHAINMAIL_LEGGINGS,
            &Item::CHAINMAIL_BOOTS,
        ],
        [
            &Item::IRON_HELMET,
            &Item::IRON_CHESTPLATE,
            &Item::IRON_LEGGINGS,
            &Item::IRON_BOOTS,
        ],
        [
            &Item::DIAMOND_HELMET,
            &Item::DIAMOND_CHESTPLATE,
            &Item::DIAMOND_LEGGINGS,
            &Item::DIAMOND_BOOTS,
        ],
    ]
});

static ARMOR_POPULATION_ORDER: [EquipmentSlot; 4] = [
    EquipmentSlot::HEAD,
    EquipmentSlot::CHEST,
    EquipmentSlot::LEGS,
    EquipmentSlot::FEET,
];

// ══════════════════════════════════════════════════════════════════
// Equipment population
//
// Mirrors mob-specific `finalizeSpawn` / `populateDefaultEquipmentSlots`
// from Vanilla's Zombie, AbstractSkeleton, WitherSkeleton, Piglin,
// Pillager, Vindicator, Drowned, and ZombifiedPiglin.
// ══════════════════════════════════════════════════════════════════

/// Weighted random selection from a table of weapon entries.
#[must_use]
fn weighted_select_item(items: &[WeaponEntry]) -> &'static Item {
    let total: f32 = items.iter().map(|e| e.weight).sum();
    let mut rng = rand::rng();
    let mut roll: f32 = rng.random_range(0.0..total);
    for entry in items {
        roll -= entry.weight;
        if roll <= 0.0 {
            return entry.item;
        }
    }
    items.last().unwrap().item
}

/// Selects armor using the vanilla algorithm.
///
/// 1. Random base tier (0-2) with up to 3 upgrade attempts at 10.87% each.
/// 2. Iterates HEAD→CHEST→LEGS→FEET, with a chance to stop early (10% Hard,
///    25% otherwise) — higher difficulty produces fewer pieces.
/// 3. Each piece gets the default equipment drop chance.
#[must_use]
fn select_vanilla_armor(difficulty: &RegionalDifficulty) -> Vec<(EquipmentSlot, ItemStack, f32)> {
    let mut rng = rand::rng();

    let mut armor_type = rng.random_range(0..3);
    let mut i = 1;
    while (i as f32) <= ARMOR_UPGRADE_MATERIAL_ATTEMPTS {
        if rng.random::<f32>() < ARMOR_UPGRADE_MATERIAL_CHANCE {
            armor_type += 1;
        }
        i += 1;
    }
    armor_type = armor_type.min(5);

    let tier = &ARMOR_TIERS[armor_type];

    let partial_chance = if difficulty.base_difficulty == Difficulty::Hard {
        0.1f32
    } else {
        0.25f32
    };

    let mut pieces = Vec::new();
    let mut first = true;
    for (i, slot) in ARMOR_POPULATION_ORDER.iter().enumerate() {
        if !first && rng.random::<f32>() < partial_chance {
            break;
        }
        first = false;
        pieces.push((
            slot.clone(),
            create_equipment_item(tier[i], difficulty),
            DEFAULT_EQUIPMENT_DROP_CHANCE,
        ));
    }
    pieces
}

/// Creates a fresh, full-durability `ItemStack` for mob equipment.
/// Vanilla mobs always spawn with equipment at full durability.
#[must_use]
fn create_equipment_item(item: &'static Item, _difficulty: &RegionalDifficulty) -> ItemStack {
    ItemStack::new(1, item)
}

/// Generates the equipment items, slots, and drop chances for a mob definition.
///
/// Handles the full weapon + armor selection logic, including enchantment
/// application when both `def.enchanted` is true and the difficulty-dependent
/// random check passes.
#[must_use]
fn equip_mob_from_def(
    def: &MobEquipmentDef,
    difficulty: &RegionalDifficulty,
) -> Vec<(EquipmentSlot, ItemStack, f32)> {
    let mut changes: Vec<(EquipmentSlot, ItemStack, f32)> = Vec::new();

    // ── Weapon ──
    match def.weapon {
        WeaponConfig::Always(item) => {
            let mut stack = create_equipment_item(item, difficulty);
            if def.enchanted && difficulty.should_happen(WEAPON_ENCHANT_CHANCE) {
                apply_vanilla_enchantments(
                    &mut stack,
                    &EquipmentSlot::MAIN_HAND,
                    difficulty.special_multiplier,
                );
            }
            changes.push((
                EquipmentSlot::MAIN_HAND,
                stack,
                DEFAULT_EQUIPMENT_DROP_CHANCE,
            ));
        }
        WeaponConfig::AlwaysWeighted(items) => {
            let item = weighted_select_item(items);
            let mut stack = create_equipment_item(item, difficulty);
            if def.enchanted && difficulty.should_happen(WEAPON_ENCHANT_CHANCE) {
                apply_vanilla_enchantments(
                    &mut stack,
                    &EquipmentSlot::MAIN_HAND,
                    difficulty.special_multiplier,
                );
            }
            changes.push((
                EquipmentSlot::MAIN_HAND,
                stack,
                DEFAULT_EQUIPMENT_DROP_CHANCE,
            ));
        }
        WeaponConfig::Chance {
            on_hard,
            otherwise,
            items,
        } => {
            let chance = if difficulty.base_difficulty == Difficulty::Hard {
                on_hard
            } else {
                otherwise
            };
            if rand::random::<f32>() < chance {
                let item = weighted_select_item(items);
                let mut stack = create_equipment_item(item, difficulty);
                if def.enchanted && difficulty.should_happen(WEAPON_ENCHANT_CHANCE) {
                    apply_vanilla_enchantments(
                        &mut stack,
                        &EquipmentSlot::MAIN_HAND,
                        difficulty.special_multiplier,
                    );
                }
                changes.push((
                    EquipmentSlot::MAIN_HAND,
                    stack,
                    DEFAULT_EQUIPMENT_DROP_CHANCE,
                ));
            }
        }
        WeaponConfig::None => {}
    }

    // ── Armor ──
    match def.armor {
        ArmorConfig::Vanilla => {
            if difficulty.should_happen(WEARING_ARMOR_CHANCE) {
                let armor_pieces = select_vanilla_armor(difficulty);
                for (slot, mut stack, drop_chance) in armor_pieces {
                    if def.enchanted && difficulty.should_happen(ARMOR_ENCHANT_CHANCE) {
                        apply_vanilla_enchantments(
                            &mut stack,
                            &slot,
                            difficulty.special_multiplier,
                        );
                    }
                    changes.push((slot, stack, drop_chance));
                }
            }
        }
        ArmorConfig::CustomPerSlot(entries) => {
            for entry in entries {
                if rand::random::<f32>() < entry.chance {
                    let mut stack = create_equipment_item(entry.item, difficulty);
                    if def.enchanted && difficulty.should_happen(ARMOR_ENCHANT_CHANCE) {
                        apply_vanilla_enchantments(
                            &mut stack,
                            entry.slot,
                            difficulty.special_multiplier,
                        );
                    }
                    changes.push((entry.slot.clone(), stack, DEFAULT_EQUIPMENT_DROP_CHANCE));
                }
            }
        }
        ArmorConfig::None => {}
    }

    changes
}

// ══════════════════════════════════════════════════════════════════
// Public entry point
// ══════════════════════════════════════════════════════════════════

/// Equips a mob with weapons/armor/enchantments when it spawns.
///
/// Called from the blanket `EntityBase::init_data_tracker` implementation for
/// all mob types. Looks up the mob's equipment definition in
/// [`EQUIPMENT_REGISTRY`], computes [`RegionalDifficulty`] at the mob's
/// position, generates equipment, stores it in the entity's equipment slots,
/// and broadcasts the changes to nearby players.
///
/// Mobs not listed in the registry silently receive no equipment.
pub async fn equip_mob_on_spawn(mob: &dyn EntityBase, world: &Arc<crate::world::World>) {
    let entity_type = mob.get_entity().entity_type;
    let pos = mob.get_entity().pos.load();
    let difficulty = RegionalDifficulty::at(world, pos);

    let Some(living) = mob.get_living_entity() else {
        return;
    };

    let entity_name = entity_type.resource_name;

    let Some(def) = EQUIPMENT_REGISTRY.get(entity_name) else {
        return;
    };

    let mut equipment = living.entity_equipment.lock().await;
    let mut drop_chances = living.equipment_drop_chances.lock().await;
    let changes_with_drops = equip_mob_from_def(def, &difficulty);

    let mut equipment_changes: Vec<(EquipmentSlot, ItemStack)> = Vec::new();

    for (slot, stack, drop_chance) in changes_with_drops {
        equipment.put(&slot, stack.clone()).await;
        drop_chances.insert(slot.clone(), drop_chance);
        equipment_changes.push((slot, stack));
    }

    drop(equipment);
    drop(drop_chances);

    living.send_equipment_changes(&equipment_changes);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zero_multiplier_difficulty() -> RegionalDifficulty {
        // Fresh Normal world: multiplier 0, so all `should_happen` rolls fail.
        RegionalDifficulty::calculate(Difficulty::Normal, 0, 0, 0.0)
    }

    #[test]
    fn armor_tier_table_covers_all_slots_for_every_material() {
        assert_eq!(ARMOR_TIERS.len(), 6);
        for tier in &*ARMOR_TIERS {
            assert_eq!(tier.len(), ARMOR_POPULATION_ORDER.len());
        }
        // Slot order must stay HEAD, CHEST, LEGS, FEET.
        assert!(ARMOR_POPULATION_ORDER[0] == EquipmentSlot::HEAD);
        assert!(ARMOR_POPULATION_ORDER[3] == EquipmentSlot::FEET);
    }

    #[test]
    fn single_entry_weapon_table_always_selects_it() {
        let table = [WeaponEntry {
            item: &Item::BOW,
            weight: 1.0,
        }];
        for _ in 0..8 {
            assert_eq!(weighted_select_item(&table).id, Item::BOW.id);
        }
    }

    #[test]
    fn always_weapon_yields_one_unenchanted_main_hand_change() {
        let def = MobEquipmentDef {
            entity_type: "test",
            weapon: WeaponConfig::Always(&Item::BOW),
            armor: ArmorConfig::None,
            enchanted: true,
            can_pick_up_loot: false,
        };
        let difficulty = zero_multiplier_difficulty();
        let changes = equip_mob_from_def(&def, &difficulty);
        assert_eq!(changes.len(), 1);
        let (slot, stack, drop_chance) = &changes[0];
        assert!(*slot == EquipmentSlot::MAIN_HAND);
        assert_eq!(stack.item.id, Item::BOW.id);
        assert!((drop_chance - DEFAULT_EQUIPMENT_DROP_CHANCE).abs() < f32::EPSILON);
        // With a zero special multiplier no enchantment roll can succeed.
        assert!(stack.patch.is_empty());
    }

    #[test]
    fn no_weapon_no_armor_yields_no_changes() {
        let def = MobEquipmentDef {
            entity_type: "test",
            weapon: WeaponConfig::None,
            armor: ArmorConfig::None,
            enchanted: true,
            can_pick_up_loot: false,
        };
        let difficulty = zero_multiplier_difficulty();
        assert!(equip_mob_from_def(&def, &difficulty).is_empty());
    }

    #[test]
    fn vanilla_armor_selection_respects_slot_order() {
        let difficulty = zero_multiplier_difficulty();
        for _ in 0..16 {
            let pieces = select_vanilla_armor(&difficulty);
            assert!(!pieces.is_empty());
            assert!(pieces.len() <= ARMOR_POPULATION_ORDER.len());
            for (index, (slot, stack, drop_chance)) in pieces.iter().enumerate() {
                assert!(*slot == ARMOR_POPULATION_ORDER[index]);
                assert_eq!(stack.item_count, 1);
                assert!((drop_chance - DEFAULT_EQUIPMENT_DROP_CHANCE).abs() < f32::EPSILON);
            }
        }
    }
}
