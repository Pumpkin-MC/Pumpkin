//! Raid wave composition — the exact vanilla tables.
//!
//! Ground truth: `/root/Vanilla/src/net/minecraft/world/entity/raid/Raid.java`
//! (`RaiderType` enum at lines 735-754, `getDefaultNumSpawns` at 635-637,
//! `getPotentialBonusSpawns` at 639-674, `getNumGroups` at 680-688,
//! `getEnchantOdds` at 690-705).
//!
//! Everything here is pure: no `World`, no `Server`, no randomness except where
//! a caller passes an explicit draw. That keeps the tables unit-testable.

use pumpkin_data::entity::EntityType;
use pumpkin_util::difficulty::Difficulty;

/// Vanilla `Raid.RaiderType` (`Raid.java:735-754`).
///
/// Declaration order is load-bearing twice over: `spawnGroup` iterates
/// `RaiderType.VALUES` in this order (`Raid.java:462`), and
/// `getPotentialBonusSpawns` switches on `type.ordinal()` (`Raid.java:644`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RaiderType {
    /// Ordinal 0.
    Vindicator,
    /// Ordinal 1.
    Evoker,
    /// Ordinal 2.
    Pillager,
    /// Ordinal 3.
    Witch,
    /// Ordinal 4.
    Ravager,
}

impl RaiderType {
    /// Vanilla `RaiderType.VALUES` (`Raid.java:742`, `Raid.java:752`) — the
    /// iteration order used by `Raid.spawnGroup`.
    pub const VALUES: [Self; 5] = [
        Self::Vindicator,
        Self::Evoker,
        Self::Pillager,
        Self::Witch,
        Self::Ravager,
    ];

    /// Vanilla `RaiderType.spawnsPerWaveBeforeBonus` (`Raid.java:736-740`).
    ///
    /// Indexed by 1-based wave number; index 0 is unused padding, exactly as in
    /// vanilla. Hard mode reaches wave 7, so all 8 slots exist.
    #[must_use]
    pub const fn spawns_per_wave_before_bonus(self) -> [i32; 8] {
        match self {
            // VINDICATOR(EntityTypes.VINDICATOR, new int[]{0, 0, 2, 0, 1, 4, 2, 5})
            Self::Vindicator => [0, 0, 2, 0, 1, 4, 2, 5],
            // EVOKER(EntityTypes.EVOKER, new int[]{0, 0, 0, 0, 0, 1, 1, 2})
            Self::Evoker => [0, 0, 0, 0, 0, 1, 1, 2],
            // PILLAGER(EntityTypes.PILLAGER, new int[]{0, 4, 3, 3, 4, 4, 4, 2})
            Self::Pillager => [0, 4, 3, 3, 4, 4, 4, 2],
            // WITCH(EntityTypes.WITCH, new int[]{0, 0, 0, 0, 3, 0, 0, 1})
            Self::Witch => [0, 0, 0, 0, 3, 0, 0, 1],
            // RAVAGER(EntityTypes.RAVAGER, new int[]{0, 0, 0, 1, 0, 1, 0, 2})
            Self::Ravager => [0, 0, 0, 1, 0, 1, 0, 2],
        }
    }

    /// The entity type spawned for this raider slot (`Raid.java:736-740`).
    #[must_use]
    pub const fn entity_type(self) -> &'static EntityType {
        match self {
            Self::Vindicator => &EntityType::VINDICATOR,
            Self::Evoker => &EntityType::EVOKER,
            Self::Pillager => &EntityType::PILLAGER,
            Self::Witch => &EntityType::WITCH,
            Self::Ravager => &EntityType::RAVAGER,
        }
    }

    /// Vanilla `Raid.getDefaultNumSpawns` (`Raid.java:635-637`).
    ///
    /// The bonus wave reuses the final-wave column (`spawnsPerWaveBeforeBonus[numGroups]`)
    /// rather than a column of its own.
    #[must_use]
    pub fn default_num_spawns(self, wave: i32, num_groups: i32, is_bonus_wave: bool) -> i32 {
        let table = self.spawns_per_wave_before_bonus();
        let index = if is_bonus_wave { num_groups } else { wave };
        usize::try_from(index)
            .ok()
            .and_then(|i| table.get(i).copied())
            .unwrap_or(0)
    }
}

/// Resolved cap for the extra-spawn roll, before the random draw.
///
/// Vanilla computes `bonusSpawns` and then applies `random.nextInt(bonusSpawns + 1)`
/// (`Raid.java:673`). On Easy the cap is itself a random draw, so it is modelled
/// separately instead of being folded into a single number.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BonusCap {
    /// Vanilla returns 0 outright — no roll happens at all.
    None,
    /// `bonusSpawns` is this constant.
    Fixed(i32),
    /// Easy-mode vindicators/pillagers: `bonusSpawns = random.nextInt(2)`.
    EasyRandomTwo,
}

/// Vanilla `Raid.getPotentialBonusSpawns` (`Raid.java:639-674`), split so the
/// branch logic is testable without a random source.
///
/// The `switch (type.ordinal())` in vanilla maps to: case 3 = `Witch`,
/// cases 0 and 2 = `Vindicator`/`Pillager`, case 4 = `Ravager`, default
/// (case 1 = `Evoker`) returns 0.
#[must_use]
pub const fn bonus_cap(
    raider: RaiderType,
    wave: i32,
    difficulty: Difficulty,
    is_bonus_wave: bool,
) -> BonusCap {
    let is_easy = matches!(difficulty, Difficulty::Easy);
    let is_normal = matches!(difficulty, Difficulty::Normal);

    match raider {
        // case 3: if (!isEasy && wav > 2 && wav != 4) bonusSpawns = 1; else return 0;
        RaiderType::Witch => {
            if !is_easy && wave > 2 && wave != 4 {
                BonusCap::Fixed(1)
            } else {
                BonusCap::None
            }
        }
        // case 0, 2: easy -> random.nextInt(2); normal -> 1; hard -> 2
        RaiderType::Vindicator | RaiderType::Pillager => {
            if is_easy {
                BonusCap::EasyRandomTwo
            } else if is_normal {
                BonusCap::Fixed(1)
            } else {
                BonusCap::Fixed(2)
            }
        }
        // case 4: bonusSpawns = !isEasy && isBonusWave ? 1 : 0;
        RaiderType::Ravager => {
            if !is_easy && is_bonus_wave {
                BonusCap::Fixed(1)
            } else {
                // Vanilla falls through to `bonusSpawns > 0 ? ... : 0`, which is 0.
                BonusCap::None
            }
        }
        // default: return 0 (EVOKER)
        RaiderType::Evoker => BonusCap::None,
    }
}

/// Vanilla `Raid.getNumGroups` (`Raid.java:680-688`).
#[must_use]
pub const fn num_groups(difficulty: Difficulty) -> i32 {
    match difficulty {
        Difficulty::Peaceful => 0,
        Difficulty::Easy => 3,
        Difficulty::Normal => 5,
        Difficulty::Hard => 7,
    }
}

/// Vanilla `Raid.getEnchantOdds` (`Raid.java:690-705`).
#[must_use]
pub fn enchant_odds(raid_omen_level: i32) -> f32 {
    match raid_omen_level {
        2 => 0.1,
        3 => 0.25,
        4 => 0.5,
        5 => 0.75,
        _ => 0.0,
    }
}

/// Vanilla `Raid.DEFAULT_MAX_RAID_OMEN_LEVEL` (`Raid.java:100`), returned by
/// `getMaxRaidOmenLevel` (`Raid.java:216-218`).
pub const MAX_RAID_OMEN_LEVEL: i32 = 5;

/// Vanilla `Raid.absorbRaidOmen` omen arithmetic (`Raid.java:228-240`).
///
/// The stored level gains `amplifier + 1` and is then clamped into
/// `[0, MAX_RAID_OMEN_LEVEL]`.
#[must_use]
pub const fn absorb_raid_omen_level(current: i32, effect_amplifier: i32) -> i32 {
    let raised = current + effect_amplifier + 1;
    // Written out rather than `Mth.clamp`'s `i32::clamp`, which is an `Ord`
    // method and therefore not callable in a `const fn`.
    if raised < 0 {
        0
    } else if raised > MAX_RAID_OMEN_LEVEL {
        MAX_RAID_OMEN_LEVEL
    } else {
        raised
    }
}

/// The mount rider that vanilla pairs with a ravager (`Raid.java:473-479`).
///
/// `groupNumber == getNumGroups(NORMAL)` (5) puts a pillager on the ravager;
/// `groupNumber >= getNumGroups(HARD)` (7) puts an evoker on the first ravager
/// of the wave and vindicators on the rest.
#[must_use]
pub fn ravager_rider(wave: i32, ravagers_spawned_before: i32) -> Option<&'static EntityType> {
    if wave == num_groups(Difficulty::Normal) {
        Some(&EntityType::PILLAGER)
    } else if wave >= num_groups(Difficulty::Hard) {
        if ravagers_spawned_before == 0 {
            Some(&EntityType::EVOKER)
        } else {
            Some(&EntityType::VINDICATOR)
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wave_tables_match_vanilla_arrays() {
        // Raid.java:736-740, verbatim.
        assert_eq!(
            RaiderType::Vindicator.spawns_per_wave_before_bonus(),
            [0, 0, 2, 0, 1, 4, 2, 5]
        );
        assert_eq!(
            RaiderType::Evoker.spawns_per_wave_before_bonus(),
            [0, 0, 0, 0, 0, 1, 1, 2]
        );
        assert_eq!(
            RaiderType::Pillager.spawns_per_wave_before_bonus(),
            [0, 4, 3, 3, 4, 4, 4, 2]
        );
        assert_eq!(
            RaiderType::Witch.spawns_per_wave_before_bonus(),
            [0, 0, 0, 0, 3, 0, 0, 1]
        );
        assert_eq!(
            RaiderType::Ravager.spawns_per_wave_before_bonus(),
            [0, 0, 0, 1, 0, 1, 0, 2]
        );
    }

    #[test]
    fn raider_iteration_order_is_vanilla_ordinal_order() {
        assert_eq!(
            RaiderType::VALUES,
            [
                RaiderType::Vindicator,
                RaiderType::Evoker,
                RaiderType::Pillager,
                RaiderType::Witch,
                RaiderType::Ravager,
            ]
        );
    }

    #[test]
    fn first_wave_is_four_pillagers_only() {
        // Wave 1 column across all raider types: only PILLAGER has 4.
        for raider in RaiderType::VALUES {
            let expected = i32::from(raider == RaiderType::Pillager) * 4;
            assert_eq!(raider.default_num_spawns(1, 5, false), expected);
        }
    }

    #[test]
    fn bonus_wave_reuses_the_final_wave_column() {
        // Easy: numGroups 3 -> bonus wave uses index 3.
        assert_eq!(RaiderType::Ravager.default_num_spawns(99, 3, true), 1);
        assert_eq!(RaiderType::Pillager.default_num_spawns(99, 3, true), 3);
        // Hard: numGroups 7 -> index 7.
        assert_eq!(RaiderType::Vindicator.default_num_spawns(99, 7, true), 5);
        assert_eq!(RaiderType::Evoker.default_num_spawns(99, 7, true), 2);
    }

    #[test]
    fn out_of_range_wave_yields_no_spawns() {
        // Guards the table lookup rather than panicking like a raw Java index.
        assert_eq!(RaiderType::Pillager.default_num_spawns(8, 7, false), 0);
        assert_eq!(RaiderType::Pillager.default_num_spawns(-1, 7, false), 0);
    }

    #[test]
    fn num_groups_per_difficulty() {
        assert_eq!(num_groups(Difficulty::Peaceful), 0);
        assert_eq!(num_groups(Difficulty::Easy), 3);
        assert_eq!(num_groups(Difficulty::Normal), 5);
        assert_eq!(num_groups(Difficulty::Hard), 7);
    }

    #[test]
    fn evoker_never_gets_bonus_spawns() {
        for wave in 1..=7 {
            for difficulty in [Difficulty::Easy, Difficulty::Normal, Difficulty::Hard] {
                for bonus in [false, true] {
                    assert_eq!(
                        bonus_cap(RaiderType::Evoker, wave, difficulty, bonus),
                        BonusCap::None
                    );
                }
            }
        }
    }

    #[test]
    fn witch_bonus_skips_easy_and_waves_one_two_four() {
        assert_eq!(
            bonus_cap(RaiderType::Witch, 3, Difficulty::Easy, false),
            BonusCap::None
        );
        assert_eq!(
            bonus_cap(RaiderType::Witch, 2, Difficulty::Hard, false),
            BonusCap::None
        );
        assert_eq!(
            bonus_cap(RaiderType::Witch, 4, Difficulty::Hard, false),
            BonusCap::None
        );
        assert_eq!(
            bonus_cap(RaiderType::Witch, 3, Difficulty::Normal, false),
            BonusCap::Fixed(1)
        );
        assert_eq!(
            bonus_cap(RaiderType::Witch, 5, Difficulty::Hard, false),
            BonusCap::Fixed(1)
        );
    }

    #[test]
    fn vindicator_and_pillager_bonus_scales_with_difficulty() {
        for raider in [RaiderType::Vindicator, RaiderType::Pillager] {
            assert_eq!(
                bonus_cap(raider, 3, Difficulty::Easy, false),
                BonusCap::EasyRandomTwo
            );
            assert_eq!(
                bonus_cap(raider, 3, Difficulty::Normal, false),
                BonusCap::Fixed(1)
            );
            assert_eq!(
                bonus_cap(raider, 3, Difficulty::Hard, false),
                BonusCap::Fixed(2)
            );
        }
    }

    #[test]
    fn ravager_bonus_only_on_non_easy_bonus_wave() {
        assert_eq!(
            bonus_cap(RaiderType::Ravager, 3, Difficulty::Hard, true),
            BonusCap::Fixed(1)
        );
        assert_eq!(
            bonus_cap(RaiderType::Ravager, 3, Difficulty::Easy, true),
            BonusCap::None
        );
        assert_eq!(
            bonus_cap(RaiderType::Ravager, 3, Difficulty::Hard, false),
            BonusCap::None
        );
    }

    #[test]
    fn enchant_odds_table() {
        assert!((enchant_odds(0) - 0.0).abs() < f32::EPSILON);
        assert!((enchant_odds(1) - 0.0).abs() < f32::EPSILON);
        assert!((enchant_odds(2) - 0.1).abs() < f32::EPSILON);
        assert!((enchant_odds(3) - 0.25).abs() < f32::EPSILON);
        assert!((enchant_odds(4) - 0.5).abs() < f32::EPSILON);
        assert!((enchant_odds(5) - 0.75).abs() < f32::EPSILON);
    }

    #[test]
    fn omen_absorption_adds_amplifier_plus_one_and_clamps() {
        // Bad Omen I (amplifier 0) on a fresh raid -> level 1.
        assert_eq!(absorb_raid_omen_level(0, 0), 1);
        // Bad Omen III (amplifier 2) -> level 3.
        assert_eq!(absorb_raid_omen_level(0, 2), 3);
        // Stacking saturates at the max.
        assert_eq!(absorb_raid_omen_level(4, 2), MAX_RAID_OMEN_LEVEL);
        assert_eq!(absorb_raid_omen_level(5, 4), MAX_RAID_OMEN_LEVEL);
        // Never goes negative.
        assert_eq!(absorb_raid_omen_level(0, -5), 0);
    }

    #[test]
    fn ravager_riders_match_wave_thresholds() {
        assert_eq!(ravager_rider(3, 0), None);
        assert_eq!(ravager_rider(4, 0), None);
        assert_eq!(ravager_rider(5, 0), Some(&EntityType::PILLAGER));
        assert_eq!(ravager_rider(6, 0), None);
        assert_eq!(ravager_rider(7, 0), Some(&EntityType::EVOKER));
        assert_eq!(ravager_rider(7, 1), Some(&EntityType::VINDICATOR));
        assert_eq!(ravager_rider(8, 2), Some(&EntityType::VINDICATOR));
    }
}
