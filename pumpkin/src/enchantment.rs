//! Generic enchantment-effect framework.
//!
//! `pumpkin_data::Enchantment` (generated from vanilla's enchantment registry) carries no
//! numeric effect parameters at all — anvil cost, exclusive sets, slots, but nothing that
//! says "how much". Vanilla itself is data-driven here: every enchantment JSON attaches a list
//! of `EnchantmentEffectComponents` (see
//! `net/minecraft/world/item/enchantment/EnchantmentEffectComponents.java` in the decompiled
//! 26.2 source), each holding one or more `LevelBasedValue` formulas
//! (`net/minecraft/world/item/enchantment/LevelBasedValue.java`).
//!
//! This module ports that shape in hand-written Rust: [`LevelBasedValue`] mirrors vanilla's
//! formula variants exactly (`constant`/`linear`/`lookup`+fallback/`fraction`/`levels_squared`/
//! `clamped`), and [`EnchantmentEffect`] mirrors the effect-component kinds Pumpkin has (or is
//! expected to grow) call sites for. [`effects_for`] is the single dispatch path: look up an
//! enchantment's effects by `registry_key` and get back typed, level-parameterized data instead
//! of a per-name `if enchantment == Enchantment::X` chain at every call site.
//!
//! All numeric constants below are transcribed directly from
//! `/tmp/pumpkin-vanilla-26.2/decompiled/data/minecraft/enchantment/*.json` and cross-checked
//! against unit tests in this file. Where a JSON effect component has no Pumpkin evaluator yet
//! (e.g. `location_changed`'s `replace_disk`/attribute-on-equip machinery, or a
//! `ConditionalEffect` predicate type Pumpkin doesn't model), the enchantment's *values* are
//! still captured here for completeness and testing, but callers don't exist yet — see the
//! module-level doc on [`EnchantmentEffect`] for exactly which variants are wired vs. deferred.

use pumpkin_data::Enchantment;
use pumpkin_data::entity::EntityType;
use pumpkin_data::tag::{EntityType as EntityTypeTag, Taggable};

/// Mirrors vanilla `LevelBasedValue`.
///
/// See `net/minecraft/world/item/enchantment/LevelBasedValue.java`; `calculate` mirrors the Java
/// method of the same name exactly, including the float-only arithmetic vanilla uses throughout
/// this type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LevelBasedValue {
    Constant(f32),
    Linear {
        base: f32,
        per_level_above_first: f32,
    },
    LevelsSquared {
        added: f32,
    },
    /// 1-indexed: `values[level - 1]` for `level <= values.len()`, else `fallback`.
    Lookup {
        values: &'static [f32],
        fallback: &'static Self,
    },
    Fraction {
        numerator: &'static Self,
        denominator: &'static Self,
    },
    Clamped {
        value: &'static Self,
        min: f32,
        max: f32,
    },
}

impl LevelBasedValue {
    #[must_use]
    pub const fn linear(base: f32, per_level_above_first: f32) -> Self {
        Self::Linear {
            base,
            per_level_above_first,
        }
    }

    /// Mirrors `LevelBasedValue.calculate(int level)` for every variant.
    #[must_use]
    pub fn calculate(&self, level: i32) -> f32 {
        match self {
            Self::Constant(v) => *v,
            Self::Linear {
                base,
                per_level_above_first,
            } => base + per_level_above_first * (level - 1) as f32,
            Self::LevelsSquared { added } => (level * level) as f32 + added,
            Self::Lookup { values, fallback } => {
                if level >= 1 && (level as usize) <= values.len() {
                    values[level as usize - 1]
                } else {
                    fallback.calculate(level)
                }
            }
            Self::Fraction {
                numerator,
                denominator,
            } => {
                let d = denominator.calculate(level);
                if d == 0.0 {
                    0.0
                } else {
                    numerator.calculate(level) / d
                }
            }
            Self::Clamped { value, min, max } => value.calculate(level).clamp(*min, *max),
        }
    }
}

/// Gates a `minecraft:damage` effect the way vanilla's `entity_properties`/
/// `sensitive_to_*` entity-type-tag requirements do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageCondition {
    Always,
    SensitiveToBaneOfArthropods,
    SensitiveToSmite,
    SensitiveToImpaling,
    /// Power's `damage` requirement gates on the *direct attacker* (the arrow projectile) being
    /// tagged `#minecraft:arrows` — a different entity than the victim every other
    /// [`DamageCondition`] variant checks. [`Self::applies`] only ever receives the
    /// victim/target entity type (see its melee call sites in player.rs/mob/mod.rs), so it
    /// cannot evaluate this condition and always returns `false` here; a projectile-attack call
    /// site would need a separate check against the arrow's own entity type. Power's damage
    /// bonus is applied today via a pre-existing, unrelated multiplicative formula in
    /// `item/items/bow.rs`, not through this framework.
    DirectAttackerIsArrow,
}

impl DamageCondition {
    #[must_use]
    pub fn applies(&self, target_type: &'static EntityType) -> bool {
        match self {
            Self::Always => true,
            Self::SensitiveToBaneOfArthropods => {
                target_type.has_tag(&EntityTypeTag::MINECRAFT_SENSITIVE_TO_BANE_OF_ARTHROPODS)
            }
            Self::SensitiveToSmite => {
                target_type.has_tag(&EntityTypeTag::MINECRAFT_SENSITIVE_TO_SMITE)
            }
            Self::SensitiveToImpaling => {
                target_type.has_tag(&EntityTypeTag::MINECRAFT_SENSITIVE_TO_IMPALING)
            }
            Self::DirectAttackerIsArrow => false,
        }
    }
}

/// Gates a `minecraft:damage_protection` effect by damage-source tag.
///
/// Mirrors the `damage_source_properties`/`tags` requirements in `protection.json`/
/// `fire_protection.json`/`blast_protection.json`/`projectile_protection.json`/
/// `feather_falling.json`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtectionCondition {
    /// Protection: any damage not tagged `bypasses_invulnerability` (and not drown/starve/
    /// `generic_kill` per Pumpkin's existing `living.rs` gate).
    Always,
    IsFire,
    IsExplosion,
    IsProjectile,
    IsFall,
}

/// Gates a `minecraft:knockback` effect (Knockback enchantment applies unconditionally, Punch
/// only when the direct attacker is tagged `#minecraft:arrows`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnockbackCondition {
    Always,
    AttackerIsArrow,
}

/// One typed enchantment-effect-component, parameterized by [`LevelBasedValue`] where vanilla's
/// component holds a formula. Variants are grouped by wiring status:
///
/// **Wired** into a Pumpkin call site via this module: `Damage` (player.rs melee, mob/mod.rs
/// melee, trident.rs impaling — Power's `DirectAttackerIsArrow` condition always evaluates to
/// `false` at these melee sites by design, since Power's actual bonus stays on bow.rs's existing
/// formula; see [`DamageCondition::DirectAttackerIsArrow`]), `DamageProtection`
/// (living.rs armor-enchantment pass, also
/// fixing a pre-existing Feather Falling constant bug — see
/// `feather_falling_is_three_per_level_not_four`), `Knockback` (player.rs, mob/mod.rs),
/// `ArmorEffectiveness` (combat.rs `breach_armor_fraction`), `SmashDamagePerFallenBlock`
/// (combat.rs `density_extra_damage`), `PostAttackKnockbackMultiplier` (combat.rs
/// `wind_burst_knockback_multiplier`), `IgniteOnHit` (player.rs, mob/mod.rs fire aspect),
/// `CrossbowChargeTime` (crossbow.rs quick charge).
///
/// **Modeled, not yet wired** (values verified against vanilla JSON in this file's tests, but no
/// call site consumes them yet — see the final report / PARITY.md for the exact blocker per
/// enchantment): `ProjectileCount`, `ProjectileSpread` (multishot's existing call site already
/// hardcodes the same constants correctly since `max_level` is 1 — no behavior to gain from
/// wiring), `ProjectilePiercing`, `TridentReturnAcceleration` (trident has no return-to-owner
/// flight logic at all yet — a missing *feature*, not a wiring gap), `TridentSpinAttackStrength`,
/// `FishingTimeReduction`/`FishingLuckBonus` (existing call sites use different unit
/// conventions not yet verified against `FishingHook.java`), `RepairWithXpMultiply`,
/// `ThornsChance` (no reflect-damage-on-hit code path exists to hook into),
/// `EquipmentDropsChance` (looting's mob-drop-chance boost has no call site yet),
/// `AttributeBonus` (vanilla applies these via an equip-time attribute-modifier system Pumpkin
/// doesn't have a generic equivalent for yet), `AmmoUseSetZero`, `BlockExperienceSetZero`,
/// `ItemDamageRemoveBinomial`, `PreventEquipmentDrop`, `PreventArmorChange` (curses currently
/// have no enforcement code at all).
#[derive(Debug, Clone, Copy)]
pub enum EnchantmentEffect {
    Damage(DamageCondition, LevelBasedValue),
    DamageProtection(ProtectionCondition, LevelBasedValue),
    Knockback(KnockbackCondition, LevelBasedValue),
    ArmorEffectiveness(LevelBasedValue),
    SmashDamagePerFallenBlock(LevelBasedValue),
    /// Wind Burst's `post_attack` explode knockback multiplier.
    PostAttackKnockbackMultiplier(LevelBasedValue),
    /// Fire Aspect's `post_attack` ignite duration, in ticks (JSON is in seconds; ×20 applied
    /// at `calculate()` call sites the same way vanilla's `EnchantmentEntityEffects.Ignite`
    /// converts `duration` seconds to ticks).
    IgniteOnHit(LevelBasedValue),

    // --- modeled, not yet wired (see enum doc) ---
    ProjectileCount(LevelBasedValue),
    ProjectileSpread(LevelBasedValue),
    ProjectilePiercing(LevelBasedValue),
    TridentReturnAcceleration(LevelBasedValue),
    TridentSpinAttackStrength(LevelBasedValue),
    FishingTimeReduction(LevelBasedValue),
    FishingLuckBonus(LevelBasedValue),
    CrossbowChargeTime(LevelBasedValue),
    RepairWithXpMultiply(f32),
    /// Looting's `equipment_drops` chance-per-level bonus (gated on the attacker being a
    /// player), separate from `AttributeBonus` since vanilla's `equipment_drops` is a
    /// `TargetedConditionalEffect<EnchantmentValueEffect>` component, not an equip-time
    /// attribute modifier.
    EquipmentDropsChance(LevelBasedValue),
    /// Thorns: `post_attack` random-chance reflect. `chance = value.calculate(level)`, damage
    /// range and item-damage cost are vanilla constants (1..=5 damage, 2 durability) not
    /// level-scaled, so they're not modeled as a `LevelBasedValue`.
    ThornsChance(LevelBasedValue),
    /// Generic per-enchantment attribute-modifier bonus (vanilla's `minecraft:attributes`
    /// component, applied on equip rather than per-hit). The `&str` names the vanilla attribute
    /// id for documentation/test purposes only.
    AttributeBonus(&'static str, LevelBasedValue),
    AmmoUseSetZero,
    BlockExperienceSetZero,
    ItemDamageRemoveBinomial(LevelBasedValue),
    PreventEquipmentDrop,
    PreventArmorChange,
}

const BANE_DAMAGE: LevelBasedValue = LevelBasedValue::linear(2.5, 2.5);
const SMITE_DAMAGE: LevelBasedValue = LevelBasedValue::linear(2.5, 2.5);
const IMPALING_DAMAGE: LevelBasedValue = LevelBasedValue::linear(2.5, 2.5);
const SHARPNESS_DAMAGE: LevelBasedValue = LevelBasedValue::linear(1.0, 0.5);
const POWER_DAMAGE: LevelBasedValue = LevelBasedValue::linear(1.0, 0.5);

const PROTECTION_VALUE: LevelBasedValue = LevelBasedValue::linear(1.0, 1.0);
const FIRE_PROTECTION_VALUE: LevelBasedValue = LevelBasedValue::linear(2.0, 2.0);
const BLAST_PROTECTION_VALUE: LevelBasedValue = LevelBasedValue::linear(2.0, 2.0);
const PROJECTILE_PROTECTION_VALUE: LevelBasedValue = LevelBasedValue::linear(2.0, 2.0);
const FEATHER_FALLING_VALUE: LevelBasedValue = LevelBasedValue::linear(3.0, 3.0);

const KNOCKBACK_VALUE: LevelBasedValue = LevelBasedValue::linear(1.0, 1.0);
const PUNCH_VALUE: LevelBasedValue = LevelBasedValue::linear(1.0, 1.0);

const DENSITY_VALUE: LevelBasedValue = LevelBasedValue::linear(0.5, 0.5);
const BREACH_VALUE: LevelBasedValue = LevelBasedValue::linear(-0.15, -0.15);
const WIND_BURST_FALLBACK: LevelBasedValue = LevelBasedValue::linear(1.5, 0.35);
const WIND_BURST_VALUES: &[f32] = &[1.2, 1.75, 2.2];
const WIND_BURST_VALUE: LevelBasedValue = LevelBasedValue::Lookup {
    values: WIND_BURST_VALUES,
    fallback: &WIND_BURST_FALLBACK,
};

const FIRE_ASPECT_DURATION_SECONDS: LevelBasedValue = LevelBasedValue::linear(4.0, 4.0);

const LOOTING_VALUE: LevelBasedValue = LevelBasedValue::linear(0.01, 0.01);
const MULTISHOT_COUNT: LevelBasedValue = LevelBasedValue::Constant(2.0);
const MULTISHOT_SPREAD: LevelBasedValue = LevelBasedValue::Constant(10.0);
const PIERCING_VALUE: LevelBasedValue = LevelBasedValue::linear(1.0, 1.0);
const LOYALTY_VALUE: LevelBasedValue = LevelBasedValue::linear(1.0, 1.0);
const LUCK_OF_THE_SEA_VALUE: LevelBasedValue = LevelBasedValue::linear(1.0, 1.0);
const LURE_VALUE: LevelBasedValue = LevelBasedValue::linear(5.0, 5.0);
const QUICK_CHARGE_VALUE: LevelBasedValue = LevelBasedValue::linear(-0.25, -0.25);
const RIPTIDE_SPIN_VALUE: LevelBasedValue = LevelBasedValue::linear(1.5, 0.75);
const MENDING_FACTOR: f32 = 2.0;
const THORNS_CHANCE: LevelBasedValue = LevelBasedValue::linear(0.15, 0.15);

const AQUA_AFFINITY_ATTR: LevelBasedValue = LevelBasedValue::linear(4.0, 4.0);
const DEPTH_STRIDER_ATTR: LevelBasedValue = LevelBasedValue::linear(0.333_333_34, 0.333_333_34);
const EFFICIENCY_ATTR: LevelBasedValue = LevelBasedValue::LevelsSquared { added: 1.0 };
const RESPIRATION_ATTR: LevelBasedValue = LevelBasedValue::linear(1.0, 1.0);
const SWIFT_SNEAK_ATTR: LevelBasedValue = LevelBasedValue::linear(0.15, 0.15);
const BLAST_PROTECTION_ATTR: LevelBasedValue = LevelBasedValue::linear(0.15, 0.15);
const SWEEPING_EDGE_NUMERATOR: LevelBasedValue = LevelBasedValue::linear(1.0, 1.0);
const SWEEPING_EDGE_DENOMINATOR: LevelBasedValue = LevelBasedValue::linear(2.0, 1.0);
const SWEEPING_EDGE_RATIO: LevelBasedValue = LevelBasedValue::Fraction {
    numerator: &SWEEPING_EDGE_NUMERATOR,
    denominator: &SWEEPING_EDGE_DENOMINATOR,
};
const SOUL_SPEED_SPEED_ATTR: LevelBasedValue = LevelBasedValue::linear(0.0405, 0.0105);

const UNBREAKING_ARMOR_NUMERATOR: LevelBasedValue = LevelBasedValue::linear(2.0, 2.0);
const UNBREAKING_ARMOR_DENOMINATOR: LevelBasedValue = LevelBasedValue::linear(10.0, 5.0);
const UNBREAKING_ARMOR_CHANCE: LevelBasedValue = LevelBasedValue::Fraction {
    numerator: &UNBREAKING_ARMOR_NUMERATOR,
    denominator: &UNBREAKING_ARMOR_DENOMINATOR,
};
const UNBREAKING_OTHER_NUMERATOR: LevelBasedValue = LevelBasedValue::linear(1.0, 1.0);
const UNBREAKING_OTHER_DENOMINATOR: LevelBasedValue = LevelBasedValue::linear(2.0, 1.0);
const UNBREAKING_OTHER_CHANCE: LevelBasedValue = LevelBasedValue::Fraction {
    numerator: &UNBREAKING_OTHER_NUMERATOR,
    denominator: &UNBREAKING_OTHER_DENOMINATOR,
};

/// Single dispatch path: look up an enchantment's effects by `registry_key`.
///
/// Replaces per-call-site `if enchantment == Enchantment::X` chains with one match keyed on the
/// stable registry key (not `id`, which is a generated ordinal that shifts as Mojang adds
/// enchantments).
#[must_use]
pub fn effects_for(enchantment: &'static Enchantment) -> &'static [EnchantmentEffect] {
    use EnchantmentEffect as E;
    match enchantment.registry_key {
        "aqua_affinity" => &[E::AttributeBonus(
            "submerged_mining_speed",
            AQUA_AFFINITY_ATTR,
        )],
        "bane_of_arthropods" => &[E::Damage(
            DamageCondition::SensitiveToBaneOfArthropods,
            BANE_DAMAGE,
        )],
        "binding_curse" => &[E::PreventArmorChange],
        "blast_protection" => &[
            E::DamageProtection(ProtectionCondition::IsExplosion, BLAST_PROTECTION_VALUE),
            E::AttributeBonus("explosion_knockback_resistance", BLAST_PROTECTION_ATTR),
        ],
        "breach" => &[E::ArmorEffectiveness(BREACH_VALUE)],
        // "channeling" has no numeric formula (boolean-gated lightning summon only) — falls
        // through to the wildcard arm below.
        "density" => &[E::SmashDamagePerFallenBlock(DENSITY_VALUE)],
        "depth_strider" => &[E::AttributeBonus(
            "water_movement_efficiency",
            DEPTH_STRIDER_ATTR,
        )],
        "efficiency" => &[E::AttributeBonus("mining_efficiency", EFFICIENCY_ATTR)],
        "feather_falling" => &[E::DamageProtection(
            ProtectionCondition::IsFall,
            FEATHER_FALLING_VALUE,
        )],
        "fire_aspect" => &[E::IgniteOnHit(FIRE_ASPECT_DURATION_SECONDS)],
        "fire_protection" => &[E::DamageProtection(
            ProtectionCondition::IsFire,
            FIRE_PROTECTION_VALUE,
        )],
        // "flame" (constant 100-tick ignite, not level-scaled), "fortune" (loot-table only, no
        // data component), and "frost_walker" (location_changed/replace_disk, no evaluator) all
        // fall through to the wildcard arm below.
        "impaling" => &[E::Damage(
            DamageCondition::SensitiveToImpaling,
            IMPALING_DAMAGE,
        )],
        "infinity" => &[E::AmmoUseSetZero],
        "knockback" => &[E::Knockback(KnockbackCondition::Always, KNOCKBACK_VALUE)],
        "looting" => &[E::EquipmentDropsChance(LOOTING_VALUE)],
        "loyalty" => &[E::TridentReturnAcceleration(LOYALTY_VALUE)],
        "luck_of_the_sea" => &[E::FishingLuckBonus(LUCK_OF_THE_SEA_VALUE)],
        // "lunge" is post_piercing_attack only (spear-specific push/exhaustion); falls through
        // to the wildcard arm below.
        "lure" => &[E::FishingTimeReduction(LURE_VALUE)],
        "mending" => &[E::RepairWithXpMultiply(MENDING_FACTOR)],
        "multishot" => &[
            E::ProjectileCount(MULTISHOT_COUNT),
            E::ProjectileSpread(MULTISHOT_SPREAD),
        ],
        "piercing" => &[E::ProjectilePiercing(PIERCING_VALUE)],
        "power" => &[E::Damage(
            DamageCondition::DirectAttackerIsArrow,
            POWER_DAMAGE,
        )],
        "projectile_protection" => &[E::DamageProtection(
            ProtectionCondition::IsProjectile,
            PROJECTILE_PROTECTION_VALUE,
        )],
        "protection" => &[E::DamageProtection(
            ProtectionCondition::Always,
            PROTECTION_VALUE,
        )],
        "punch" => &[E::Knockback(
            KnockbackCondition::AttackerIsArrow,
            PUNCH_VALUE,
        )],
        "quick_charge" => &[E::CrossbowChargeTime(QUICK_CHARGE_VALUE)],
        "respiration" => &[E::AttributeBonus("oxygen_bonus", RESPIRATION_ATTR)],
        "riptide" => &[E::TridentSpinAttackStrength(RIPTIDE_SPIN_VALUE)],
        "sharpness" => &[E::Damage(DamageCondition::Always, SHARPNESS_DAMAGE)],
        "silk_touch" => &[E::BlockExperienceSetZero],
        "smite" => &[E::Damage(DamageCondition::SensitiveToSmite, SMITE_DAMAGE)],
        "soul_speed" => &[E::AttributeBonus("movement_speed", SOUL_SPEED_SPEED_ATTR)],
        "sweeping_edge" => &[E::AttributeBonus(
            "sweeping_damage_ratio",
            SWEEPING_EDGE_RATIO,
        )],
        "swift_sneak" => &[E::AttributeBonus("sneaking_speed", SWIFT_SNEAK_ATTR)],
        "thorns" => &[E::ThornsChance(THORNS_CHANCE)],
        "unbreaking" => &[
            E::ItemDamageRemoveBinomial(UNBREAKING_ARMOR_CHANCE),
            E::ItemDamageRemoveBinomial(UNBREAKING_OTHER_CHANCE),
        ],
        "vanishing_curse" => &[E::PreventEquipmentDrop],
        "wind_burst" => &[E::PostAttackKnockbackMultiplier(WIND_BURST_VALUE)],
        _ => &[],
    }
}

/// Sums every `Damage` effect of `enchantment` that applies to `target_type`.
///
/// Convenience for the common case used by melee-attack call sites (player/mob damage
/// calculation), mirroring `EnchantmentHelper.modifyDamage` iterating every equipped/held
/// enchantment.
#[must_use]
pub fn damage_bonus(
    enchantment: &'static Enchantment,
    level: i32,
    target_type: &'static EntityType,
) -> f32 {
    effects_for(enchantment)
        .iter()
        .map(|effect| match effect {
            EnchantmentEffect::Damage(condition, value) if condition.applies(target_type) => {
                value.calculate(level)
            }
            _ => 0.0,
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_matches_vanilla_formula() {
        // base + per_level_above_first * (level - 1)
        assert_eq!(SHARPNESS_DAMAGE.calculate(1), 1.0);
        assert_eq!(SHARPNESS_DAMAGE.calculate(5), 3.0);
        assert_eq!(BANE_DAMAGE.calculate(1), 2.5);
        assert_eq!(BANE_DAMAGE.calculate(5), 12.5);
    }

    #[test]
    fn lookup_uses_table_within_range_and_fallback_beyond_it() {
        // wind_burst.json: values [1.2, 1.75, 2.2], fallback linear(1.5, 0.35).
        assert_eq!(WIND_BURST_VALUE.calculate(1), 1.2);
        assert_eq!(WIND_BURST_VALUE.calculate(2), 1.75);
        assert_eq!(WIND_BURST_VALUE.calculate(3), 2.2);
        // Levels above max_level=3 (reachable via NBT/commands) must fall back, not clamp to
        // the last table entry.
        assert_eq!(WIND_BURST_VALUE.calculate(4), 1.5 + 0.35 * 3.0);
    }

    #[test]
    fn density_matches_old_free_function_for_all_levels() {
        // Behavior-preserving-refactor check against the pre-framework `density_extra_damage`.
        for level in 1..=5 {
            let fall_distance = 4.0f32;
            let old = 0.5 * f64::from(level as u32) * f64::from(fall_distance);
            let new = f64::from(DENSITY_VALUE.calculate(level) * fall_distance);
            assert!(
                (old - new).abs() < 1e-9,
                "level {level}: old={old} new={new}"
            );
        }
    }

    #[test]
    fn breach_matches_old_free_function_for_all_levels() {
        for level in 1..=4 {
            let base = 0.8f32;
            let old = (base - 0.15 * level as f32).clamp(0.0, 1.0);
            let new = (base + BREACH_VALUE.calculate(level)).clamp(0.0, 1.0);
            assert!(
                (old - new).abs() < 1e-6,
                "level {level}: old={old} new={new}"
            );
        }
    }

    #[test]
    fn levels_squared_matches_efficiency_json() {
        // efficiency.json: levels_squared, added 1.0
        assert_eq!(EFFICIENCY_ATTR.calculate(1), 2.0);
        assert_eq!(EFFICIENCY_ATTR.calculate(5), 26.0);
    }

    #[test]
    fn fraction_matches_sweeping_edge_json() {
        // sweeping_edge.json: numerator linear(1,1), denominator linear(2,1)
        assert!((SWEEPING_EDGE_RATIO.calculate(1) - 0.5).abs() < 1e-6);
        assert!((SWEEPING_EDGE_RATIO.calculate(2) - (2.0 / 3.0)).abs() < 1e-6);
        assert!((SWEEPING_EDGE_RATIO.calculate(3) - 0.75).abs() < 1e-6);
    }

    #[test]
    fn fraction_returns_zero_for_zero_denominator() {
        const ZERO_DENOMINATOR: LevelBasedValue = LevelBasedValue::Constant(0.0);
        const NUMERATOR: LevelBasedValue = LevelBasedValue::Constant(5.0);
        const FRACTION: LevelBasedValue = LevelBasedValue::Fraction {
            numerator: &NUMERATOR,
            denominator: &ZERO_DENOMINATOR,
        };
        assert_eq!(FRACTION.calculate(1), 0.0);
    }

    #[test]
    fn clamped_bounds_the_inner_value() {
        const INNER: LevelBasedValue = LevelBasedValue::linear(-1.0, -1.0);
        const CLAMPED: LevelBasedValue = LevelBasedValue::Clamped {
            value: &INNER,
            min: 0.0,
            max: 10.0,
        };
        assert_eq!(CLAMPED.calculate(5), 0.0);
    }

    #[test]
    fn feather_falling_is_three_per_level_not_four() {
        // feather_falling.json: linear(base=3.0, per_level_above_first=3.0). Pumpkin's
        // pre-framework `living.rs` hardcoded `level * 4`, which over-reduces fall damage.
        assert_eq!(FEATHER_FALLING_VALUE.calculate(1), 3.0);
        assert_eq!(FEATHER_FALLING_VALUE.calculate(4), 12.0);
    }

    #[test]
    fn protection_family_matches_json_constants() {
        assert_eq!(PROTECTION_VALUE.calculate(4), 4.0);
        assert_eq!(FIRE_PROTECTION_VALUE.calculate(4), 8.0);
        assert_eq!(BLAST_PROTECTION_VALUE.calculate(4), 8.0);
        assert_eq!(PROJECTILE_PROTECTION_VALUE.calculate(4), 8.0);
    }

    #[test]
    fn knockback_and_punch_share_the_same_linear_formula() {
        assert_eq!(KNOCKBACK_VALUE.calculate(2), 2.0);
        assert_eq!(PUNCH_VALUE.calculate(2), 2.0);
    }

    #[test]
    fn sharpness_and_power_share_the_same_linear_formula() {
        assert_eq!(SHARPNESS_DAMAGE.calculate(5), 3.0);
        assert_eq!(POWER_DAMAGE.calculate(5), 3.0);
    }

    #[test]
    fn fire_aspect_duration_in_ticks_matches_existing_call_sites() {
        // fire_aspect.json linear(4,4) seconds * 20 ticks/sec == the pre-framework
        // `level * 80` used in player.rs / mob/mod.rs.
        for level in 1..=2 {
            let ticks = FIRE_ASPECT_DURATION_SECONDS.calculate(level) * 20.0;
            assert_eq!(ticks, level as f32 * 80.0);
        }
    }

    #[test]
    fn unbreaking_fractions_match_json() {
        // Armor: numerator linear(2,2), denominator linear(10,5) -> level1: 2/10=0.2
        assert!((UNBREAKING_ARMOR_CHANCE.calculate(1) - 0.2).abs() < 1e-6);
        // Other tools: numerator linear(1,1), denominator linear(2,1) -> level1: 1/2=0.5
        assert!((UNBREAKING_OTHER_CHANCE.calculate(1) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn effects_for_dispatches_every_enchantment_without_panicking() {
        for enchantment in Enchantment::all() {
            let _ = effects_for(enchantment);
        }
    }

    #[test]
    fn damage_bonus_gates_on_target_tag() {
        let skeleton = &pumpkin_data::entity::EntityType::SKELETON;
        let cow = &pumpkin_data::entity::EntityType::COW;
        assert_eq!(damage_bonus(&Enchantment::SMITE, 1, skeleton), 2.5);
        assert_eq!(damage_bonus(&Enchantment::SMITE, 1, cow), 0.0);
        assert_eq!(damage_bonus(&Enchantment::SHARPNESS, 1, cow), 1.0);
    }

    #[test]
    fn power_never_contributes_at_melee_call_sites() {
        // Power's damage requirement gates on the *direct attacker* being tagged
        // `#minecraft:arrows` (power.json), not on the victim's type. A melee call site (the
        // only kind `damage_bonus`/`DamageCondition::applies` can evaluate today) must never
        // apply it, regardless of level or victim — otherwise punching someone while holding a
        // Power bow silently adds bonus melee damage that vanilla never grants.
        let cow = &pumpkin_data::entity::EntityType::COW;
        assert_eq!(damage_bonus(&Enchantment::POWER, 5, cow), 0.0);
    }
}
