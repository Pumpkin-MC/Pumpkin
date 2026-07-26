use super::BlockRegistry;
use crate::block::{
    EmitsRedstonePowerArgs, GetInsideCollisionShapeArgs, GetRedstonePowerArgs,
    GetStateForNeighborUpdateArgs, OnNeighborUpdateArgs, PrepareArgs,
};
use crate::world::World;
use pumpkin_data::BlockStateId;
use pumpkin_data::block_rotation::{Mirror, Rotation};
use pumpkin_data::{Block, BlockDirection, BlockState};
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;
use std::sync::Arc;

impl BlockRegistry {
    /// Updates state of all neighbors of the block
    pub async fn post_process_state(
        &self,
        world: &Arc<World>,
        position: &BlockPos,
        block: &Block,
        flags: BlockFlags,
    ) {
        let state_id = world.get_block_state_id(position);
        for direction in BlockDirection::all() {
            let neighbor_pos = position.offset(direction.to_offset());
            let neighbor_state_id = world.get_block_state_id(&neighbor_pos);
            let pumpkin_block = self.get_pumpkin_block(block.id);
            if let Some(pumpkin_block) = pumpkin_block {
                let new_state = pumpkin_block
                    .get_state_for_neighbor_update(GetStateForNeighborUpdateArgs {
                        world,
                        block,
                        state_id,
                        position,
                        direction: direction.opposite(),
                        neighbor_position: &neighbor_pos,
                        neighbor_state_id,
                    })
                    .await;
                world.set_block_state(&neighbor_pos, new_state, flags).await;
            }
        }
    }

    pub async fn prepare(
        &self,
        world: &Arc<World>,
        position: &BlockPos,
        block: &Block,
        state_id: BlockStateId,
        flags: BlockFlags,
    ) {
        let pumpkin_block = self.get_pumpkin_block(block.id);
        if let Some(pumpkin_block) = pumpkin_block {
            pumpkin_block
                .prepare(PrepareArgs {
                    world,
                    block,
                    state_id,
                    position,
                    flags,
                })
                .await;
        }
    }

    #[expect(clippy::too_many_arguments)]
    pub async fn get_state_for_neighbor_update(
        &self,
        world: &Arc<World>,
        block: &Block,
        state_id: BlockStateId,
        position: &BlockPos,
        direction: BlockDirection,
        neighbor_location: &BlockPos,
        neighbor_state_id: BlockStateId,
    ) -> BlockStateId {
        let pumpkin_block = self.get_pumpkin_block(block.id);
        if let Some(pumpkin_block) = pumpkin_block {
            return pumpkin_block
                .get_state_for_neighbor_update(GetStateForNeighborUpdateArgs {
                    world,
                    block,
                    state_id,
                    position,
                    direction,
                    neighbor_position: neighbor_location,
                    neighbor_state_id,
                })
                .await;
        }
        state_id
    }

    pub async fn update_neighbors(
        &self,
        world: &Arc<World>,
        position: &BlockPos,
        _block: &Block,
        flags: BlockFlags,
    ) {
        for direction in BlockDirection::abstract_block_update_order() {
            let pos = position.offset(direction.to_offset());

            Box::pin(world.replace_with_state_for_neighbor_update(
                &pos,
                direction.opposite(),
                flags,
            ))
            .await;
        }
    }

    pub async fn on_neighbor_update(
        &self,
        world: &Arc<World>,
        block: &Block,
        position: &BlockPos,
        source_block: &Block,
        notify: bool,
    ) {
        let pumpkin_block = self.get_pumpkin_block(block.id);
        if let Some(pumpkin_block) = pumpkin_block {
            pumpkin_block
                .on_neighbor_update(OnNeighborUpdateArgs {
                    world,
                    block,
                    position,
                    source_block,
                    notify,
                })
                .await;
        }
    }

    pub async fn emits_redstone_power(
        &self,
        block: &Block,
        state: &BlockState,
        direction: BlockDirection,
    ) -> bool {
        let pumpkin_block = self.get_pumpkin_block(block.id);
        if let Some(pumpkin_block) = pumpkin_block {
            return pumpkin_block
                .emits_redstone_power(EmitsRedstonePowerArgs {
                    block,
                    state,
                    direction,
                })
                .await;
        }
        false
    }

    pub async fn get_weak_redstone_power(
        &self,
        block: &Block,
        world: &World,
        position: &BlockPos,
        state: &BlockState,
        direction: BlockDirection,
    ) -> u8 {
        let pumpkin_block = self.get_pumpkin_block(block.id);
        if let Some(pumpkin_block) = pumpkin_block {
            return pumpkin_block
                .get_weak_redstone_power(GetRedstonePowerArgs {
                    world,
                    block,
                    state,
                    position,
                    direction,
                })
                .await;
        }
        0
    }

    pub async fn get_strong_redstone_power(
        &self,
        block: &Block,
        world: &World,
        position: &BlockPos,
        state: &BlockState,
        direction: BlockDirection,
    ) -> u8 {
        let pumpkin_block = self.get_pumpkin_block(block.id);
        if let Some(pumpkin_block) = pumpkin_block {
            return pumpkin_block
                .get_strong_redstone_power(GetRedstonePowerArgs {
                    world,
                    block,
                    state,
                    position,
                    direction,
                })
                .await;
        }
        0
    }

    pub async fn get_inside_collision_shape(
        &self,
        block: &Block,
        world: &World,
        state: &BlockState,
        position: &BlockPos,
    ) -> BoundingBox {
        let pumpkin_block = self.get_pumpkin_block(block.id);
        if let Some(pumpkin_block) = pumpkin_block {
            return pumpkin_block
                .get_inside_collision_shape(GetInsideCollisionShapeArgs {
                    world,
                    block,
                    state,
                    position,
                })
                .await;
        }
        BoundingBox::full_block()
    }

    #[must_use]
    pub fn mirror(
        &self,
        block: &Block,
        state_id: BlockStateId,
        mirror: Mirror,
    ) -> &'static BlockState {
        self.get_pumpkin_block(block.id).map_or_else(
            || block.mirror(state_id, mirror),
            |pumpkin_block| pumpkin_block.mirror(block, state_id, mirror),
        )
    }

    #[must_use]
    pub fn rotate(
        &self,
        block: &Block,
        state_id: BlockStateId,
        rotation: Rotation,
    ) -> &'static BlockState {
        self.get_pumpkin_block(block.id).map_or_else(
            || block.rotate(state_id, rotation),
            |pumpkin_block| pumpkin_block.rotate(block, state_id, rotation),
        )
    }
}
