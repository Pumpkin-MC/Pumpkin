//! Vanilla `Enemy` targeting helpers.
//!
//! In Java, iron golems / snow golems use:
//! ```text
//! NearestAttackableTargetGoal(Mob.class, …,
//!   living -> living instanceof Enemy && !(living instanceof Creeper)  // iron golem
//!   living -> living instanceof Enemy                                    // snow golem
//! )
//! ```
//!
//! Pumpkin approximates `Enemy` with [`MobCategory::MONSTER`] (all hostile spawn
//! categories). That includes zombies, skeletons, spiders, endermen, **wardens**,
//! illagers, etc. Iron golems are `MISC` and are not selected by this filter.
//!
//! Reference: Yarn / Mojmap `IronGolem` / `SnowGolem` constructors,
//! <https://minecraft.wiki/w/Iron_Golem#Behavior>

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
