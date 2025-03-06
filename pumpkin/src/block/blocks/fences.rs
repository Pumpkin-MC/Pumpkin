use async_trait::async_trait;
use pumpkin_data::block::Block;
use pumpkin_data::block::CardinalDirection;
use pumpkin_data::block::{BlockProperties, Boolean};
use pumpkin_macros::pumpkin_block;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::BlockDirection;

use crate::block::pumpkin_block::PumpkinBlock;
use crate::block::registry::BlockRegistry;
use crate::server::Server;
use crate::world::World;

const FENCE_BLOCKS_IDS: [u16; 13] = [
    Block::OAK_FENCE.id,
    Block::DARK_OAK_FENCE.id,
    Block::SPRUCE_FENCE.id,
    Block::BIRCH_FENCE.id,
    Block::JUNGLE_FENCE.id,
    Block::ACACIA_FENCE.id,
    Block::PALE_OAK_FENCE.id,
    Block::CHERRY_FENCE.id,
    Block::MANGROVE_FENCE.id,
    Block::NETHER_BRICK_FENCE.id,
    Block::CRIMSON_FENCE.id,
    Block::WARPED_FENCE.id,
    Block::BAMBOO_FENCE.id,
];

fn connects_to(from: &Block, to: &Block) -> bool {
    if from.id == to.id {
        return true;
    }

    if from.id == Block::NETHER_BRICK_FENCE.id || to.id == Block::NETHER_BRICK_FENCE.id {
        return false;
    }

    FENCE_BLOCKS_IDS.contains(&to.id)
}

pub async fn fence_state<T: BlockProperties + FenceProperties>(
    world: &World,
    block: &Block,
    block_pos: &BlockPos,
    default_state_id: u16,
) -> u16 {
    let mut block_properties = T::from_state_id(default_state_id).unwrap();

    for direction in BlockDirection::horizontal() {
        let offset = block_pos.offset(direction.to_offset());
        let other_block = world.get_block(&offset).await.unwrap_or(Block::AIR);

        if connects_to(block, &other_block) {
            block_properties.set_connection(direction, true);
        }
    }

    block_properties.to_state_id()
}

pub trait FenceProperties: Send + Sync {
    fn set_connection(&mut self, direction: BlockDirection, connected: bool);
}

// Macro to easily define new fence block variants
macro_rules! define_fence_block {
    ($block_name:ident, $block_id:expr, $props_type:ty, $default_state_id:expr) => {
        #[pumpkin_block($block_id)]
        pub struct $block_name;

        impl FenceProperties for $props_type {
            fn set_connection(&mut self, direction: BlockDirection, connected: bool) {
                let value = if connected {
                    Boolean::True
                } else {
                    Boolean::False
                };
                match direction {
                    BlockDirection::North => self.north = value,
                    BlockDirection::South => self.south = value,
                    BlockDirection::West => self.west = value,
                    BlockDirection::East => self.east = value,
                    _ => {}
                }
            }
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
                fence_state::<$props_type>(world, block, block_pos, $default_state_id).await
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
                    .set_block_state(
                        block_pos,
                        fence_state::<$props_type>(world, block, block_pos, $default_state_id)
                            .await,
                    )
                    .await;
            }
        }
    };
}

define_fence_block!(
    OakFenceBlock,
    "minecraft:oak_fence",
    pumpkin_data::block::OakFenceProps,
    Block::OAK_FENCE.default_state_id
);

define_fence_block!(
    DarkOakFenceBlock,
    "minecraft:dark_oak_fence",
    pumpkin_data::block::DarkOakFenceProps,
    Block::DARK_OAK_FENCE.default_state_id
);

define_fence_block!(
    SpruceFenceBlock,
    "minecraft:spruce_fence",
    pumpkin_data::block::SpruceFenceProps,
    Block::SPRUCE_FENCE.default_state_id
);

define_fence_block!(
    BirchFenceBlock,
    "minecraft:birch_fence",
    pumpkin_data::block::BirchFenceProps,
    Block::BIRCH_FENCE.default_state_id
);

define_fence_block!(
    JungleFenceBlock,
    "minecraft:jungle_fence",
    pumpkin_data::block::JungleFenceProps,
    Block::JUNGLE_FENCE.default_state_id
);

define_fence_block!(
    AcaciaFenceBlock,
    "minecraft:acacia_fence",
    pumpkin_data::block::AcaciaFenceProps,
    Block::ACACIA_FENCE.default_state_id
);

define_fence_block!(
    PaleOakFenceBlock,
    "minecraft:pale_oak_fence",
    pumpkin_data::block::PaleOakFenceProps,
    Block::PALE_OAK_FENCE.default_state_id
);

define_fence_block!(
    CherryFenceBlock,
    "minecraft:cherry_fence",
    pumpkin_data::block::CherryFenceProps,
    Block::CHERRY_FENCE.default_state_id
);

define_fence_block!(
    MangroveFenceBlock,
    "minecraft:mangrove_fence",
    pumpkin_data::block::MangroveFenceProps,
    Block::MANGROVE_FENCE.default_state_id
);

define_fence_block!(
    CrimsonFenceBlock,
    "minecraft:crimson_fence",
    pumpkin_data::block::CrimsonFenceProps,
    Block::CRIMSON_FENCE.default_state_id
);

define_fence_block!(
    WarpedFenceBlock,
    "minecraft:warped_fence",
    pumpkin_data::block::WarpedFenceProps,
    Block::WARPED_FENCE.default_state_id
);

define_fence_block!(
    BambooFenceBlock,
    "minecraft:bamboo_fence",
    pumpkin_data::block::BambooFenceProps,
    Block::BAMBOO_FENCE.default_state_id
);

define_fence_block!(
    NetherBrickFenceBlock,
    "minecraft:nether_brick_fence",
    pumpkin_data::block::NetherBrickFenceProps,
    Block::NETHER_BRICK_FENCE.default_state_id
);

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
