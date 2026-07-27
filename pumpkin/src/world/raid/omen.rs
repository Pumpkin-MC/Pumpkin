//! The 26.2 Bad Omen → Raid Omen → raid trigger path.
//!
//! Ground truth:
//! - `/root/Vanilla/src/net/minecraft/world/effect/BadOmenMobEffect.java`
//! - `/root/Vanilla/src/net/minecraft/world/effect/RaidOmenMobEffect.java`
//! - `/root/Vanilla/src/net/minecraft/world/entity/raid/Raids.java:106-141`
//!
//! # The 26.2 mechanic, as verified in the decompiled source
//!
//! Killing a raid captain does **not** grant Bad Omen in 26.2. That was the
//! pre-1.21 behaviour. An exhaustive search of `/root/Vanilla/src` for
//! `MobEffects.BAD_OMEN` finds exactly five sites: the effect registration
//! (`MobEffects.java:63`), `OminousBottleAmplifier.java:41` and `:46`, and the
//! trial-spawner conversion (`TrialSpawnerStateData.java:147`, `:178`, `:207`,
//! `:213`). None of them is a captain kill, and `Raider.die`
//! (`Raider.java:127-144`) grants no effect to the killer either.
//!
//! Instead, killing a captain drops an **Ominous Bottle**: the pillager loot table
//! (`VanillaEntityLoot.java:114`) adds `Items.OMINOUS_BOTTLE` with
//! `SetOminousBottleAmplifierFunction` (uniform 0-4), gated on
//! `RaiderPredicate.CAPTAIN_WITHOUT_RAID`. Drinking the bottle applies Bad Omen at
//! that amplifier for 120000 ticks (`OminousBottleAmplifier.java:41`).
//!
//! From there the two-stage conversion runs:
//!
//! 1. **Bad Omen → Raid Omen** (`BadOmenMobEffect.applyEffectTick`). Ticks every
//!    tick (`shouldApplyEffectTickThisTick` returns `true`). When the carrier is a
//!    non-spectator player, the difficulty is not Peaceful, the player *is standing
//!    in a village*, and any raid there is below its max omen level, it applies
//!    `RAID_OMEN` for 600 ticks at the same amplifier, records the player's
//!    `raidOmenPosition`, and returns `false` — which removes Bad Omen.
//! 2. **Raid Omen → raid** (`RaidOmenMobEffect.applyEffectTick`). Ticks only on the
//!    final tick (`shouldApplyEffectTickThisTick` is `remainingDuration == 1`), then
//!    calls `Raids.createOrExtendRaid(player, raidOmenPosition)` and clears the
//!    stored position.
//!
//! So the village requirement sits on the *Bad Omen* side, and the raid starts 30
//! seconds later at the position where the player entered the village.

use std::sync::Arc;

use pumpkin_data::effect::StatusEffect;
use pumpkin_data::potion::Effect;
use pumpkin_util::Difficulty;
use pumpkin_util::math::position::BlockPos;

use crate::entity::player::Player;
use crate::world::World;

use super::village;

/// Vanilla `BadOmenMobEffect` applies `RAID_OMEN` for 600 ticks
/// (`BadOmenMobEffect.java:33`).
pub const RAID_OMEN_DURATION: i32 = 600;

/// Vanilla `OminousBottleAmplifier` applies Bad Omen for 120000 ticks
/// (`OminousBottleAmplifier.java:41`).
pub const BAD_OMEN_DURATION: i32 = 120_000;

/// Whether Bad Omen should convert this tick — the guard set of
/// `BadOmenMobEffect.applyEffectTick` (`BadOmenMobEffect.java:28-35`).
///
/// Returns `true` when vanilla would convert (and therefore *remove* Bad Omen).
#[must_use]
pub fn should_convert_bad_omen(world: &Arc<World>, player: &Player) -> bool {
    // `mob instanceof ServerPlayer && !player.isSpectator()`
    if player.is_spectator() {
        return false;
    }
    // `level.getDifficulty() != Difficulty.PEACEFUL`
    if world.level_info.load().difficulty == Difficulty::Peaceful {
        return false;
    }
    let pos = player.living_entity.entity.block_pos.load();
    // `level.isVillage(player.blockPosition())`
    if !village::is_village(world, &pos) {
        return false;
    }
    // `raid == null || raid.getRaidOmenLevel() < raid.getMaxRaidOmenLevel()`
    world
        .raids
        .raid_at(&pos)
        .is_none_or(|raid| raid.raid_omen_level() < raid.max_raid_omen_level())
}

/// Vanilla `BadOmenMobEffect.applyEffectTick` (`BadOmenMobEffect.java:26-37`).
///
/// Applies Raid Omen at the same amplifier, stores the raid-omen position, and
/// reports whether Bad Omen must now be removed (vanilla's `return false`).
pub async fn convert_bad_omen(world: &Arc<World>, player: &Arc<Player>, amplifier: u8) -> bool {
    if !should_convert_bad_omen(world, player) {
        return false;
    }

    let pos = player.living_entity.entity.block_pos.load();
    // BadOmenMobEffect.java:32 — `new MobEffectInstance(RAID_OMEN, 600, amplification)`.
    // That three-argument constructor uses ambient = false, showParticles = true,
    // showIcon = true.
    player
        .add_effect(Effect {
            effect_type: &StatusEffect::RAID_OMEN,
            duration: RAID_OMEN_DURATION,
            amplifier,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        })
        .await;

    // BadOmenMobEffect.java:33 — `player.setRaidOmenPosition(player.blockPosition())`.
    player.raid_omen_position.store(Some(pos));
    true
}

/// Vanilla `RaidOmenMobEffect.applyEffectTick` (`RaidOmenMobEffect.java:26-38`).
///
/// Only called on the effect's final tick, matching
/// `shouldApplyEffectTickThisTick == (remainingDuration == 1)`
/// (`RaidOmenMobEffect.java:22-24`). Returns whether Raid Omen must be removed.
pub async fn trigger_raid_from_omen(world: &Arc<World>, player: &Arc<Player>) -> bool {
    // `mob instanceof ServerPlayer && !mob.isSpectator()`
    if player.is_spectator() {
        return false;
    }
    // `(raidOmenPosition = player.getRaidOmenPosition()) != null`
    let Some(position) = player.raid_omen_position.load() else {
        return false;
    };

    // RaidOmenMobEffect.java:31.
    world
        .raids
        .create_or_extend_raid(world, player, position)
        .await;
    // RaidOmenMobEffect.java:32.
    player.raid_omen_position.store(None);
    true
}

/// Whether this effect ticks on this frame, for the two omen effects.
///
/// - Bad Omen: `shouldApplyEffectTickThisTick` returns `true` unconditionally
///   (`BadOmenMobEffect.java:21-24`).
/// - Raid Omen: returns `remainingDuration == 1` (`RaidOmenMobEffect.java:21-24`).
///
/// Returns `None` for any other effect so the caller can fall through to its own
/// table.
#[must_use]
pub fn should_apply_omen_tick(
    effect_type: &'static StatusEffect,
    remaining_duration: i32,
) -> Option<bool> {
    if effect_type == &StatusEffect::BAD_OMEN {
        Some(true)
    } else if effect_type == &StatusEffect::RAID_OMEN {
        Some(remaining_duration == 1)
    } else {
        None
    }
}

/// Convenience for the loot/consume path: the Bad Omen instance an Ominous Bottle
/// applies (`OminousBottleAmplifier.java:41`).
///
/// Vanilla's six-argument constructor there is
/// `(BAD_OMEN, 120000, value, false, false, true)` — ambient false, particles
/// hidden, icon shown.
#[must_use]
pub const fn ominous_bottle_effect(amplifier: u8) -> Effect {
    Effect {
        effect_type: &StatusEffect::BAD_OMEN,
        duration: BAD_OMEN_DURATION,
        amplifier,
        ambient: false,
        show_particles: false,
        show_icon: true,
        blend: false,
    }
}

/// The raid-omen position a player carries between the two stages.
///
/// Vanilla stores this on `ServerPlayer.raidOmenPosition`
/// (`ServerPlayer.java:302`) and persists it as `raid_omen_position`
/// (`ServerPlayer.java:454`, `:473`).
pub type RaidOmenPosition = Option<BlockPos>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bad_omen_ticks_every_tick() {
        assert_eq!(
            should_apply_omen_tick(&StatusEffect::BAD_OMEN, 500),
            Some(true)
        );
        assert_eq!(
            should_apply_omen_tick(&StatusEffect::BAD_OMEN, 1),
            Some(true)
        );
    }

    #[test]
    fn raid_omen_only_fires_on_its_last_tick() {
        assert_eq!(
            should_apply_omen_tick(&StatusEffect::RAID_OMEN, 600),
            Some(false)
        );
        assert_eq!(
            should_apply_omen_tick(&StatusEffect::RAID_OMEN, 2),
            Some(false)
        );
        assert_eq!(
            should_apply_omen_tick(&StatusEffect::RAID_OMEN, 1),
            Some(true)
        );
    }

    #[test]
    fn other_effects_fall_through() {
        assert_eq!(should_apply_omen_tick(&StatusEffect::POISON, 1), None);
        assert_eq!(
            should_apply_omen_tick(&StatusEffect::HERO_OF_THE_VILLAGE, 1),
            None
        );
    }

    #[test]
    fn ominous_bottle_matches_the_vanilla_instance() {
        let effect = ominous_bottle_effect(3);
        assert_eq!(effect.effect_type, &StatusEffect::BAD_OMEN);
        assert_eq!(effect.duration, 120_000);
        assert_eq!(effect.amplifier, 3);
        assert!(!effect.ambient);
        assert!(!effect.show_particles);
        assert!(effect.show_icon);
    }

    #[test]
    fn raid_omen_duration_is_thirty_seconds() {
        // 600 ticks = 30s; the delay between entering the village and the horn.
        assert_eq!(RAID_OMEN_DURATION, 600);
    }
}
