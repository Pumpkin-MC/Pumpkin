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
//!
//! Split into cohesive submodules:
//! - `registry`: equipment table types and the `EQUIPMENT_REGISTRY`
//! - `difficulty`: vanilla `RegionalDifficulty` computation
//! - `enchant`: spawn enchantment pools, cost, and application
//! - `populate`: weapon/armor selection and the spawn entry point

mod difficulty;
mod enchant;
mod populate;
mod registry;

pub use difficulty::*;
pub use enchant::*;
pub use populate::*;
pub use registry::*;

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
