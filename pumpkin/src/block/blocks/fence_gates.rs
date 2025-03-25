use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::block::Block;
use pumpkin_data::block::BlockProperties;
use pumpkin_data::block::HorizontalFacing;
use pumpkin_data::tag::RegistryKey;
use pumpkin_data::tag::get_tag_values;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::BlockDirection;

use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::block::registry::BlockActionResult;
use crate::block::registry::BlockRegistry;
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::BlockFlags;
use crate::world::World;
use pumpkin_data::item::Item;

type FenceGateProperties = pumpkin_data::block::OakFenceGateLikeProperties;

pub async fn toggle_fence_gate(world: &Arc<World>, block_pos: &BlockPos) -> u16 {
    let (block, state) = world.get_block_and_block_state(block_pos).await.unwrap();

    let mut fence_gate_props = FenceGateProperties::from_state_id(state.id, &block);
    fence_gate_props.open = fence_gate_props.open.flip();
    world
        .set_block_state(
            block_pos,
            fence_gate_props.to_state_id(&block),
            BlockFlags::NOTIFY_LISTENERS,
        )
        .await;

    fence_gate_props.to_state_id(&block)
}

pub fn register_fence_gate_blocks(manager: &mut BlockRegistry) {
    let tag_values: &'static [&'static str] =
        get_tag_values(RegistryKey::Block, "c:fence_gates").unwrap();

    for block in tag_values {
        pub struct FenceGateBlock {
            id: &'static str,
        }
        impl BlockMetadata for FenceGateBlock {
            fn namespace(&self) -> &'static str {
                "minecraft"
            }

            fn id(&self) -> &'static str {
                self.id
            }
        }

        #[async_trait]
        impl PumpkinBlock for FenceGateBlock {
            async fn on_place(
                &self,
                _server: &Server,
                _world: &World,
                block: &Block,
                _face: &BlockDirection,
                _block_pos: &BlockPos,
                _use_item_on: &SUseItemOn,
                player_direction: &HorizontalFacing,
                _other: bool,
            ) -> u16 {
                let mut fence_gate_props = FenceGateProperties::default(block);
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
                world: &Arc<World>,
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
                world: &Arc<World>,
            ) {
                toggle_fence_gate(world, &location).await;
            }
        }

        manager.register(FenceGateBlock { id: block });
    }
}
