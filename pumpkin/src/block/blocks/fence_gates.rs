use std::sync::Arc;

use crate::block::BlockIsReplacing;
use crate::entity::player::Player;
use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::tag::RegistryKey;
use pumpkin_data::tag::get_tag_values;
use pumpkin_protocol::java::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::world::BlockFlags;

use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::block::registry::BlockActionResult;
use crate::server::Server;
use crate::world::World;
use pumpkin_data::item::Item;

type FenceGateProperties = pumpkin_data::block_properties::OakFenceGateLikeProperties;

pub async fn toggle_fence_gate(
    world: &Arc<World>,
    block_pos: &BlockPos,
    player: &Player,
) -> BlockStateId {
    let (block, state) = world.get_block_and_block_state(block_pos).await;

    let mut fence_gate_props = FenceGateProperties::from_state_id(state.id, &block);
    if fence_gate_props.open {
        fence_gate_props.open = false;
    } else {
        if fence_gate_props.facing
            == player
                .living_entity
                .entity
                .get_horizontal_facing()
                .opposite()
        {
            fence_gate_props.facing = player.living_entity.entity.get_horizontal_facing();
        }
        fence_gate_props.open = true;
    }
    world
        .set_block_state(
            block_pos,
            fence_gate_props.to_state_id(&block),
            BlockFlags::NOTIFY_LISTENERS,
        )
        .await;
    // TODO playSound depend on WoodType
    fence_gate_props.to_state_id(&block)
}

pub struct FenceGateBlock;
impl BlockMetadata for FenceGateBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        get_tag_values(RegistryKey::Block, "c:fence_gates").unwrap()
    }
}

#[async_trait]
impl PumpkinBlock for FenceGateBlock {
    async fn on_place(
        &self,
        _server: &Server,
        _world: &World,
        player: &Player,
        block: &Block,
        _block_pos: &BlockPos,
        _face: BlockDirection,
        _replacing: BlockIsReplacing,
        _use_item_on: &SUseItemOn,
    ) -> BlockStateId {
        let mut fence_gate_props = FenceGateProperties::default(block);
        fence_gate_props.facing = player.living_entity.entity.get_horizontal_facing();
        fence_gate_props.to_state_id(block)
    }

    async fn use_with_item(
        &self,
        _block: &Block,
        player: &Player,
        location: BlockPos,
        _item: &Item,
        _server: &Server,
        world: &Arc<World>,
    ) -> BlockActionResult {
        toggle_fence_gate(world, &location, player).await;
        BlockActionResult::Consume
    }

    async fn normal_use(
        &self,
        _block: &Block,
        player: &Player,
        location: BlockPos,
        _server: &Server,
        world: &Arc<World>,
    ) {
        toggle_fence_gate(world, &location, player).await;
    }
}
