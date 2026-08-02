use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use super::move_to_target_pos::{MoveToTargetPos, MoveToTargetPosGoal};
use super::{Controls, Goal, GoalFuture, ParentHandle};
use crate::entity::mob::Mob;
use crate::world::World;
use pumpkin_data::Block;
use pumpkin_data::block_properties::{BlockProperties, WheatLikeProperties};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::world::WorldEvent;
use pumpkin_protocol::java::client::play::CWorldEvent;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;
use rand::RngExt;

/// Port of vanilla's `Rabbit.RaidGardenGoal`.
///
/// Makes rabbits seek out fully grown carrots on farmland and nibble the crop down one
/// growth stage at a time, fully removing it once its age reaches 0.
///
/// Vanilla source is `net/minecraft/world/entity/animal/rabbit/Rabbit.java` (inner class
/// `RaidGardenGoal`, lines ~592-663 of the 26.2 decompile). Note the task brief referred
/// to this behavior by the name of an older/different goal class, `RemoveBlockGoal`
/// (used today by zombies raiding turtle eggs, already ported as `StepAndDestroyBlockGoal`
/// / `destroy_egg.rs`); current vanilla implements the rabbit's carrot-raiding behavior as
/// its own bespoke goal instead, so this ports `RaidGardenGoal` directly.
pub struct RaidGardenGoal {
    move_to_target_pos_goal: MoveToTargetPosGoal<Self>,
    wants_to_raid: bool,
    can_raid: AtomicBool,
    /// Mirrors vanilla's `Rabbit.moreCarrotTicks`: while positive, the rabbit isn't hungry
    /// enough to raid a garden again. Vanilla decays this by a random 0..3 every tick in
    /// `aiStep` and persists it to NBT (`MoreCarrotTicks`). This port only decays it while
    /// this goal itself is polled (roughly once per cooldown interval) and does not persist
    /// it across saves - an intentional, reported scope reduction since `RabbitEntity`
    /// doesn't currently have NBT-backed extra state and adding it is out of scope for this
    /// goal port.
    more_carrot_ticks: AtomicI32,
}

impl RaidGardenGoal {
    #[must_use]
    pub fn new(speed: f64) -> Box<Self> {
        let mut this = Box::new(Self {
            move_to_target_pos_goal: MoveToTargetPosGoal::new(ParentHandle::none(), speed, 16, 1),
            wants_to_raid: false,
            can_raid: AtomicBool::new(false),
            more_carrot_ticks: AtomicI32::new(0),
        });

        this.move_to_target_pos_goal.move_to_target_pos = unsafe { ParentHandle::new(&this) };

        this
    }

    fn wants_more_food(&self, mob: &dyn Mob) -> bool {
        let ticks = self.more_carrot_ticks.load(Ordering::Relaxed);
        // Approximate vanilla's per-tick `moreCarrotTicks -= random.nextInt(3)` decay by
        // applying it once per cooldown poll instead of every server tick.
        if ticks > 0 {
            let decay = mob.get_random().random_range(0..3);
            self.more_carrot_ticks
                .store((ticks - decay).max(0), Ordering::Relaxed);
        }
        self.more_carrot_ticks.load(Ordering::Relaxed) <= 0
    }
}

impl Goal for RaidGardenGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async {
            let world = mob.get_entity().world.load();
            if !world.level_info.load().game_rules.mob_griefing {
                return false;
            }

            if self.move_to_target_pos_goal.cooldown <= 0 {
                self.can_raid.store(false, Ordering::Relaxed);
                self.wants_to_raid = self.wants_more_food(mob);
            }

            self.move_to_target_pos_goal.can_start(mob).await
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
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

                    let packet = CWorldEvent::new(
                        WorldEvent::ParticlesDestroyBlock as i32,
                        crops_pos,
                        i32::from(state_id.as_u16()),
                        false,
                    );
                    world.broadcast_to_chunk(crops_pos.chunk_position(), &packet);
                }

                self.more_carrot_ticks.store(40, Ordering::Relaxed);
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
