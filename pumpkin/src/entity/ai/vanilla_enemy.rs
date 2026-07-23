//! Vanilla `Enemy` targeting helpers — synced to **Minecraft 26.2**
//! (`server-26.2.jar` / protocol 776).
//!
//! Iron golem 26.2 (`IronGolem.registerGoals`):
//! ```text
//! NearestAttackableTargetGoal(Mob.class, 5, false, false,
//!   (target, level) -> target instanceof Enemy && !(target instanceof Creeper))
//! ```
//!
//! Pumpkin approximates `Enemy` with [`MobCategory::MONSTER`]. That includes
//! zombies, skeletons, spiders, endermen, **wardens**, illagers, etc.
//! Iron golems are `MISC` and are not selected by this filter.
//!
//! Paper/Leaves do not change this predicate (only Bukkit target-reason hooks).

use pumpkin_data::entity::{EntityType, MobCategory};

/// Vanilla iron golem: only creepers are excluded from the Enemy target list.
pub const IRON_GOLEM_ENEMY_EXCLUDES: &[&EntityType] = &[&EntityType::CREEPER];

/// Vanilla snow golem: all Enemy types (including creepers).
pub const SNOW_GOLEM_ENEMY_EXCLUDES: &[&EntityType] = &[];

/// Spawn category used as the `Enemy` stand-in.
pub const ENEMY_CATEGORY: &MobCategory = &MobCategory::MONSTER;

/// Vanilla `NearestAttackableTargetGoal` reciprocal chance for iron golems.
pub const IRON_GOLEM_TARGET_CHANCE: i32 = 5;

/// Vanilla snow golem target chance.
pub const SNOW_GOLEM_TARGET_CHANCE: i32 = 10;
