use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Weak};

use super::move_to_target_pos::{MoveToTargetPos, MoveToTargetPosGoal};
use super::{Controls, Goal, GoalFuture, ParentHandle};
use crate::entity::EntityBase;
use crate::entity::mob::Mob;
use crate::entity::passive::rabbit::RabbitEntity;
use crate::world::World;
use crate::world::game_event::{GameEventContext, emit_game_event};
use pumpkin_data::Block;
use pumpkin_data::block_properties::{BlockProperties, WheatLikeProperties};
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::world::WorldEvent;
use pumpkin_protocol::java::client::play::CWorldEvent;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;

/// Port of vanilla's `Rabbit.RaidGardenGoal`.
///
/// Makes rabbits seek out fully grown carrots on farmland and nibble the crop down one
/// growth stage at a time, fully removing it once its age reaches 0.
///
/// Vanilla source is `net/minecraft/world/entity/animal/Rabbit.java` (inner class
/// `RaidGardenGoal`, lines 518-596 of the Mojang-named 1.21.4 decompile that every line
/// citation in this file refers to). Note the original task brief referred to this
/// behavior by the name of an older/different goal class, `RemoveBlockGoal`
/// (used today by zombies raiding turtle eggs, already ported as `StepAndDestroyBlockGoal`
/// / `destroy_egg.rs`); current vanilla implements the rabbit's carrot-raiding behavior as
/// its own bespoke goal instead, so this ports `RaidGardenGoal` directly.
pub struct RaidGardenGoal {
    move_to_target_pos_goal: MoveToTargetPosGoal<Self>,
    wants_to_raid: bool,
    can_raid: AtomicBool,
    /// Owner, for vanilla's `this.rabbit.wantsMoreFood()` (line 536) and
    /// `this.rabbit.moreCarrotTicks = 40` (line 575). The counter itself lives on
    /// `RabbitEntity`, not here, because vanilla decays it every tick in
    /// `customServerAiStep` (lines 182-187) and persists it as `MoreCarrotTicks` NBT
    /// (lines 275/282) - neither of which a goal can express.
    rabbit: Weak<RabbitEntity>,
}

impl RaidGardenGoal {
    #[must_use]
    pub fn new(speed: f64, rabbit: Weak<RabbitEntity>) -> Box<Self> {
        let mut this = Box::new(Self {
            move_to_target_pos_goal: MoveToTargetPosGoal::new(ParentHandle::none(), speed, 16, 1),
            wants_to_raid: false,
            can_raid: AtomicBool::new(false),
            rabbit,
        });

        // SAFETY: `this` heap allocation address is pinned in Box and outlives `ParentHandle` references.
        this.move_to_target_pos_goal.move_to_target_pos = unsafe { ParentHandle::new(&this) };

        this
    }
}

impl Goal for RaidGardenGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            // Vanilla `canUse` lines 530-537: the mobGriefing check is *inside* the
            // `nextStartTick <= 0` block, so once the goal is on cooldown it is not consulted
            // at all. Hoisting it to the top of `can_start` would make the goal re-evaluate
            // the game rule on every poll, a different (if benign-looking) gate.
            if self.move_to_target_pos_goal.cooldown <= 0 {
                let world = mob.get_entity().world.load();
                if !world.level_info.load().game_rules.mob_griefing {
                    return false;
                }

                self.can_raid.store(false, Ordering::Relaxed);
                self.wants_to_raid = self.rabbit.upgrade().is_some_and(|r| r.wants_more_food());
            }

            self.move_to_target_pos_goal.can_start(mob).await
        })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            self.can_raid.load(Ordering::Relaxed)
                && self.move_to_target_pos_goal.should_continue(mob).await
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async { self.move_to_target_pos_goal.start(mob).await })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async { self.move_to_target_pos_goal.stop(mob).await })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            self.move_to_target_pos_goal.tick(mob).await;

            let target_pos = self.move_to_target_pos_goal.target_pos;
            let target_f64 = target_pos.up().to_f64();
            mob.get_mob_entity().look_control.lock().unwrap().look_at(
                mob,
                target_f64.x + 0.5,
                target_f64.y + 1.0,
                target_f64.z + 0.5,
            );

            if !self.move_to_target_pos_goal.reached {
                return;
            }

            let world = mob.get_entity().world.load_full();
            let crops_pos = target_pos.up();
            let (block, state_id) = world.get_block_and_state_id(&crops_pos);

            if self.can_raid.load(Ordering::Relaxed) && block == &Block::CARROTS {
                let props = WheatLikeProperties::from_state_id(state_id, block);
                if props.age == 0 {
                    world
                        .break_block(&crops_pos, None, BlockFlags::NOTIFY_ALL)
                        .await;
                } else {
                    let mut new_props = props;
                    new_props.age -= 1;
                    let new_state_id = new_props.to_state_id(block);
                    world
                        .set_block_state(&crops_pos, new_state_id, BlockFlags::NOTIFY_ALL)
                        .await;

                    // Vanilla line 571: the age-decrement path emits BLOCK_CHANGE explicitly
                    // (the age-0 path goes through `destroyBlock`, which emits it internally).
                    emit_game_event(
                        &world,
                        GameEvent::BlockChange,
                        crops_pos.to_f64(),
                        self.rabbit
                            .upgrade()
                            .map_or_else(GameEventContext::none, |r| {
                                GameEventContext::of_entity(r as Arc<dyn EntityBase>)
                            }),
                    )
                    .await;

                    let packet = CWorldEvent::new(
                        WorldEvent::ParticlesDestroyBlock as i32,
                        crops_pos,
                        i32::from(state_id.as_u16()),
                        false,
                    );
                    world.broadcast_to_chunk(crops_pos.chunk_position(), &packet);
                }

                if let Some(rabbit) = self.rabbit.upgrade() {
                    rabbit.set_more_carrot_delay();
                }
            }

            self.can_raid.store(false, Ordering::Relaxed);
            self.move_to_target_pos_goal.cooldown = 10;
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        self.move_to_target_pos_goal.controls()
    }
}

impl MoveToTargetPos for RaidGardenGoal {
    fn is_target_pos<'a>(
        &'a self,
        world: Arc<World>,
        block_pos: BlockPos,
    ) -> Pin<Box<dyn Future<Output = bool> + Send + 'a>> {
        Box::pin(async move {
            let block = world.get_block(&block_pos);
            if !block.has_tag(&tag::Block::MINECRAFT_SUPPORTS_CROPS)
                || !self.wants_to_raid
                || self.can_raid.load(Ordering::Relaxed)
            {
                return false;
            }

            let above_pos = block_pos.up();
            let (above_block, above_state_id) = world.get_block_and_state_id(&above_pos);
            if above_block == &Block::CARROTS {
                let props = WheatLikeProperties::from_state_id(above_state_id, above_block);
                if props.age == 7 {
                    self.can_raid.store(true, Ordering::Relaxed);
                    return true;
                }
            }

            false
        })
    }
}
