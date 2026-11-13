use std::sync::Weak;

use super::{Goal, to_goal_ticks};
use crate::entity::EntityBase;
use crate::entity::mob::Mob;
use crate::entity::mob::enderman::EndermanEntity;
use crate::entity::predicate::EntityPredicate;
use crate::world::World;
use crate::world::game_event::GameEventContext;
use pumpkin_data::block_properties::is_air;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::{Block, BlockState, BlockStateId};
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;
use rand::RngExt;

pub struct PlaceBlockGoal {
    enderman: Weak<EndermanEntity>,
}

impl PlaceBlockGoal {
    #[must_use]
    pub const fn new(enderman: Weak<EndermanEntity>) -> Self {
        Self { enderman }
    }
}

impl Goal for PlaceBlockGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(enderman) = self.enderman.upgrade() else {
            return false;
        };
        if enderman.get_carried_block().is_none() {
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
        let Some(enderman) = self.enderman.upgrade() else {
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

        let world = entity.world.load_full();
        let target_pos = BlockPos::new(bx, by, bz);
        let target_state_id = world.get_block_state_id(&target_pos);
        let below_pos = target_pos.down();
        let (below_block, below_state) = world.get_block_and_state(&below_pos);
        let Some(carried) = enderman.get_carried_block() else {
            return;
        };
        let carried = world.update_from_neighbor_shapes(carried, &target_pos);
        if !self.can_place_block(
            &world,
            &target_pos,
            carried,
            target_state_id,
            below_block,
            below_state,
        ) {
            return;
        }

        world.set_block_state(&target_pos, carried, BlockFlags::NOTIFY_ALL);
        let source: &dyn EntityBase = enderman.as_ref();
        world.emit_game_event_with_context(
            GameEvent::BlockPlace.name(),
            target_pos.to_f64(),
            &GameEventContext {
                source_entity: Some(source),
            },
        );
        enderman.set_carried_block(None);
    }
}

impl PlaceBlockGoal {
    fn can_place_block(
        &self,
        world: &World,
        pos: &BlockPos,
        carried: BlockStateId,
        target_state_id: BlockStateId,
        below_block: &Block,
        below_state: &BlockState,
    ) -> bool {
        if !is_air(target_state_id)
            || !below_state.is_solid()
            || !below_state.is_full_cube()
            || below_block == &Block::BEDROCK
        {
            return false;
        }
        let (block, state) = BlockState::from_id_with_block(carried);
        if !world.block_registry.can_place_at(
            None,
            Some(world),
            world,
            None,
            block,
            state,
            pos,
            None,
            None,
        ) {
            return false;
        }
        let Some(enderman) = self.enderman.upgrade() else {
            return false;
        };
        let enderman_id = enderman.mob_entity.living_entity.entity.entity_id;
        !world
            .get_all_at_box(&BoundingBox::from_block(pos))
            .iter()
            .any(|other| {
                other.get_entity().entity_id != enderman_id
                    && EntityPredicate::ExceptSpectator.test(other.get_entity())
            })
    }
}
