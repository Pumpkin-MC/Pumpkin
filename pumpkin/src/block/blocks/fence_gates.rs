use async_trait::async_trait;
use pumpkin_data::block::Block;
use pumpkin_data::block::BlockProperties;
use pumpkin_data::block::CardinalDirection;
use pumpkin_data::block::FenceGateBlockProps;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::BlockDirection;

use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::block::registry::BlockActionResult;
use crate::block::registry::BlockRegistry;
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;
use pumpkin_data::item::Item;

/// This returns an index and not a state id making it so all fences can use the same state calculation function
pub async fn toggle_fence_gate(world: &World, block_pos: &BlockPos) -> u16 {
    let (block, state) = world.get_block_and_block_state(block_pos).await.unwrap();

    let mut fence_gate_props = FenceGateBlockProps::from_state_id(state.id, &block).unwrap();
    fence_gate_props.open = fence_gate_props.open.flip();
    world
        .set_block_state(block_pos, fence_gate_props.to_state_id(&block))
        .await;

    fence_gate_props.to_state_id(&block)
}

// Macro to easily define new fence block variants
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
                _world: &World,
                block: &Block,
                _face: &BlockDirection,
                _block_pos: &BlockPos,
                _use_item_on: &SUseItemOn,
                player_direction: &CardinalDirection,
                _other: bool,
            ) -> u16 {
                let mut fence_gate_props = FenceGateBlockProps::default(block);
                fence_gate_props.facing = *player_direction;
                fence_gate_props.to_state_id(block)
            }

            async fn use_with_item(
                &self,
                _block: &Block,
                _player: &Player,
                location: BlockPos,
                _item: &Item,
                _server: &Server,
                world: &World,
            ) -> BlockActionResult {
                toggle_fence_gate(world, &location).await;
                BlockActionResult::Consume
            }

            async fn normal_use(
                &self,
                _block: &Block,
                _player: &Player,
                location: BlockPos,
                _server: &Server,
                world: &World,
            ) {
                toggle_fence_gate(world, &location).await;
            }
        }
    };
}

define_fence_block!(OakFenceGateBlock, Block::OAK_FENCE_GATE);
define_fence_block!(DarkOakFenceGateBlock, Block::DARK_OAK_FENCE_GATE);
define_fence_block!(SpruceFenceGateBlock, Block::SPRUCE_FENCE_GATE);
define_fence_block!(BirchFenceGateBlock, Block::BIRCH_FENCE_GATE);
define_fence_block!(JungleFenceGateBlock, Block::JUNGLE_FENCE_GATE);
define_fence_block!(AcaciaFenceGateBlock, Block::ACACIA_FENCE_GATE);
define_fence_block!(PaleOakFenceGateBlock, Block::PALE_OAK_FENCE_GATE);
define_fence_block!(CherryFenceGateBlock, Block::CHERRY_FENCE_GATE);
define_fence_block!(MangroveFenceGateBlock, Block::MANGROVE_FENCE_GATE);
define_fence_block!(CrimsonFenceGateBlock, Block::CRIMSON_FENCE_GATE);
define_fence_block!(WarpedFenceGateBlock, Block::WARPED_FENCE_GATE);
define_fence_block!(BambooFenceGateBlock, Block::BAMBOO_FENCE_GATE);

pub fn register_fence_gate_blocks(manager: &mut BlockRegistry) {
    manager.register(OakFenceGateBlock);
    manager.register(DarkOakFenceGateBlock);
    manager.register(SpruceFenceGateBlock);
    manager.register(BirchFenceGateBlock);
    manager.register(JungleFenceGateBlock);
    manager.register(AcaciaFenceGateBlock);
    manager.register(PaleOakFenceGateBlock);
    manager.register(CherryFenceGateBlock);
    manager.register(MangroveFenceGateBlock);
    manager.register(CrimsonFenceGateBlock);
    manager.register(WarpedFenceGateBlock);
    manager.register(BambooFenceGateBlock);
}
