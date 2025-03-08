use async_trait::async_trait;
use pumpkin_data::block::Block;
use pumpkin_data::block::CardinalDirection;
use pumpkin_data::block::FenceBlockProps;
use pumpkin_data::block::{BlockProperties, Boolean};
use pumpkin_data::tag::RegistryKey;
use pumpkin_data::tag::Tagable;
use pumpkin_data::tag::get_tag_values;
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

pub fn register_fence_blocks(manager: &mut BlockRegistry) {
    let tag_values: &'static [&'static str] =
        get_tag_values(RegistryKey::Block, "c:fences").unwrap();

    for block in tag_values {
        pub struct FenceBlock {
            id: &'static str,
        }
        impl BlockMetadata for FenceBlock {
            fn namespace(&self) -> &'static str {
                "minecraft"
            }

            fn id(&self) -> &'static str {
                self.id
            }
        }

        #[async_trait]
        impl PumpkinBlock for FenceBlock {
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

        manager.register(FenceBlock { id: block });
    }
}
