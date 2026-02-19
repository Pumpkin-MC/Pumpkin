use std::sync::Arc;

use super::{Goal, GoalFuture, to_goal_ticks};
use crate::entity::mob::Mob;
use crate::entity::mob::enderman::EndermanEntity;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use pumpkin_world::world::BlockFlags;
use rand::RngExt;

pub struct PickUpBlockGoal {
    enderman: Arc<EndermanEntity>,
}

impl PickUpBlockGoal {
    pub const fn new(enderman: Arc<EndermanEntity>) -> Self {
        Self { enderman }
    }
}

impl Goal for PickUpBlockGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            // Vanilla check order: carried_block → mob_griefing → random(toGoalTicks(20))
            if self.enderman.get_carried_block().is_some() {
                return false;
            }

            let entity = &mob.get_mob_entity().living_entity.entity;
            let world = entity.world.load();
            if !world.level_info.load().game_rules.mob_griefing {
                return false;
            }

            // Vanilla: nextInt(toGoalTicks(20)) == 0 → toGoalTicks(20) = 10
            if mob.get_random().random_range(0..to_goal_ticks(20)) != 0 {
                return false;
            }

            true
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let entity = &mob.get_mob_entity().living_entity.entity;
            let pos = entity.pos.load();

            // Vanilla: one random candidate per tick (not a loop)
            let (bx, by, bz) = {
                let mut rng = mob.get_random();
                (
                    pos.x.floor() as i32 + rng.random_range(-2..=2),
                    pos.y.floor() as i32 + rng.random_range(0..=2),
                    pos.z.floor() as i32 + rng.random_range(-2..=2),
                )
            };

            let world = entity.world.load();
            let target_pos = BlockPos::new(bx, by, bz);

            let (block, _state_id) = world.get_block_and_state_id(&target_pos).await;

            if !block.has_tag(&tag::Block::MINECRAFT_ENDERMAN_HOLDABLE) {
                return;
            }

            let enderman_block = entity.block_pos.load();
            let enderman_center = Vector3::new(
                enderman_block.0.x as f64 + 0.5,
                by as f64 + 0.5,
                enderman_block.0.z as f64 + 0.5,
            );
            let block_center = Vector3::new(bx as f64 + 0.5, by as f64 + 0.5, bz as f64 + 0.5);
            if let Some((hit_pos, _)) = world
                .raycast(enderman_center, block_center, async |block_pos, w| {
                    let state = w.get_block_state(block_pos).await;
                    state.is_solid()
                })
                .await
                && hit_pos != target_pos
            {
                return;
            }

            // Vanilla stores the block's DEFAULT state, not the in-world state
            let default_state_id = block.default_state.id;

            // Pick up the block: set air at position, carry the block
            // TODO: Emit game event (BLOCK_DESTROY) — needs game event system
            world
                .set_block_state(&target_pos, 0, BlockFlags::NOTIFY_ALL)
                .await;
            self.enderman
                .set_carried_block(Some(default_state_id))
                .await;
        })
    }

    fn should_continue<'a>(&'a self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async { false })
    }
}
