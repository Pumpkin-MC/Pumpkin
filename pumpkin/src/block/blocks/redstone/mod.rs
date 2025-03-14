use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::BlockDirection;

use crate::world::World;

pub(crate) mod observer;
pub(crate) mod redstone_block;
pub(crate) mod redstone_lamp;
pub(crate) mod redstone_wire;

pub async fn update_wire_neighbors(world: &World, pos: BlockPos) {
    for direction in &BlockDirection::all() {
        let neighbor_pos = pos.offset(direction.to_offset());
        let block = world.get_block(&neighbor_pos).await.unwrap();
        world
            .block_registry
            .on_neighbor_update(world, &block, &neighbor_pos, &block, true)
            .await;
        for n_direction in &BlockDirection::all() {
            let n_neighbor_pos = neighbor_pos.offset(n_direction.to_offset());
            let block = world.get_block(&n_neighbor_pos).await.unwrap();
            world
                .block_registry
                .on_neighbor_update(world, &block, &n_neighbor_pos, &block, true)
                .await;
        }
    }
}
