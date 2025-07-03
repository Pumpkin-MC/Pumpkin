use std::sync::Arc;

use crate::block::pumpkin_block::NormalUseArgs;
use crate::block::pumpkin_block::OnPlaceArgs;
use crate::block::pumpkin_block::UseWithItemArgs;
use crate::entity::player::Player;
use async_trait::async_trait;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::tag::RegistryKey;
use pumpkin_data::tag::get_tag_values;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::world::BlockFlags;

use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::block::registry::BlockActionResult;
use crate::world::World;

type FenceGateProperties = pumpkin_data::block_properties::OakFenceGateLikeProperties;

pub async fn toggle_fence_gate(
    world: &Arc<World>,
    block_pos: &BlockPos,
    player: &Player,
) -> BlockStateId {
    let (block, state) = world.get_block_and_block_state(block_pos).await;

    let mut fence_gate_props = FenceGateProperties::from_state_id(state.id, block);
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
            fence_gate_props.to_state_id(block),
            BlockFlags::NOTIFY_LISTENERS,
        )
        .await;
    // TODO playSound depend on WoodType
    fence_gate_props.to_state_id(block)
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
    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut fence_gate_props = FenceGateProperties::default(args.block);
        fence_gate_props.facing = args.player.living_entity.entity.get_horizontal_facing();
        fence_gate_props.to_state_id(args.block)
    }

    async fn use_with_item(&self, args: UseWithItemArgs<'_>) -> BlockActionResult {
        toggle_fence_gate(args.world, args.location, args.player).await;
        BlockActionResult::Consume
    }

    async fn normal_use(&self, args: NormalUseArgs<'_>) {
        toggle_fence_gate(args.world, args.location, args.player).await;
    }
}
