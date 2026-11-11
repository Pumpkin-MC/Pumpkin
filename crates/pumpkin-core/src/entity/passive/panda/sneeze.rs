//! Vanilla `Panda` sneezing: the sneeze counter flow of `Panda.tick`, `afterSneeze` and the
//! `PandaSneezeGoal` that starts it.

use std::sync::atomic::Ordering::Relaxed;

use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_protocol::codec::var_int::VarInt;
use rand::RngExt;

use super::{PandaEntity, as_panda};
use crate::entity::EntityBase;
use crate::entity::ageable::AgeableMob;
use crate::entity::ai::goal::{Goal, GoalReach, to_goal_ticks};
use crate::entity::mob::Mob;
use crate::world::loot::{LootContextParameters, generate_loot_from_handle};

/// `PandaSneezeGoal.canUse`: a weak baby sneezes on one in this many `nextInt` rolls.
const WEAK_BABY_SNEEZE_BOUND: i32 = 500;
/// `PandaSneezeGoal.canUse`: any baby sneezes on one in this many `nextInt` rolls.
const BABY_SNEEZE_BOUND: i32 = 6000;
/// `Panda.tick` ends the sneeze once the sneeze counter has passed this value.
const SNEEZE_COUNTER_END: i32 = 20;
/// `Panda.afterSneeze` makes the pandas inside the bounding box inflated by this much jump.
const SNEEZE_JUMP_RANGE: f64 = 10.0;
/// `BuiltInLootTables.PANDA_SNEEZE`.
const PANDA_SNEEZE_LOOT_TABLE: &str = "minecraft:gameplay/panda_sneeze";

/// Vanilla `PandaSneezeGoal.canUse` after the baby and `canPerformAction` checks: a weak panda
/// first rolls the short odds and falls back to the long odds, any other panda only rolls the
/// long ones. `next_int(bound)` stands for vanilla's `RandomSource.nextInt`, and both bounds go
/// through `reducedTickDelay` because the selector asks every second tick.
fn rolls_sneeze(weak: bool, mut next_int: impl FnMut(i32) -> i32) -> bool {
    (weak && next_int(to_goal_ticks(WEAK_BABY_SNEEZE_BOUND)) == 1)
        || next_int(to_goal_ticks(BABY_SNEEZE_BOUND)) == 1
}

impl PandaEntity {
    /// The condition of `PandaSneezeGoal.canUse` before the random roll.
    fn can_sneeze(&self) -> bool {
        self.is_baby() && self.can_perform_action()
    }

    /// Vanilla `Panda.sneeze`: starts or ends the sneeze flag; ending it resets the counter.
    pub fn sneeze(&self, value: bool) {
        self.set_flag(super::FLAG_SNEEZE, value);
        if !value {
            self.set_sneeze_counter(0);
        }
    }

    /// Vanilla `getSneezeCounter`.
    #[must_use]
    pub fn get_sneeze_counter(&self) -> i32 {
        self.sneeze_counter.load(Relaxed)
    }

    /// Vanilla `setSneezeCounter`; the counter is synched so the client can animate the sneeze.
    pub fn set_sneeze_counter(&self, value: i32) {
        self.sneeze_counter.store(value, Relaxed);
        self.get_entity().set_synced_data(
            pumpkin_data::tracked_data::panda::SNEEZE_COUNTER,
            VarInt(value),
        );
    }

    /// The sneeze part of vanilla `Panda.tick`: while the flag is set the counter grows by one
    /// per tick, plays `PANDA_PRE_SNEEZE` at 1 and ends the sneeze with `afterSneeze` once it
    /// passes 20.
    pub(super) fn tick_sneeze(&self) {
        if !self.is_sneezing() {
            return;
        }
        let counter = self.get_sneeze_counter() + 1;
        self.set_sneeze_counter(counter);
        if counter > SNEEZE_COUNTER_END {
            self.sneeze(false);
            self.after_sneeze();
        } else if counter == 1 {
            self.play_entity_sound(Sound::EntityPandaPreSneeze, 1.0, 1.0);
        }
    }

    /// Vanilla `Panda.afterSneeze`: plays `PANDA_SNEEZE`, makes the adult pandas within 10
    /// blocks jump and may drop the sneeze gift. The sneeze particle is added by the client
    /// from the synched sneeze counter, so no particle packet is sent.
    ///
    /// This reads and changes other entities up to 10 blocks away from the panda's tick, which
    /// the conservative entity island reach covers only because `has_audited_tick` is false for
    /// pandas; audit it before ever setting that.
    fn after_sneeze(&self) {
        self.play_entity_sound(Sound::EntityPandaSneeze, 1.0, 1.0);

        let entity = self.get_entity();
        let world = entity.world.load_full();
        let area = entity.bounding_box.load().expand(
            SNEEZE_JUMP_RANGE,
            SNEEZE_JUMP_RANGE,
            SNEEZE_JUMP_RANGE,
        );
        for other in world.get_entities_at_box(&area) {
            if let Some(panda) = other.cast_any().downcast_ref::<Self>()
                && !panda.is_baby()
                && panda.get_entity().on_ground.load(Relaxed)
                && !panda.get_entity().is_in_water()
                && panda.can_perform_action()
            {
                panda
                    .mob_entity
                    .living_entity
                    .jump_from_ground(panda.jump_power_multiplier());
            }
        }

        self.drop_sneeze_gift(self.get_random().random());
    }

    /// The loot rolled by `dropFromGiftLootTable` with `PANDA_SNEEZE` for a loot seed; empty if
    /// the table is unknown.
    fn roll_sneeze_gift(&self, seed: i64) -> Vec<ItemStack> {
        let world = self.get_entity().world.load_full();
        let Some(table) = world.get_loot_table(PANDA_SNEEZE_LOOT_TABLE) else {
            return Vec::new();
        };
        let params = LootContextParameters {
            world: Some(&world),
            position: Some(self.get_entity().pos.load()),
            this_entity: Some(self),
            ..Default::default()
        };
        generate_loot_from_handle(&table, seed, &params)
    }

    /// The end of vanilla `afterSneeze`: with `mob_drops` on, drops the `PANDA_SNEEZE` gift at
    /// the panda's feet. A plugin can cancel each stack through `EntityDropItemEvent`. Returns
    /// whether an item was dropped.
    fn drop_sneeze_gift(&self, seed: i64) -> bool {
        let entity = self.get_entity();
        let world = entity.world.load_full();
        if !world.level_info.load().game_rules.mob_drops {
            return false;
        }
        let mut dropped = false;
        for stack in self.roll_sneeze_gift(seed) {
            dropped |= self.spawn_stack_at_feet(stack);
        }
        dropped
    }
}

/// Vanilla `PandaSneezeGoal` (goal priority 8, no control flags): a baby that can act sneezes
/// now and then, a weak one much more often. The goal only raises the sneeze flag; the counter,
/// the sounds and the effects are driven by the panda tick.
#[derive(Default)]
pub struct PandaSneezeGoal;

impl Goal for PandaSneezeGoal {
    /// Vanilla `PandaSneezeGoal.canUse`: a baby that `canPerformAction` and wins the roll.
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(panda) = as_panda(mob) else {
            return false;
        };
        if !panda.can_sneeze() {
            return false;
        }
        let mut rng = mob.get_random();
        rolls_sneeze(panda.is_weak(), |bound| rng.random_range(0..bound))
    }

    /// Vanilla `PandaSneezeGoal.canContinueToUse`: never, the goal is over right after `start`.
    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        false
    }

    /// Vanilla `PandaSneezeGoal.start`: `sneeze(true)`.
    fn start(&mut self, mob: &dyn Mob) {
        if let Some(panda) = as_panda(mob) {
            panda.sneeze(true);
        }
    }

    /// The goal only reads the panda itself and raises its sneeze flag, so it reaches no other
    /// entity. The 10 block sneeze effect belongs to the panda tick, not to this goal.
    fn tick_reach(&self, _mob: &dyn Mob, _running: bool) -> Option<GoalReach> {
        Some(GoalReach::default())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use pumpkin_data::item::Item;

    use super::super::test_support::{TestResult, scripted, spawn_panda, stone_floor};
    use super::*;
    use crate::entity::ai::test_support::world;

    #[test]
    fn weak_babies_roll_the_short_odds_first_and_others_only_the_long_ones() {
        // `reducedTickDelay` halves the bounds 500 and 6000.
        let rolls =
            |weak: bool, calls: &[(i32, i32)]| scripted(calls, |next| rolls_sneeze(weak, next));
        assert!(rolls(true, &[(250, 1)]));
        assert!(rolls(true, &[(250, 0), (3000, 1)]));
        assert!(!rolls(true, &[(250, 0), (3000, 0)]));
        assert!(rolls(false, &[(3000, 1)]));
        assert!(!rolls(false, &[(3000, 0)]));
    }

    #[tokio::test]
    async fn a_sneeze_lasts_21_ticks_and_only_grounded_adults_that_can_act_within_ten_blocks_jump()
    -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        stone_floor(&world);
        let sneezer = spawn_panda(&world, 0.5, 0.5);
        let near = spawn_panda(&world, 5.5, 0.5);
        let edge = spawn_panda(&world, 10.5, 0.5);
        let far = spawn_panda(&world, 13.5, 0.5);
        let baby = spawn_panda(&world, 2.5, 3.5);
        baby.set_age(-24000);
        let sitting = spawn_panda(&world, 2.5, -3.5);
        sitting.set_sitting(true);
        let airborne = spawn_panda(&world, -3.5, 3.5);
        airborne.get_entity().on_ground.store(false, Relaxed);
        let swimming = spawn_panda(&world, -3.5, -3.5);
        swimming.get_entity().touching_water.store(true, Relaxed);
        let jump_speed = |panda: &PandaEntity| panda.get_entity().velocity.load().y;

        sneezer.set_age(-24000);
        sneezer.sneeze(true);
        for tick in 1..=SNEEZE_COUNTER_END {
            sneezer.tick_sneeze();
            assert!(sneezer.is_sneezing(), "the sneeze ended at tick {tick}");
            assert_eq!(sneezer.get_sneeze_counter(), tick);
            if tick == 10 {
                // Neither growing up nor sitting down interrupts a sneeze that has started.
                sneezer.set_age(0);
                sneezer.set_sitting(true);
            }
        }
        assert!(
            jump_speed(&near).abs() < 1e-9,
            "jumped before the sneeze ended"
        );

        sneezer.tick_sneeze();
        assert!(!sneezer.is_sneezing());
        assert_eq!(sneezer.get_sneeze_counter(), 0);
        for (name, jumped, panda) in [
            ("near adult", true, &near),
            ("adult at the edge of the box", true, &edge),
            ("adult out of range", false, &far),
            ("baby", false, &baby),
            ("sitting adult", false, &sitting),
            ("airborne adult", false, &airborne),
            ("adult in water", false, &swimming),
        ] {
            assert_eq!(jump_speed(panda) > 0.3, jumped, "{name}");
        }
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn sneeze_gift_drops_at_the_panda_only_with_mob_drops_on() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let panda = spawn_panda(&world, 0.5, 0.5);
        let winning_seed = (0..100_000)
            .find(|seed| !panda.roll_sneeze_gift(*seed).is_empty())
            .ok_or("no seed wins the gift")?;
        let gift = panda.roll_sneeze_gift(winning_seed);
        assert_eq!(gift.len(), 1);
        assert_eq!(gift[0].item.id, Item::SLIME_BALL.id);
        let items_near = || {
            let area = panda.get_entity().bounding_box.load().expand(2.0, 2.0, 2.0);
            world
                .get_entities_at_box(&area)
                .iter()
                .filter(|entity| entity.get_item_entity().is_some())
                .count()
        };

        let mut level_info = (**world.level_info.load()).clone();
        level_info.game_rules.mob_drops = false;
        world.level_info.store(Arc::new(level_info.clone()));
        assert!(!panda.drop_sneeze_gift(winning_seed));
        assert_eq!(items_near(), 0, "dropped with mob drops off");

        level_info.game_rules.mob_drops = true;
        world.level_info.store(Arc::new(level_info));
        assert!(panda.drop_sneeze_gift(winning_seed));
        assert_eq!(items_near(), 1);
        world.level.shutdown().await;
        Ok(())
    }
}
