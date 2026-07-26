use std::collections::HashMap;
use std::sync::LazyLock;

use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::item::Item;

// ══════════════════════════════════════════════════════════════════
// Equipment Table Registry
// ══════════════════════════════════════════════════════════════════

/// A weighted entry in a weapon selection table.
/// Matches vanilla's weighted random selection in mob `populateDefaultEquipmentSlots`.
#[derive(Clone, Copy)]
pub struct WeaponEntry {
    /// The item to potentially give.
    pub item: &'static Item,
    /// Relative weight in the selection pool.
    pub weight: f32,
}

/// How a mob's main-hand weapon is selected on spawn.
#[derive(Clone, Copy)]
pub enum WeaponConfig {
    /// Always give this exact item (e.g. skeleton → bow).
    Always(&'static Item),
    /// Always give one of the weighted items (e.g. piglin weapons).
    AlwaysWeighted(&'static [WeaponEntry]),
    /// Give a weighted weapon with a difficulty-dependent chance.
    Chance {
        /// Chance when the base difficulty is Hard.
        on_hard: f32,
        /// Chance on all other difficulties.
        otherwise: f32,
        /// Weighted item pool to select from.
        items: &'static [WeaponEntry],
    },
    /// No weapon.
    None,
}

/// A per-slot armor entry with an independent spawn chance.
pub struct ArmorSlotEntry {
    /// Which equipment slot this armor occupies.
    pub slot: &'static EquipmentSlot,
    /// The armor item.
    pub item: &'static Item,
    /// Independent chance this slot receives armor.
    pub chance: f32,
}

/// How a mob's armor is selected on spawn.
#[derive(Clone, Copy)]
pub enum ArmorConfig {
    /// Use the vanilla algorithm: random tier (0-2 base + 3 upgrade attempts at
    /// 10.87% each), partial armor break chance (10% on Hard, 25% otherwise).
    /// See [`select_vanilla_armor`](super::populate).
    Vanilla,
    /// Custom per-slot entries with independent chances (e.g. piglin golden armor).
    CustomPerSlot(&'static [ArmorSlotEntry]),
    /// No armor.
    None,
}

/// Equipment definition for a single mob type. All equipment is randomized at spawn
/// using [`RegionalDifficulty`](super::RegionalDifficulty) to compute per-world/per-chunk scaling factors.
pub struct MobEquipmentDef {
    /// The entity resource name (e.g. `"zombie"`, `"skeleton"`).
    pub entity_type: &'static str,
    /// Main-hand weapon configuration.
    pub weapon: WeaponConfig,
    /// Armor configuration.
    pub armor: ArmorConfig,
    /// Whether spawn-time enchantments can be applied.
    pub enchanted: bool,
    /// Whether this mob can randomly pick up loot from the ground.
    pub can_pick_up_loot: bool,
}

/// Registry of all mobs that receive equipment at spawn.
///
/// Maps entity resource names to their equipment definitions. Only mobs listed
/// here will receive weapons, armor, enchantments, and drop-chance settings.
/// Unlisted mobs spawn with no equipment (matching vanilla — not all mobs have
/// equipment tables).
pub static EQUIPMENT_REGISTRY: LazyLock<HashMap<&'static str, MobEquipmentDef>> =
    LazyLock::new(|| {
        static ZOMBIE_WEAPONS: [WeaponEntry; 3] = [
            WeaponEntry {
                item: &Item::IRON_SWORD,
                weight: 1.0,
            },
            WeaponEntry {
                item: &Item::IRON_SPEAR,
                weight: 1.0,
            },
            WeaponEntry {
                item: &Item::IRON_SHOVEL,
                weight: 4.0,
            },
        ];

        static DROWNED_WEAPONS: [WeaponEntry; 2] = [
            WeaponEntry {
                item: &Item::TRIDENT,
                weight: 10.0,
            },
            WeaponEntry {
                item: &Item::FISHING_ROD,
                weight: 6.0,
            },
        ];

        static PIGLIN_WEAPONS: [WeaponEntry; 3] = [
            WeaponEntry {
                item: &Item::CROSSBOW,
                weight: 5.0,
            },
            WeaponEntry {
                item: &Item::GOLDEN_SWORD,
                weight: 4.5,
            },
            WeaponEntry {
                item: &Item::GOLDEN_SPEAR,
                weight: 0.5,
            },
        ];

        static PIGLIN_ARMOR: [ArmorSlotEntry; 4] = [
            ArmorSlotEntry {
                slot: &EquipmentSlot::HEAD,
                item: &Item::GOLDEN_HELMET,
                chance: 0.1,
            },
            ArmorSlotEntry {
                slot: &EquipmentSlot::CHEST,
                item: &Item::GOLDEN_CHESTPLATE,
                chance: 0.1,
            },
            ArmorSlotEntry {
                slot: &EquipmentSlot::LEGS,
                item: &Item::GOLDEN_LEGGINGS,
                chance: 0.1,
            },
            ArmorSlotEntry {
                slot: &EquipmentSlot::FEET,
                item: &Item::GOLDEN_BOOTS,
                chance: 0.1,
            },
        ];

        static ZOMBIFIED_PIGLIN_WEAPONS: [WeaponEntry; 2] = [
            WeaponEntry {
                item: &Item::GOLDEN_SWORD,
                weight: 19.0,
            },
            WeaponEntry {
                item: &Item::GOLDEN_SPEAR,
                weight: 1.0,
            },
        ];

        let mut m = HashMap::new();

        // ─── Zombie ───
        m.insert(
            "zombie",
            MobEquipmentDef {
                entity_type: "zombie",
                weapon: WeaponConfig::Chance {
                    on_hard: 0.05,
                    otherwise: 0.01,
                    items: &ZOMBIE_WEAPONS,
                },
                armor: ArmorConfig::Vanilla,
                enchanted: true,
                can_pick_up_loot: true,
            },
        );

        // ─── Husk ───
        m.insert(
            "husk",
            MobEquipmentDef {
                entity_type: "husk",
                weapon: WeaponConfig::Chance {
                    on_hard: 0.05,
                    otherwise: 0.01,
                    items: &ZOMBIE_WEAPONS,
                },
                armor: ArmorConfig::Vanilla,
                enchanted: true,
                can_pick_up_loot: true,
            },
        );

        // ─── Zombie Villager ───
        m.insert(
            "zombie_villager",
            MobEquipmentDef {
                entity_type: "zombie_villager",
                weapon: WeaponConfig::Chance {
                    on_hard: 0.05,
                    otherwise: 0.01,
                    items: &ZOMBIE_WEAPONS,
                },
                armor: ArmorConfig::Vanilla,
                enchanted: true,
                can_pick_up_loot: true,
            },
        );

        // ─── Drowned ───
        m.insert(
            "drowned",
            MobEquipmentDef {
                entity_type: "drowned",
                weapon: WeaponConfig::Chance {
                    on_hard: 0.10,
                    otherwise: 0.10,
                    items: &DROWNED_WEAPONS,
                },
                armor: ArmorConfig::None,
                enchanted: true,
                can_pick_up_loot: true,
            },
        );

        // ─── Zombified Piglin ───
        m.insert(
            "zombified_piglin",
            MobEquipmentDef {
                entity_type: "zombified_piglin",
                weapon: WeaponConfig::AlwaysWeighted(&ZOMBIFIED_PIGLIN_WEAPONS),
                armor: ArmorConfig::None,
                enchanted: true,
                can_pick_up_loot: false,
            },
        );

        // ─── Skeleton ───
        m.insert(
            "skeleton",
            MobEquipmentDef {
                entity_type: "skeleton",
                weapon: WeaponConfig::Always(&Item::BOW),
                armor: ArmorConfig::Vanilla,
                enchanted: true,
                can_pick_up_loot: true,
            },
        );

        // ─── Stray ───
        m.insert(
            "stray",
            MobEquipmentDef {
                entity_type: "stray",
                weapon: WeaponConfig::Always(&Item::BOW),
                armor: ArmorConfig::Vanilla,
                enchanted: true,
                can_pick_up_loot: true,
            },
        );

        // ─── Bogged ───
        m.insert(
            "bogged",
            MobEquipmentDef {
                entity_type: "bogged",
                weapon: WeaponConfig::Always(&Item::BOW),
                armor: ArmorConfig::Vanilla,
                enchanted: true,
                can_pick_up_loot: true,
            },
        );

        // ─── Wither Skeleton ───
        m.insert(
            "wither_skeleton",
            MobEquipmentDef {
                entity_type: "wither_skeleton",
                weapon: WeaponConfig::Always(&Item::STONE_SWORD),
                armor: ArmorConfig::None,
                enchanted: false,
                can_pick_up_loot: false,
            },
        );

        // ─── Piglin ───
        m.insert(
            "piglin",
            MobEquipmentDef {
                entity_type: "piglin",
                weapon: WeaponConfig::AlwaysWeighted(&PIGLIN_WEAPONS),
                armor: ArmorConfig::CustomPerSlot(&PIGLIN_ARMOR),
                enchanted: true,
                can_pick_up_loot: false,
            },
        );

        // ─── Pillager ───
        m.insert(
            "pillager",
            MobEquipmentDef {
                entity_type: "pillager",
                weapon: WeaponConfig::Always(&Item::CROSSBOW),
                armor: ArmorConfig::None,
                enchanted: false,
                can_pick_up_loot: false,
            },
        );

        // ─── Vindicator ───
        m.insert(
            "vindicator",
            MobEquipmentDef {
                entity_type: "vindicator",
                weapon: WeaponConfig::Always(&Item::IRON_AXE),
                armor: ArmorConfig::None,
                enchanted: true,
                can_pick_up_loot: false,
            },
        );

        m
    });

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_contains_all_expected_mob_types() {
        let expected = [
            "zombie",
            "husk",
            "zombie_villager",
            "drowned",
            "zombified_piglin",
            "skeleton",
            "stray",
            "bogged",
            "wither_skeleton",
            "piglin",
            "pillager",
            "vindicator",
        ];
        for name in expected {
            assert!(EQUIPMENT_REGISTRY.contains_key(name), "missing {name}");
        }
        assert_eq!(EQUIPMENT_REGISTRY.len(), expected.len());
    }

    #[test]
    fn registry_entries_carry_their_own_entity_type() {
        for (name, def) in &*EQUIPMENT_REGISTRY {
            assert_eq!(*name, def.entity_type);
        }
    }

    #[test]
    fn skeleton_family_always_gets_a_bow() {
        for name in ["skeleton", "stray", "bogged"] {
            let def = EQUIPMENT_REGISTRY.get(name).unwrap();
            assert!(matches!(def.weapon, WeaponConfig::Always(item) if item.id == Item::BOW.id));
            assert!(def.enchanted);
            assert!(def.can_pick_up_loot);
        }
    }

    #[test]
    fn zombie_weapon_chance_matches_vanilla_difficulty_split() {
        let def = EQUIPMENT_REGISTRY.get("zombie").unwrap();
        match def.weapon {
            WeaponConfig::Chance {
                on_hard,
                otherwise,
                items,
            } => {
                assert!((on_hard - 0.05).abs() < f32::EPSILON);
                assert!((otherwise - 0.01).abs() < f32::EPSILON);
                assert_eq!(items.len(), 3);
            }
            _ => panic!("zombie weapon must be a difficulty-gated chance"),
        }
    }

    #[test]
    fn wither_skeleton_is_never_enchanted() {
        let def = EQUIPMENT_REGISTRY.get("wither_skeleton").unwrap();
        assert!(!def.enchanted);
        assert!(matches!(def.armor, ArmorConfig::None));
        assert!(
            matches!(def.weapon, WeaponConfig::Always(item) if item.id == Item::STONE_SWORD.id)
        );
    }
}
