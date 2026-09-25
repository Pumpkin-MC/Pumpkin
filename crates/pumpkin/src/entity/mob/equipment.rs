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

use pumpkin_data::AttributeModifierSlot;
use pumpkin_data::Enchantment;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::data_component_impl::{
    AttributeModifiersImpl, CustomNameImpl, EnchantmentsImpl, EquipmentSlot, EquipmentType,
    EquippableImpl, IDSet, Operation,
};
use pumpkin_data::enchantment_provider::EnchantmentProvider;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag::{Tag, Taggable};
use pumpkin_util::difficulty::Difficulty;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use crate::entity::EntityBase;
use crate::entity::mob::{Mob, MobEntity};

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
    (phase - 4).abs() as f32 / 4.0
}

// ══════════════════════════════════════════════════════════════════
// Enchantment system — powered by data-driven EnchantmentProvider
// ══════════════════════════════════════════════════════════════════

/// Applies mob spawn equipment enchantments to a stack using the data-driven
/// `minecraft:mob_spawn_equipment` enchantment provider.
pub fn apply_vanilla_enchantments(
    stack: &mut ItemStack,
    _slot: &EquipmentSlot,
    special_multiplier: f32,
) {
    EnchantmentProvider::MOB_SPAWN_EQUIPMENT.apply_to_stack(stack, special_multiplier);
}

/// Applies pillager crossbow enchantments based on raid wave or default spawn provider.
pub fn apply_pillager_crossbow_enchantments(stack: &mut ItemStack, wave: Option<u32>) {
    match wave {
        Some(w) if w >= 5 => {
            EnchantmentProvider::RAID_PILLAGER_POST_WAVE_5.apply_to_stack(stack, 0.0);
        }
        Some(w) if w >= 3 => {
            EnchantmentProvider::RAID_PILLAGER_POST_WAVE_3.apply_to_stack(stack, 0.0);
        }
        _ => {
            EnchantmentProvider::PILLAGER_SPAWN_CROSSBOW.apply_to_stack(stack, 0.0);
        }
    }
}

/// Applies vindicator weapon enchantments based on raid wave.
pub fn apply_vindicator_weapon_enchantments(stack: &mut ItemStack, wave: Option<u32>) {
    match wave {
        Some(w) if w >= 5 => {
            EnchantmentProvider::RAID_VINDICATOR_POST_WAVE_5.apply_to_stack(stack, 0.0);
        }
        Some(_) => {
            EnchantmentProvider::RAID_VINDICATOR.apply_to_stack(stack, 0.0);
        }
        None => {}
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

#[must_use]
pub fn is_equippable_in_slot(
    entity_type: &EntityType,
    stack: &ItemStack,
    slot: &EquipmentSlot,
) -> bool {
    stack.get_data_component::<EquippableImpl>().map_or_else(
        || *slot == EquipmentSlot::MAIN_HAND,
        |equippable| {
            *equippable.slot == *slot
                && equippable
                    .allowed_entities
                    .as_ref()
                    .is_none_or(|allowed| match allowed {
                        IDSet::Tag(tag) => entity_type.is_tagged_with(tag).unwrap_or(false),
                        IDSet::IDs(ids) => ids.iter().any(|id| id.id == entity_type.id),
                    })
        },
    )
}

#[must_use]
pub fn get_equipment_slot_for_item(stack: &ItemStack) -> EquipmentSlot {
    stack
        .get_data_component::<EquippableImpl>()
        .map_or(EquipmentSlot::MAIN_HAND, |equippable| {
            equippable.slot.clone()
        })
}

#[must_use]
/// Whether `new_item` is an upgrade over `current_item` in `slot`.
pub fn can_replace_current_item(
    mob: &MobEntity,
    preferred_weapon_type: Option<&'static Tag>,
    new_item: &ItemStack,
    current_item: &ItemStack,
    slot: &EquipmentSlot,
) -> bool {
    if current_item.is_empty() {
        return true;
    }
    if slot.is_armor_slot() {
        compare_armor(mob, new_item, current_item, slot)
    } else {
        *slot == EquipmentSlot::MAIN_HAND
            && compare_weapons(mob, preferred_weapon_type, new_item, current_item, slot)
    }
}

fn compare_armor(
    mob: &MobEntity,
    new_item: &ItemStack,
    current_item: &ItemStack,
    slot: &EquipmentSlot,
) -> bool {
    if current_item.get_enchantment_level(&Enchantment::BINDING_CURSE) > 0 {
        return false;
    }
    let new_defense = approximate_attribute_with(mob, new_item, &Attributes::ARMOR, slot);
    let old_defense = approximate_attribute_with(mob, current_item, &Attributes::ARMOR, slot);
    let new_toughness =
        approximate_attribute_with(mob, new_item, &Attributes::ARMOR_TOUGHNESS, slot);
    let old_toughness =
        approximate_attribute_with(mob, current_item, &Attributes::ARMOR_TOUGHNESS, slot);
    if new_defense != old_defense {
        return new_defense > old_defense;
    }
    if new_toughness != old_toughness {
        return new_toughness > old_toughness;
    }
    can_replace_equal_item(new_item, current_item)
}

fn compare_weapons(
    mob: &MobEntity,
    preferred_weapon_type: Option<&'static Tag>,
    new_item: &ItemStack,
    current_item: &ItemStack,
    slot: &EquipmentSlot,
) -> bool {
    if let Some(preferred) = preferred_weapon_type {
        let current_preferred = current_item.item.has_tag(preferred);
        let new_preferred = new_item.item.has_tag(preferred);
        if current_preferred && !new_preferred {
            return false;
        }
        if !current_preferred && new_preferred {
            return true;
        }
    }
    let new_damage = approximate_attribute_with(mob, new_item, &Attributes::ATTACK_DAMAGE, slot);
    let old_damage =
        approximate_attribute_with(mob, current_item, &Attributes::ATTACK_DAMAGE, slot);
    if new_damage != old_damage {
        return new_damage > old_damage;
    }
    can_replace_equal_item(new_item, current_item)
}

fn approximate_attribute_with(
    mob: &MobEntity,
    stack: &ItemStack,
    attribute: &Attributes,
    slot: &EquipmentSlot,
) -> f64 {
    let base_value = mob.living_entity.get_attribute_base(attribute);
    let mut add_value = 0.0;
    let mut add_multiplied_base = 0.0;
    let mut multiplied_total = 1.0;
    if let Some(modifiers) = stack.get_data_component::<AttributeModifiersImpl>() {
        for modifier in modifiers.attribute_modifiers.iter() {
            if modifier.r#type.id != attribute.id || !attribute_slot_matches(&modifier.slot, slot) {
                continue;
            }
            match modifier.operation {
                Operation::AddValue => add_value += modifier.amount,
                Operation::AddMultipliedBase => add_multiplied_base += modifier.amount,
                Operation::AddMultipliedTotal => multiplied_total *= 1.0 + modifier.amount,
            }
        }
    }
    (base_value + add_value) * (1.0 + add_multiplied_base) * multiplied_total
}

fn attribute_slot_matches(group: &AttributeModifierSlot, slot: &EquipmentSlot) -> bool {
    match group {
        AttributeModifierSlot::Any => true,
        AttributeModifierSlot::MainHand => *slot == EquipmentSlot::MAIN_HAND,
        AttributeModifierSlot::OffHand => *slot == EquipmentSlot::OFF_HAND,
        AttributeModifierSlot::Hand => slot.slot_type() == EquipmentType::Hand,
        AttributeModifierSlot::Feet => *slot == EquipmentSlot::FEET,
        AttributeModifierSlot::Legs => *slot == EquipmentSlot::LEGS,
        AttributeModifierSlot::Chest => *slot == EquipmentSlot::CHEST,
        AttributeModifierSlot::Head => *slot == EquipmentSlot::HEAD,
        AttributeModifierSlot::Armor => slot.slot_type() == EquipmentType::HumanoidArmor,
        AttributeModifierSlot::Body => *slot == EquipmentSlot::BODY,
        AttributeModifierSlot::Saddle => *slot == EquipmentSlot::SADDLE,
    }
}

#[must_use]
pub fn can_replace_equal_item(new_item: &ItemStack, current_item: &ItemStack) -> bool {
    let enchantment_count = |stack: &ItemStack| {
        stack
            .get_data_component::<EnchantmentsImpl>()
            .map_or(0, |enchantments| enchantments.enchantment.len())
    };
    let new_enchantments = enchantment_count(new_item);
    let current_enchantments = enchantment_count(current_item);
    if new_enchantments != current_enchantments {
        return new_enchantments > current_enchantments;
    }
    let new_damage = new_item.get_damage();
    let current_damage = current_item.get_damage();
    if new_damage != current_damage {
        return new_damage < current_damage;
    }
    new_item.get_data_component::<CustomNameImpl>().is_some()
        && current_item
            .get_data_component::<CustomNameImpl>()
            .is_none()
}

#[must_use]
/// Equips `stack` if it beats what is worn, returning what got equipped.
pub fn equip_item_if_possible(mob: &dyn Mob, stack: ItemStack) -> ItemStack {
    let mob_entity = mob.get_mob_entity();
    let entity = &mob_entity.living_entity.entity;
    let mut slot = get_equipment_slot_for_item(&stack);
    if !is_equippable_in_slot(entity.entity_type, &stack, &slot) {
        return ItemStack::EMPTY.clone();
    }

    let item_in = |slot: &EquipmentSlot| {
        mob_entity
            .living_entity
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(slot)
    };
    let mut current = item_in(&slot);
    let mut can_replace = mob.can_replace_current_item(&stack, &current, &slot);
    if slot.is_armor_slot() && !can_replace {
        slot = EquipmentSlot::MAIN_HAND;
        current = item_in(&slot);
        can_replace = current.is_empty();
    }
    if !can_replace {
        return ItemStack::EMPTY.clone();
    }

    let drop_chance = mob_entity.drop_chance(&slot);
    if !current.is_empty() && (rand::random::<f32>() - 0.1).max(0.0) < drop_chance {
        mob_entity.spawn_at_location(current);
    }

    let mut stack = stack;
    let to_equip = limit_for_slot(&slot, &mut stack);
    mob_entity.set_item_slot_and_drop_when_killed(&slot, to_equip.clone());
    mob_entity
        .persistence_required
        .store(true, std::sync::atomic::Ordering::Relaxed);
    to_equip
}

// Hand slots have no count limit, every other slot holds one item.
fn limit_for_slot(slot: &EquipmentSlot, stack: &mut ItemStack) -> ItemStack {
    if slot.slot_type() == EquipmentType::Hand {
        std::mem::replace(stack, ItemStack::EMPTY.clone())
    } else {
        stack.split(1)
    }
}

#[cfg(test)]
mod tests {
    use super::moon_brightness;

    #[test]
    fn moon_brightness_matches_the_vanilla_phase_table() {
        let expected = [1.0, 0.75, 0.5, 0.25, 0.0, 0.25, 0.5, 0.75];
        for (phase, want) in expected.into_iter().enumerate() {
            let time_of_day = phase as i64 * 24000;
            assert!(
                (moon_brightness(time_of_day) - want).abs() < f32::EPSILON,
                "phase {phase}: got {}, want {want}",
                moon_brightness(time_of_day)
            );
        }
    }

    #[test]
    fn moon_brightness_wraps_every_eight_days() {
        for phase in 0..8i64 {
            assert!(
                (moon_brightness(phase * 24000) - moon_brightness((phase + 8) * 24000)).abs()
                    < f32::EPSILON,
                "phase {phase} does not wrap"
            );
        }
    }

    #[test]
    fn mob_spawn_equipment_enchantment_provider_applies_enchantments() {
        use pumpkin_data::data_component_impl::EquipmentSlot;
        use pumpkin_data::item::Item;
        use pumpkin_data::item_stack::ItemStack;

        let mut stack = ItemStack::new(1, &Item::DIAMOND_SWORD);
        super::apply_vanilla_enchantments(&mut stack, &EquipmentSlot::MAIN_HAND, 1.0);
        assert!(stack.has_enchantments());
    }

    #[test]
    fn pillager_crossbow_enchantments() {
        use pumpkin_data::Enchantment;
        use pumpkin_data::item::Item;
        use pumpkin_data::item_stack::ItemStack;

        let mut stack = ItemStack::new(1, &Item::CROSSBOW);
        super::apply_pillager_crossbow_enchantments(&mut stack, None);
        assert_eq!(stack.get_enchantment_level(&Enchantment::PIERCING), 1);

        let mut stack3 = ItemStack::new(1, &Item::CROSSBOW);
        super::apply_pillager_crossbow_enchantments(&mut stack3, Some(3));
        assert_eq!(stack3.get_enchantment_level(&Enchantment::QUICK_CHARGE), 1);

        let mut stack5 = ItemStack::new(1, &Item::CROSSBOW);
        super::apply_pillager_crossbow_enchantments(&mut stack5, Some(5));
        assert_eq!(stack5.get_enchantment_level(&Enchantment::QUICK_CHARGE), 2);
    }

    #[test]
    fn vindicator_weapon_enchantments() {
        use pumpkin_data::Enchantment;
        use pumpkin_data::item::Item;
        use pumpkin_data::item_stack::ItemStack;

        let mut stack = ItemStack::new(1, &Item::IRON_AXE);
        super::apply_vindicator_weapon_enchantments(&mut stack, Some(1));
        assert_eq!(stack.get_enchantment_level(&Enchantment::SHARPNESS), 1);

        let mut stack5 = ItemStack::new(1, &Item::IRON_AXE);
        super::apply_vindicator_weapon_enchantments(&mut stack5, Some(5));
        assert_eq!(stack5.get_enchantment_level(&Enchantment::SHARPNESS), 2);
    }
}
