//! Mob spawn equipment system.
//!
//! This module handles the automatic equipping of mobs when they spawn, matching
//! vanilla Minecraft's `populateDefaultEquipmentSlots` and
//! `populateDefaultEquipmentEnchantments` behaviour. It features:
//!
//! - A data-driven `EQUIPMENT_REGISTRY` mapping 13 mob types to their weapon/armor
//!   configurations.
//! - Exact vanilla `RegionalDifficulty` computation (game time, chunk inhabited time,
//!   moon phase).
//! - Weighted enchantment selection with exclusive-set conflict resolution and
//!   cost-based level determination.
//! - Per-slot drop chances with looting bonus on death.
//!
//! Mobs not listed in the registry spawn with no equipment, matching vanilla
//! (not all mob types have equipment definitions).

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::LazyLock;

use pumpkin_data::Enchantment;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_util::difficulty::Difficulty;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use crate::entity::EntityBase;

// ══════════════════════════════════════════════════════════════════
// Global constants extracted from vanilla Minecraft 26.2
// Sources: Mob.java, DifficultyInstance.java, DropChances.java,
// EnchantmentsByCostWithDifficulty.java
// ══════════════════════════════════════════════════════════════════

/// Base chance (before `specialMultiplier` scaling) that a mob will wear armor.
/// From vanilla `Mob.MAX_WEARING_ARMOR_CHANCE`.
pub const WEARING_ARMOR_CHANCE: f32 = 0.15;

/// Chance per attempt to promote the armor tier to the next material.
/// From vanilla `Mob.WEARING_ARMOR_UPGRADE_MATERIAL_CHANCE`.
pub const ARMOR_UPGRADE_MATERIAL_CHANCE: f32 = 0.1087;

/// Maximum number of upgrade attempts for armor tier selection.
/// From vanilla `Mob.WEARING_ARMOR_UPGRADE_MATERIAL_ATTEMPTS`.
pub const ARMOR_UPGRADE_MATERIAL_ATTEMPTS: f32 = 3.0;

/// Default per-slot drop chance for equipment on mob death.
/// From vanilla `Mob.DEFAULT_EQUIPMENT_DROP_CHANCE`.
pub const DEFAULT_EQUIPMENT_DROP_CHANCE: f32 = 0.085;

/// Base chance (before `specialMultiplier`) for weapon enchantments at spawn.
/// From vanilla `Mob.MAX_ENCHANTED_WEAPON_CHANCE`.
pub const WEAPON_ENCHANT_CHANCE: f32 = 0.25;

/// Base chance (before `specialMultiplier`) for armor enchantments at spawn.
/// From vanilla `Mob.MAX_ENCHANTED_ARMOR_CHANCE`.
pub const ARMOR_ENCHANT_CHANCE: f32 = 0.5;

/// Minimum enchantment cost for mob spawn equipment.
/// From vanilla `mob_spawn_equipment.json`.
pub const MOB_SPAWN_ENCHANT_MIN_COST: i32 = 5;

/// Cost span added to the minimum, scaled by `specialMultiplier`.
/// From vanilla `mob_spawn_equipment.json`.
pub const MOB_SPAWN_ENCHANT_COST_SPAN: i32 = 17;

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
    /// See [`select_vanilla_armor`].
    Vanilla,
    /// Custom per-slot entries with independent chances (e.g. piglin golden armor).
    CustomPerSlot(&'static [ArmorSlotEntry]),
    /// No armor.
    None,
}

/// Equipment definition for a single mob type. All equipment is randomized at spawn
/// using [`RegionalDifficulty`] to compute per-world/per-chunk scaling factors.
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

// ══════════════════════════════════════════════════════════════════
// Regional Difficulty — exact vanilla DifficultyInstance.java
// Vanilla approximation: identical formula to vanilla Minecraft 26.2's
// DifficultyInstance, including clamped regional difficulty, special
// multiplier (0-1 linear), and effective difficulty (2-4 range).
// ══════════════════════════════════════════════════════════════════

/// Computed difficulty values for a specific world chunk.
///
/// Mirrors Vanilla's `DifficultyInstance`. Used to scale equipment spawn rates,
/// enchantment costs, and loot-pickup flags.
#[derive(Clone, Copy)]
pub struct RegionalDifficulty {
    /// The world's base difficulty level (`Easy`, `Normal`, `Hard`).
    pub base_difficulty: Difficulty,
    /// Effective difficulty computed from game time, inhabited time, and moon phase.
    /// Clamped to the range `[2.0, 4.0]` (or `0.0` for Peaceful).
    pub effective_difficulty: f32,
    /// Linear multiplier in `[0.0, 1.0]` derived from `effective_difficulty`.
    /// When `0.0` (fresh chunk + early game), no equipment, enchantments, or
    /// loot-pickup flags are applied.
    pub special_multiplier: f32,
}

impl RegionalDifficulty {
    /// Computes difficulty at the given world position.
    ///
    /// Looks up the chunk's inhabited time and combines it with the world's
    /// difficulty, game time, and moon phase.
    pub fn at(world: &Arc<crate::world::World>, pos: Vector3<f64>) -> Self {
        let level_info = world.level_info.load();
        let difficulty = level_info.difficulty;
        let time_of_day = world.level_time.try_lock().map_or(0, |t| t.time_of_day);
        let inhabited_time = {
            let chunk_x = (pos.x / 16.0).floor() as i32;
            let chunk_z = (pos.z / 16.0).floor() as i32;
            world
                .level
                .loaded_chunks
                .get(&pumpkin_util::math::vector2::Vector2::new(chunk_x, chunk_z))
                .map_or(0, |c| {
                    c.inhabited_time.load(std::sync::atomic::Ordering::Relaxed)
                })
        };
        let moon_brightness = moon_brightness(time_of_day);

        Self::calculate(difficulty, time_of_day, inhabited_time, moon_brightness)
    }

    /// Direct calculation from raw inputs. Used by `at()` and for testing.
    #[must_use]
    pub fn calculate(
        difficulty: Difficulty,
        total_game_time: i64,
        chunk_inhabited_time: u64,
        moon_brightness: f32,
    ) -> Self {
        if difficulty == Difficulty::Peaceful {
            return Self {
                base_difficulty: difficulty,
                effective_difficulty: 0.0,
                special_multiplier: 0.0,
            };
        }

        let is_hard = difficulty == Difficulty::Hard;

        let mut scale = 0.75f32;
        let global_scale = ((total_game_time as f32 - 72000.0) / 1440000.0).clamp(0.0, 1.0) * 0.25;
        scale += global_scale;

        let mut local_scale = 0.0f32;
        local_scale += (chunk_inhabited_time as f32 / 3600000.0).clamp(0.0, 1.0)
            * if is_hard { 1.0 } else { 0.75 };
        local_scale += (moon_brightness * 0.25).clamp(0.0, global_scale);

        if difficulty == Difficulty::Easy {
            local_scale *= 0.5;
        }

        let difficulty_id = match difficulty {
            Difficulty::Peaceful => 0,
            Difficulty::Easy => 1,
            Difficulty::Normal => 2,
            Difficulty::Hard => 3,
        };

        let effective = difficulty_id as f32 * (scale + local_scale);

        let special_multiplier = if effective < 2.0 {
            0.0
        } else if effective > 4.0 {
            1.0
        } else {
            (effective - 2.0) / 2.0
        };

        Self {
            base_difficulty: difficulty,
            effective_difficulty: effective,
            special_multiplier,
        }
    }

    /// Random check scaled by `special_multiplier`.
    ///
    /// Returns `true` with probability `base_chance * special_multiplier`. When
    /// `special_multiplier` is `0.0` this always returns `false` (matching vanilla
    /// behaviour on fresh Normal/Easy worlds).
    #[must_use]
    pub fn should_happen(&self, base_chance: f32) -> bool {
        rand::random::<f32>() < base_chance * self.special_multiplier
    }
}

/// Moon brightness factor for the given time of day (0.0 to 1.0).
/// Full moon at phase 0, new moon at phase 4.
#[must_use]
fn moon_brightness(time_of_day: i64) -> f32 {
    let phase = (time_of_day / 24000 % 8) as i32;
    if phase == 0 {
        1.0
    } else {
        1.0 - (phase - 4).abs() as f32 / 4.0
    }
}

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
pub fn apply_vanilla_enchantments(
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
        let Some(&fallback) = candidates.last() else {
            break;
        };
        let selected = selected.unwrap_or(fallback);

        let level = enchantment_level_from_cost(selected, cost);
        stack.add_enchantment(selected, level.clamp(1, selected.max_level) as u16);
        applied.push(selected);

        cost /= 2;
        if cost < 1 {
            break;
        }
    }
}

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
    items.last().map_or(&Item::AIR, |e| e.item)
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
            create_equipment_item(tier[i]),
            DEFAULT_EQUIPMENT_DROP_CHANCE,
        ));
    }
    pieces
}

/// Creates a fresh, full-durability `ItemStack` for mob equipment.
/// Vanilla mobs always spawn with equipment at full durability.
#[must_use]
fn create_equipment_item(item: &'static Item) -> ItemStack {
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
            let mut stack = create_equipment_item(item);
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
            let mut stack = create_equipment_item(item);
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
                let mut stack = create_equipment_item(item);
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
                    let mut stack = create_equipment_item(entry.item);
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
pub fn equip_mob_on_spawn(mob: &dyn EntityBase, world: &Arc<crate::world::World>) {
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

    let mut equipment = living
        .entity_equipment
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut drop_chances = living
        .equipment_drop_chances
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let changes_with_drops = equip_mob_from_def(def, &difficulty);

    let mut equipment_changes: Vec<(EquipmentSlot, ItemStack)> = Vec::new();

    for (slot, stack, drop_chance) in changes_with_drops {
        equipment.put(&slot, stack.clone());
        drop_chances.insert(slot.clone(), drop_chance);
        equipment_changes.push((slot, stack));
    }

    drop(equipment);
    drop(drop_chances);

    living.send_equipment_changes(&equipment_changes);
}
