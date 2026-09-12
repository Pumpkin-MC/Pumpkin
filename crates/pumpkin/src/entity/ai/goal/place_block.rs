use std::sync::Arc;

use super::{Goal, to_goal_ticks};
use crate::entity::mob::Mob;
use crate::entity::mob::enderman::EndermanEntity;
use pumpkin_data::Block;
use pumpkin_data::block_properties::is_air;
use pumpkin_util::math::boundingbox::BoundingBox;
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
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
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
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(block_state_id) = self.enderman.get_carried_block() else {
            return;
        };

        let entity = &mob.get_mob_entity().living_entity.entity;
        let pos = entity.pos.load();

        let (bx, by, bz) = {
            let mut rng = mob.get_random();
            (
                (pos.x - 1.0 + rng.random::<f64>() * 2.0).floor() as i32,
                (pos.y + rng.random::<f64>() * 2.0).floor() as i32,
                (pos.z - 1.0 + rng.random::<f64>() * 2.0).floor() as i32,
            )
        };

        let world = entity.world.load();
        let target_pos = BlockPos::new(bx, by, bz);

        let state_id = world.get_block_state_id(&target_pos);
        if !is_air(state_id) {
            return;
        }

        let below_pos = BlockPos::new(bx, by - 1, bz);
        let (below_block, below_state) = world.get_block_and_state(&below_pos);
        if !below_state.is_solid() || !below_state.is_full_cube() || below_block == &Block::BEDROCK
        {
            return;
        }

        // Never place into an occupied block.
        let world_clone = entity.world.load_full();
        if !world_clone
            .get_entities_at_box(&BoundingBox::from_block(&target_pos))
            .is_empty()
        {
            return;
        }

        // TODO: also check the carried block can survive there.
        world_clone.set_block_state(&target_pos, block_state_id, BlockFlags::NOTIFY_ALL);
        self.enderman.set_carried_block(None);
    }
}
