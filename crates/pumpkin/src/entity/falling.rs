use pumpkin_data::Block;
use pumpkin_data::BlockState;
use pumpkin_data::BlockStateId;
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::world::WorldEvent;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;
use std::sync::{Arc, atomic::Ordering};

use crate::{
    block::blocks::falling::FallingBlock,
    entity::{Entity, EntityBase, living::LivingEntity},
    server::Server,
    world::World,
};

/// Vanilla only lets a falling block settle where it can replace whatever is already there;
/// anything else (torches, slabs, stairs, ...) makes it drop as an item instead.
const fn can_replace_landing_block(state: &BlockState) -> bool {
    state.replaceable()
}

pub struct FallingEntity {
    entity: Entity,
    block_state_id: BlockStateId,
    /// When set, landing destroys the block instead of placing it, and nothing is dropped.
    /// Mirrors vanilla's `FallingBlockEntity.cancelDrop`.
    cancel_drop: bool,
}

impl FallingEntity {
    pub const fn new(entity: Entity, block_state_id: BlockStateId) -> Self {
        Self {
            entity,
            block_state_id,
            cancel_drop: false,
        }
    }

    /// Replaced the current Block and Spawns a new Falling one (synchronous)
    pub fn replace_spawn(world: &Arc<World>, position: BlockPos, block_state: BlockStateId) {
        Self::replace_spawn_with(world, position, block_state, false);
    }

    /// As [`Self::replace_spawn`], but breaking on landing rather than placing the block when
    /// `cancel_drop` is set, the way vanilla's `disableDrop` behaves.
    pub fn replace_spawn_with(
        world: &Arc<World>,
        position: BlockPos,
        block_state: BlockStateId,
        cancel_drop: bool,
    ) {
        // Replace the original block, TODO: use fluid state
        world.set_block_state(
            &position,
            Block::AIR.default_state.id,
            BlockFlags::NOTIFY_ALL,
        );

        let position = position.0.to_f64().add_raw(0.5, 0.0, 0.5);
        let entity = Entity::new(world.clone(), position, &EntityType::FALLING_BLOCK);
        entity
            .data
            .store(i32::from(block_state.as_u16()), Ordering::Relaxed);
        let mut falling = Self::new(entity, block_state);
        falling.cancel_drop = cancel_drop;
        world.spawn_entity_non_save(Arc::new(falling));
    }

    /// Vanilla's `FallingBlockEntity` landing branch: break without dropping anything when
    /// the drop was cancelled, otherwise place the block back where it can settle and drop
    /// it as an item where it cannot.
    fn land(&self) {
        let world = self.entity.world.load();
        let position = self.entity.block_pos.load();
        let mut state_id = self.block_state_id;
        let block = Block::from_state_id(state_id);

        if self.cancel_drop {
            // Vanilla's `onBrokenAfterFall`: break particles and the destroy event, and the
            // block along with anything it was carrying is gone.
            world.sync_world_event(
                WorldEvent::ParticlesDestroyBlock,
                position,
                i32::from(state_id.as_u16()),
            );
            world.emit_game_event(GameEvent::BlockDestroy.name(), position.to_centered_f64());
            return;
        }

        if !can_replace_landing_block(world.get_block_state(&position)) {
            if world.level_info.load().game_rules.entity_drops
                && let Some(item) = Item::from_id(block.item_id)
            {
                world.drop_stack(&position, ItemStack::new(1, item));
            }
            return;
        }

        if block.has_tag(&tag::Block::MINECRAFT_CONCRETE_POWDERS)
            && FallingBlock::should_solidify(&**world, &position)
            && let Some(name) = block.name.strip_suffix("_powder")
            && let Some(concrete) = Block::from_name(name)
        {
            state_id = concrete.default_state.id;
        }

        world.set_block_state(&position, state_id, BlockFlags::NOTIFY_ALL);
    }
}

impl EntityBase for FallingEntity {
    fn tick(&self, caller: &dyn EntityBase, _server: &Server) {
        let entity = &self.entity;
        let mut velo = entity.velocity.load();
        velo.y -= self.get_gravity();

        entity.velocity.store(velo);

        entity.move_entity(caller, velo);
        entity.tick_block_collisions(caller);
        if entity.on_ground.load(Ordering::Relaxed) {
            entity.velocity.store(velo.multiply(0.7, -0.5, 0.7));
            self.land();
            self.entity.remove();
        }

        entity.velocity.store(velo.multiply(0.98, 0.98, 0.98));

        if entity.velocity_dirty.swap(false, Ordering::SeqCst) {
            entity.send_pos_rot();
            entity.send_velocity();
        }
    }

    fn init_data_tracker(&self) {
        self.entity.set_synced_data(
            pumpkin_data::tracked_data::falling_block::START_POS,
            self.entity.block_pos.load(),
        );
    }

    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }
    fn damage(&self, _caller: &dyn EntityBase, _amount: f32, _damage_type: DamageType) -> bool {
        false
    }

    fn get_gravity(&self) -> f64 {
        0.04
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::can_replace_landing_block;
    use pumpkin_data::Block;

    #[test]
    fn falling_blocks_do_not_replace_partial_blocks() {
        assert!(can_replace_landing_block(Block::AIR.default_state));
        assert!(!can_replace_landing_block(Block::TORCH.default_state));
        assert!(!can_replace_landing_block(Block::STONE_SLAB.default_state));
        assert!(!can_replace_landing_block(Block::OAK_STAIRS.default_state));
    }
}
