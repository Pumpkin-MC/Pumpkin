use async_trait::async_trait;
use pumpkin_data::block::Block;
use pumpkin_data::block::CardinalDirection;
use pumpkin_data::block::FenceBlockProps;
use pumpkin_data::block::{BlockProperties, Boolean};
use pumpkin_data::tag::Tagable;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::BlockDirection;

use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::block::registry::BlockRegistry;
use crate::server::Server;
use crate::world::World;

fn connects_to(from: &Block, to: &Block) -> bool {
    if from.id == to.id {
        return true;
    }

    // If the block is not a wooden fence, it cannot connect to a wooden fence
    if !from.is_tagged_with("c:fences/wooden").unwrap() {
        return false;
    }

    to.is_tagged_with("c:fences/wooden").unwrap() || to.is_tagged_with("c:fence_gates").unwrap()
}

/// This returns an index and not a state id making it so all fences can use the same state calculation function
pub async fn fence_state(world: &World, block: &Block, block_pos: &BlockPos) -> u16 {
    let mut block_properties = FenceBlockProps::default(block);

    for direction in BlockDirection::horizontal() {
        let offset = block_pos.offset(direction.to_offset());
        let other_block = world.get_block(&offset).await.unwrap_or(Block::AIR);

        if connects_to(block, &other_block) {
            match direction {
                BlockDirection::North => block_properties.north = Boolean::True,
                BlockDirection::South => block_properties.south = Boolean::True,
                BlockDirection::West => block_properties.west = Boolean::True,
                BlockDirection::East => block_properties.east = Boolean::True,
                _ => {}
            }
        }
    }

    block_properties.to_state_id(block)
}

macro_rules! define_fence_block {
    ($block_name:ident, $block:expr) => {
        pub struct $block_name;
        impl BlockMetadata for $block_name {
            const NAMESPACE: &'static str = "minecraft";
            const ID: &'static str = $block.name;
        }

        #[async_trait]
        impl PumpkinBlock for $block_name {
            async fn on_place(
                &self,
                _server: &Server,
                world: &World,
                block: &Block,
                _face: &BlockDirection,
                block_pos: &BlockPos,
                _use_item_on: &SUseItemOn,
                _player_direction: &CardinalDirection,
                _other: bool,
            ) -> u16 {
                fence_state(world, block, block_pos).await
            }

            async fn on_neighbor_update(
                &self,
                _server: &Server,
                world: &World,
                block: &Block,
                block_pos: &BlockPos,
                _source_face: &BlockDirection,
                _source_block_pos: &BlockPos,
            ) {
                world
                    .set_block_state(block_pos, fence_state(world, block, block_pos).await)
                    .await;
            }
        }
    };
}

define_fence_block!(OakFenceBlock, Block::OAK_FENCE);
define_fence_block!(DarkOakFenceBlock, Block::DARK_OAK_FENCE);
define_fence_block!(SpruceFenceBlock, Block::SPRUCE_FENCE);
define_fence_block!(BirchFenceBlock, Block::BIRCH_FENCE);
define_fence_block!(JungleFenceBlock, Block::JUNGLE_FENCE);
define_fence_block!(AcaciaFenceBlock, Block::ACACIA_FENCE);
define_fence_block!(PaleOakFenceBlock, Block::PALE_OAK_FENCE);
define_fence_block!(CherryFenceBlock, Block::CHERRY_FENCE);
define_fence_block!(MangroveFenceBlock, Block::MANGROVE_FENCE);
define_fence_block!(CrimsonFenceBlock, Block::CRIMSON_FENCE);
define_fence_block!(WarpedFenceBlock, Block::WARPED_FENCE);
define_fence_block!(BambooFenceBlock, Block::BAMBOO_FENCE);
define_fence_block!(NetherBrickFenceBlock, Block::NETHER_BRICK_FENCE);

pub fn register_fence_blocks(manager: &mut BlockRegistry) {
    manager.register(OakFenceBlock);
    manager.register(DarkOakFenceBlock);
    manager.register(SpruceFenceBlock);
    manager.register(BirchFenceBlock);
    manager.register(JungleFenceBlock);
    manager.register(AcaciaFenceBlock);
    manager.register(PaleOakFenceBlock);
    manager.register(CherryFenceBlock);
    manager.register(MangroveFenceBlock);
    manager.register(CrimsonFenceBlock);
    manager.register(WarpedFenceBlock);
    manager.register(BambooFenceBlock);
    manager.register(NetherBrickFenceBlock);
}
