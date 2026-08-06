use std::pin::Pin;
use std::sync::Arc;

use super::move_to_target_pos::{MoveToTargetPos, MoveToTargetPosGoal};
use super::{Controls, Goal, GoalFuture, ParentHandle, to_goal_ticks};
use crate::entity::Entity;
use crate::entity::item::ItemEntity;
use crate::entity::mob::Mob;
use crate::world::World;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::math::position::BlockPos;
use rand::RngExt;
use std::sync::atomic::Ordering;

/// Ticks the digging animation runs for before a seed pops out. Vanilla's `Digging`
/// brain behavior runs for a random 160-180 ticks; we use a fixed midpoint since this
/// is a simplified `Goal`-based approximation, not a port of the Brain/Activity system.
const DIG_DURATION: i32 = 170;
/// How many ticks before the end of the dig the seed item actually drops. Vanilla drops
/// the seed from a fixed tick recorded when digging starts (`DATA_DROP_SEED_AT_TICK`);
/// we approximate it as "near the end of the animation".
const DROP_AT_TICKS_REMAINING: i32 = 10;

/// Simplified, `Goal`-based approximation of the Sniffer's dig-for-seeds behavior.
///
/// Vanilla implements this via Brain/Behavior machinery, not a `Goal`.
/// See:
/// `net/minecraft/world/entity/animal/sniffer/SnifferAi.java` (`Digging`/`FinishedDigging`
/// behaviors driving `Sniffer.State.DIGGING`) together with `Sniffer.calculateDigPosition`,
/// `Sniffer.canDig`, and `Sniffer.dropSeed` in
/// `net/minecraft/world/entity/animal/sniffer/Sniffer.java`. Per task instructions this
/// ports a reasonable approximation of the core "walk to a diggable block, dig, drop a
/// seed item" mechanic onto the existing `Goal`/`MoveToTargetPosGoal` machinery instead of
/// porting the full Brain system.
///
/// Simplifications versus vanilla:
/// - No explored-position memory (`getExploredPositions`); the sniffer may redig the same
///   spot.
/// - Seed choice is a flat 50/50 between Torchflower Seeds and Pitcher Pod, matching the
///   two-entry `sniffer_digging` loot table pool, instead of going through the loot table
///   system.
/// - No "in love"/breeding or panic/temptation state checks (those subsystems aren't fully
///   wired for Sniffer in Pumpkin yet).
pub struct SnifferDigGoal {
    move_to_target_pos_goal: MoveToTargetPosGoal<Self>,
    digging_ticks: i32,
}

impl SnifferDigGoal {
    #[must_use]
    pub fn new(speed: f64) -> Box<Self> {
        let mut this = Box::new(Self {
            move_to_target_pos_goal: MoveToTargetPosGoal::new(ParentHandle::none(), speed, 10, 2),
            digging_ticks: 0,
        });

        // SAFETY: `this` heap allocation address is pinned in Box and outlives `ParentHandle` references.
        this.move_to_target_pos_goal.move_to_target_pos = unsafe { ParentHandle::new(&this) };

        this
    }
}

impl Goal for SnifferDigGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let entity = &mob.get_mob_entity().living_entity.entity;
            if !entity.on_ground.load(Ordering::Relaxed)
                || entity.touching_water.load(Ordering::SeqCst)
            {
                return false;
            }

            if mob.get_random().random_range(0..to_goal_ticks(700)) != 0 {
                return false;
            }

            self.move_to_target_pos_goal.can_start(mob).await
        })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            self.digging_ticks > 0 || self.move_to_target_pos_goal.should_continue(mob).await
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            self.move_to_target_pos_goal.start(mob).await;
            self.digging_ticks = 0;
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            self.move_to_target_pos_goal.stop(mob).await;
            self.digging_ticks = 0;
            if let Some(sniffer) = mob
                .cast_any()
                .downcast_ref::<crate::entity::passive::sniffer::SnifferEntity>()
            {
                sniffer.transition_to(crate::entity::passive::sniffer::SnifferState::Idling);
            }
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            if self.digging_ticks == 0 {
                self.move_to_target_pos_goal.tick(mob).await;
            }

            if !self.move_to_target_pos_goal.reached {
                return;
            }

            let entity = &mob.get_mob_entity().living_entity.entity;
            let world = entity.world.load_full();
            let dig_pos = self.move_to_target_pos_goal.target_pos;
            let head_pos = dig_pos.up();

            if self.digging_ticks == 0 {
                self.digging_ticks = DIG_DURATION;
                let pos_f64 = head_pos.to_f64();
                world.play_sound_raw(
                    Sound::EntitySnifferDigging as u16,
                    SoundCategory::Neutral,
                    &pos_f64,
                    1.0,
                    1.0,
                );
                if let Some(sniffer) = mob
                    .cast_any()
                    .downcast_ref::<crate::entity::passive::sniffer::SnifferEntity>()
                {
                    sniffer.transition_to(crate::entity::passive::sniffer::SnifferState::Digging);
                }
                return;
            }

            self.digging_ticks -= 1;

            if self.digging_ticks == DROP_AT_TICKS_REMAINING {
                let seed = if mob.get_random().random_range(0..2) == 0 {
                    &Item::TORCHFLOWER_SEEDS
                } else {
                    &Item::PITCHER_POD
                };

                let item_entity = ItemEntity::new(
                    Entity::new(world.clone(), head_pos.to_f64(), &EntityType::ITEM),
                    ItemStack::new(1, seed),
                );
                world.spawn_entity(Arc::new(item_entity)).await;

                let pos_f64 = head_pos.to_f64();
                world.play_sound_raw(
                    Sound::EntitySnifferDropSeed as u16,
                    SoundCategory::Neutral,
                    &pos_f64,
                    1.0,
                    1.0,
                );
            }

            if self.digging_ticks <= 0 {
                self.digging_ticks = 0;
                self.move_to_target_pos_goal.cooldown = to_goal_ticks(200);
                if let Some(sniffer) = mob
                    .cast_any()
                    .downcast_ref::<crate::entity::passive::sniffer::SnifferEntity>()
                {
                    sniffer.transition_to(crate::entity::passive::sniffer::SnifferState::Idling);
                }
            }
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        self.move_to_target_pos_goal.controls()
    }
}

impl MoveToTargetPos for SnifferDigGoal {
    fn is_target_pos<'a>(
        &'a self,
        world: Arc<World>,
        block_pos: BlockPos,
    ) -> Pin<Box<dyn Future<Output = bool> + Send + 'a>> {
        Box::pin(async move {
            let block = world.get_block(&block_pos);
            if !block.has_tag(&tag::Block::MINECRAFT_SNIFFER_DIGGABLE_BLOCK) {
                return false;
            }

            world.get_block_state(&block_pos.up()).is_air()
                && world.get_block_state(&block_pos.up_height(2)).is_air()
        })
    }
}
