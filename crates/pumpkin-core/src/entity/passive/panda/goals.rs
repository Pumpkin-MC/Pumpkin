//! Vanilla `Panda` inner goals that the shared goals cannot express.

use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;

use pumpkin_data::Block;
use pumpkin_util::math::position::BlockPos;
use rand::RngExt;

use super::{PandaEntity, TOTAL_UNHAPPY_TIME, as_panda};
use crate::entity::EntityBase;
use crate::entity::ageable::AgeableMob;
use crate::entity::ai::goal::breed::BreedGoal;
use crate::entity::ai::goal::look_at_entity::LookTargetSlot;
use crate::entity::ai::goal::melee_attack::MeleeAttackGoal;
use crate::entity::ai::goal::revenge::RevengeGoal;
use crate::entity::ai::goal::{Controls, Goal, GoalReach, to_goal_ticks};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::mob::Mob;

/// `PandaBreedGoal` looks for bamboo in rings of radius 0 to 7 and at 0 to 2 blocks above.
const BAMBOO_SEARCH_RADIUS: i32 = 7;
const BAMBOO_SEARCH_HEIGHT: i32 = 2;
/// `canContinueToUse` of `PandaSitGoal`: 1 in this many goal ticks ends the sit, unless lazy.
const SIT_END_ODDS: i32 = 600;
/// `canContinueToUse` of `PandaSitGoal`: 1 in this many goal ticks ends the sit, lazy or not.
const SIT_FORCED_END_ODDS: i32 = 2000;
/// `PandaSitGoal.stop` waits `nextInt(bound) + SIT_COOLDOWN_MIN_SECONDS` seconds before it sits
/// again: the bound is 50 for a lazy panda and 150 for any other.
const LAZY_SIT_COOLDOWN_RANGE_SECONDS: i32 = 50;
const SIT_COOLDOWN_RANGE_SECONDS: i32 = 150;
const SIT_COOLDOWN_MIN_SECONDS: i32 = 10;
const TICKS_PER_SECOND: i32 = 20;
/// `PandaSitGoal.canUse` looks for food on the ground within this many blocks of the panda's box.
const SIT_GROUND_ITEM_START_RANGE: f64 = 6.0;
/// `PandaSitGoal.start` walks to the first food within this many blocks of the panda's box.
const SIT_GROUND_ITEM_WALK_RANGE: f64 = 8.0;
/// Speed modifier with which `PandaSitGoal.start` walks to the food.
const SIT_GROUND_ITEM_SPEED: f64 = 1.2;
/// Server ticks the panda waits after a sulk before it sulks about missing bamboo again.
const UNHAPPY_COOLDOWN_TICKS: i32 = 600;
/// `Panda.BREED_TARGETING` range: the sulk looks at the nearest player within this many blocks.
const SULK_LOOK_RANGE: f64 = 8.0;

type PandaCheck = fn(&PandaEntity) -> bool;

/// A shared goal that only starts while a check on the panda passes, as vanilla's panda
/// subclasses that add a `canUse` condition. Continuing is the inner goal's decision alone.
pub struct PandaGoal<G> {
    inner: G,
    start_if: PandaCheck,
}

impl<G: Goal> PandaGoal<G> {
    /// Wraps `inner` so that it only starts while `start_if` holds for the panda.
    pub const fn new(inner: G, start_if: PandaCheck) -> Self {
        Self { inner, start_if }
    }
}

impl<G: Goal> Goal for PandaGoal<G> {
    /// Vanilla `canUse`: the panda check first, then the inner goal.
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        as_panda(mob).is_some_and(self.start_if) && self.inner.can_start(mob)
    }

    /// Vanilla `canContinueToUse`, left to the inner goal.
    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.inner.should_continue(mob)
    }

    /// Vanilla `start` of the inner goal.
    fn start(&mut self, mob: &dyn Mob) {
        self.inner.start(mob);
    }

    /// Vanilla `stop` of the inner goal.
    fn stop(&mut self, mob: &dyn Mob) {
        self.inner.stop(mob);
    }

    /// Vanilla `tick` of the inner goal.
    fn tick(&mut self, mob: &dyn Mob) {
        self.inner.tick(mob);
    }

    /// Vanilla `requiresUpdateEveryTick` of the inner goal.
    fn should_run_every_tick(&self) -> bool {
        self.inner.should_run_every_tick()
    }

    /// Vanilla's interruptible check of the inner goal.
    fn can_stop(&self) -> bool {
        self.inner.can_stop()
    }

    /// Vanilla `getFlags` of the inner goal.
    fn controls(&self) -> Controls {
        self.inner.controls()
    }

    /// The panda check only reads the panda itself, so the inner reach holds.
    fn tick_reach(&self, mob: &dyn Mob, running: bool) -> Option<GoalReach> {
        self.inner.tick_reach(mob, running)
    }
}

/// Vanilla `PandaBreedGoal`: `BreedGoal` that needs bamboo nearby and sulks when there is none.
pub struct PandaBreedGoal {
    inner: Box<BreedGoal>,
    unhappy_cooldown: i32,
    look_target: Arc<LookTargetSlot>,
}

impl PandaBreedGoal {
    /// Vanilla `PandaBreedGoal(panda, speedModifier)`; `look_target` is the slot of the panda's
    /// look goal, which the sulk points at the nearest player.
    #[must_use]
    pub fn new(speed: f64, look_target: Arc<LookTargetSlot>) -> Self {
        Self {
            inner: BreedGoal::new(speed),
            unhappy_cooldown: 0,
            look_target,
        }
    }

    /// `BREED_TARGETING`: the nearest player within 8 blocks that a non-combat check accepts.
    fn nearest_player(mob: &dyn Mob) -> Option<Arc<dyn EntityBase>> {
        let entity = mob.get_entity();
        let world = entity.world.load();
        let targeting =
            TargetPredicate::create_non_attackable().set_base_max_distance(SULK_LOOK_RANGE);
        world
            .get_nearest_player(entity.pos.load(), SULK_LOOK_RANGE, |player| {
                targeting.test(&world, Some(mob), player.as_ref())
            })
            .map(|player| player as Arc<dyn EntityBase>)
    }

    /// Vanilla `PandaBreedGoal.canFindBamboo`: a bamboo block within 7 blocks horizontally and 0
    /// to 2 blocks above the panda's feet block. Vanilla walks square rings from the centre
    /// outwards and stops at the first hit, which only matters for the order of the reads.
    fn can_find_bamboo(mob: &dyn Mob) -> bool {
        let entity = mob.get_entity();
        let world = entity.world.load();
        let panda_pos = entity.block_pos.load();
        (0..=BAMBOO_SEARCH_HEIGHT).any(|y| {
            (-BAMBOO_SEARCH_RADIUS..=BAMBOO_SEARCH_RADIUS).any(|x| {
                (-BAMBOO_SEARCH_RADIUS..=BAMBOO_SEARCH_RADIUS).any(|z| {
                    let pos =
                        BlockPos::new(panda_pos.0.x + x, panda_pos.0.y + y, panda_pos.0.z + z);
                    world.get_block(&pos) == &Block::BAMBOO
                })
            })
        })
    }
}

impl Goal for PandaBreedGoal {
    /// Vanilla `PandaBreedGoal.canUse`. Without bamboo the panda starts to sulk once per
    /// `UNHAPPY_COOLDOWN_TICKS` and never breeds.
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(panda) = as_panda(mob) else {
            return false;
        };
        if !self.inner.can_start(mob) || panda.get_unhappy_counter() != 0 {
            return false;
        }

        if Self::can_find_bamboo(mob) {
            return true;
        }
        let tick_count = mob.get_entity().tick_count.load(Relaxed);
        if self.unhappy_cooldown <= tick_count {
            panda.set_unhappy_counter(TOTAL_UNHAPPY_TIME);
            self.unhappy_cooldown = tick_count + UNHAPPY_COOLDOWN_TICKS;
            if !panda.mob_entity.is_no_ai() {
                self.look_target.set_target(Self::nearest_player(mob));
            }
        }
        false
    }

    /// Vanilla `canContinueToUse`, left to the breed goal.
    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.inner.should_continue(mob)
    }

    /// Vanilla `start` of the breed goal.
    fn start(&mut self, mob: &dyn Mob) {
        self.inner.start(mob);
    }

    /// Vanilla `stop` of the breed goal.
    fn stop(&mut self, mob: &dyn Mob) {
        self.inner.stop(mob);
    }

    /// Vanilla `tick` of the breed goal.
    fn tick(&mut self, mob: &dyn Mob) {
        self.inner.tick(mob);
    }

    /// Vanilla `requiresUpdateEveryTick` of the breed goal.
    fn should_run_every_tick(&self) -> bool {
        self.inner.should_run_every_tick()
    }

    /// Vanilla's interruptible check of the breed goal.
    fn can_stop(&self) -> bool {
        self.inner.can_stop()
    }

    /// Vanilla `getFlags` of the breed goal.
    fn controls(&self) -> Controls {
        self.inner.controls()
    }
}

/// Vanilla `Panda.doHurtTarget` as the swing of `PandaAttackGoal`: a panda that is not aggressive
/// remembers that it bit, whether or not the bite hurts.
pub fn bite(mob: &dyn Mob, target: &dyn EntityBase) {
    let Some(panda) = as_panda(mob) else {
        return;
    };
    if !panda.is_aggressive() {
        panda.did_bite.store(true, Relaxed);
    }
    panda.mob_entity.living_entity.swing_hand();
    if panda.mob_entity.try_attack(panda.get_entity(), target) {
        panda.on_attack(target);
    }
}

/// Vanilla `PandaAttackGoal`: a melee attack that only starts while the panda can act. Vanilla
/// leaves `canContinueToUse` alone, so a panda that sits down mid-attack keeps attacking.
pub fn attack_goal(speed: f64) -> PandaGoal<MeleeAttackGoal> {
    PandaGoal::new(
        MeleeAttackGoal::new(speed, true).performing(bite),
        PandaEntity::can_perform_action,
    )
}

/// Vanilla `PandaHurtByTargetGoal` with `setAlertOthers`: only aggressive pandas are alerted,
/// and a panda that was fed bamboo or has bitten lets go of its target.
pub fn hurt_by_target_goal() -> RevengeGoal {
    RevengeGoal::new(true)
        .alerting_others()
        .alerting_only(|mob| as_panda(mob).is_some_and(PandaEntity::is_aggressive))
        .continuing_while(|mob| {
            as_panda(mob).is_none_or(|panda| {
                !panda.got_bamboo.load(Relaxed) && !panda.did_bite.load(Relaxed)
            })
        })
}

/// Vanilla `PandaSitGoal`: a panda that holds an item sits down with it (and eats it, see
/// `Panda.handleEating`). A panda with an empty mouth starts the goal when there is something
/// to eat on the ground within 6 blocks and walks towards it; picking it up is
/// `Panda.pickUpItem`. Leaving the goal drops the item and keeps the panda from sitting for a
/// while.
#[derive(Default)]
pub struct PandaSitGoal {
    /// Vanilla `cooldown`: the `tick_count` before which the panda does not sit again.
    cooldown: i32,
}

impl PandaSitGoal {
    /// Vanilla `canContinueToUse` with `next_int(bound)` standing for `RandomSource.nextInt`: a
    /// panda leaves the water and the sit with small random chances per goal tick.
    pub(super) fn keeps_sitting(panda: &PandaEntity, mut next_int: impl FnMut(i32) -> i32) -> bool {
        if panda.get_entity().is_in_water() {
            return false;
        }
        if !panda.is_lazy() && next_int(to_goal_ticks(SIT_END_ODDS)) == 1 {
            return false;
        }
        next_int(to_goal_ticks(SIT_FORCED_END_ODDS)) != 1
    }
}

impl Goal for PandaSitGoal {
    /// Vanilla `canUse`: not in cooldown, an adult out of the water that can act, not sulking
    /// and either holding an item or with food on the ground nearby.
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(panda) = as_panda(mob) else {
            return false;
        };
        self.cooldown <= panda.get_entity().tick_count.load(Relaxed)
            && !panda.is_baby()
            && !panda.get_entity().is_in_water()
            && panda.can_perform_action()
            && panda.get_unhappy_counter() <= 0
            && (panda.is_holding_item()
                || !panda
                    .ground_items_to_eat(SIT_GROUND_ITEM_START_RANGE)
                    .is_empty())
    }

    /// Vanilla `canContinueToUse`.
    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        as_panda(mob).is_some_and(|panda| {
            let mut rng = panda.get_random();
            Self::keeps_sitting(panda, |bound| rng.random_range(0..bound))
        })
    }

    /// Vanilla `start`: sit down with the held item, or with an empty mouth walk to the first
    /// food on the ground. Either way the cooldown is cleared.
    fn start(&mut self, mob: &dyn Mob) {
        if let Some(panda) = as_panda(mob) {
            if panda.is_holding_item() {
                panda.try_to_sit();
            } else if let Some(item) = panda
                .ground_items_to_eat(SIT_GROUND_ITEM_WALK_RANGE)
                .first()
            {
                let pos = panda.get_entity().pos.load();
                let target = item.get_entity().pos.load();
                panda
                    .mob_entity
                    .steered()
                    .navigator()
                    .set_progress(NavigatorGoal::new(pos, target, SIT_GROUND_ITEM_SPEED));
            }
        }
        self.cooldown = 0;
    }

    /// Vanilla `tick`: a panda that stood up with an item in its mouth sits down again.
    fn tick(&mut self, mob: &dyn Mob) {
        if let Some(panda) = as_panda(mob)
            && !panda.is_sitting()
            && panda.is_holding_item()
        {
            panda.try_to_sit();
        }
    }

    /// Vanilla `stop`.
    fn stop(&mut self, mob: &dyn Mob) {
        let Some(panda) = as_panda(mob) else {
            return;
        };
        if panda.drop_main_hand_item() {
            let range = if panda.is_lazy() {
                LAZY_SIT_COOLDOWN_RANGE_SECONDS
            } else {
                SIT_COOLDOWN_RANGE_SECONDS
            };
            let wait_seconds = panda.get_random().random_range(0..range) + SIT_COOLDOWN_MIN_SECONDS;
            self.cooldown =
                panda.get_entity().tick_count.load(Relaxed) + wait_seconds * TICKS_PER_SECOND;
        }
        panda.set_sitting(false);
    }

    /// Vanilla `PandaSitGoal` flags: move only.
    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}
