use std::sync::Arc;

use super::{Goal, GoalFuture, to_goal_ticks};
use crate::entity::mob::Mob;
use crate::entity::mob::enderman::EndermanEntity;
use pumpkin_data::Block;
use pumpkin_data::block_properties::is_air;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;
use rand::RngExt;

pub struct PlaceBlockGoal {
    enderman: Arc<EndermanEntity>,
}

impl PlaceBlockGoal {
    pub const fn new(enderman: Arc<EndermanEntity>) -> Self {
        Self { enderman }
    }
}

impl Goal for PlaceBlockGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if self.enderman.get_carried_block().is_none() {
                return false;
            }

            let entity = &mob.get_mob_entity().living_entity.entity;
            let world = entity.world.load();
            if !world.level_info.load().game_rules.mob_griefing {
                return false;
            }

            if mob.get_random().random_range(0..to_goal_ticks(2000)) != 0 {
                return false;
            }

            true
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(block_state_id) = self.enderman.get_carried_block() else {
                return;
            };

            let entity = &mob.get_mob_entity().living_entity.entity;
            let pos = entity.pos.load();

            let (bx, by, bz) = {
                let mut rng = mob.get_random();
                (
                    pos.x.floor() as i32 + rng.random_range(-1..=1),
                    pos.y.floor() as i32 + rng.random_range(0..=2),
                    pos.z.floor() as i32 + rng.random_range(-1..=1),
                )
            };

            let world = entity.world.load();
            let target_pos = BlockPos::new(bx, by, bz);

            let state_id = world.get_block_state_id(&target_pos).await;
            if !is_air(state_id) {
                return;
            }

            let below_pos = BlockPos::new(bx, by - 1, bz);
            let below_block = world.get_block(&below_pos).await;
            let below_state = world.get_block_state(&below_pos).await;
            if !below_state.is_solid()
                || !below_state.is_full_cube()
                || below_block == &Block::BEDROCK
            {
                return;
            }

            // TODO: Validate canPlaceAt and check entity collisions at target position
            world
                .set_block_state(&target_pos, block_state_id, BlockFlags::NOTIFY_ALL)
                .await;
            self.enderman.set_carried_block(None).await;
        })
    }

    fn should_continue<'a>(&'a self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async { false })
    }
}
