use std::sync::Arc;

use crate::block::BlockFuture;
use crate::block::GetStateForNeighborUpdateArgs;
use crate::block::NormalUseArgs;
use crate::block::OnPlaceArgs;
use crate::entity::player::Player;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::tag;
use pumpkin_data::tag::RegistryKey;
use pumpkin_data::tag::Taggable;
use pumpkin_data::tag::get_tag_values;
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::world::BlockFlags;

use crate::block::BlockBehaviour;
use crate::block::registry::BlockActionResult;
use crate::world::World;

type FenceGateProperties = pumpkin_data::block_properties::OakFenceGateLikeProperties;

pub async fn toggle_fence_gate(
    world: &Arc<World>,
    block_pos: &BlockPos,
    player: &Player,
) -> BlockStateId {
    let (block, state) = world.get_block_and_state_id(block_pos).await;

    let mut fence_gate_props = FenceGateProperties::from_state_id(state, block);
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

#[pumpkin_block_from_tag("minecraft:fence_gates")]
pub struct FenceGateBlock;

impl BlockBehaviour for FenceGateBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut fence_gate_props = FenceGateProperties::default(args.block);
            fence_gate_props.facing = args.player.living_entity.entity.get_horizontal_facing();
            fence_gate_props.to_state_id(args.block)
        })
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let fence_props = is_in_wall(&args).await;
            fence_props.to_state_id(args.block)
        })
    }

    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            toggle_fence_gate(args.world, args.position, args.player).await;

            BlockActionResult::Success
        })
    }
}

async fn is_in_wall(args: &GetStateForNeighborUpdateArgs<'_>) -> FenceGateProperties {
    let mut fence_props = FenceGateProperties::from_state_id(args.state_id, args.block);

    let side_offset_left = args
        .position
        .offset(fence_props.facing.rotate_clockwise().to_offset());

    let side_offset_right = args
        .position
        .offset(fence_props.facing.rotate_counter_clockwise().to_offset());

    let neighbor_on_side =
        args.neighbor_position == &side_offset_left || args.neighbor_position == &side_offset_right;

    if neighbor_on_side {
        let neighbor_right = args.world.get_block(&side_offset_right).await;
        let neighbor_left = args.world.get_block(&side_offset_left).await;

        fence_props.in_wall = neighbor_left.has_tag(&tag::Block::MINECRAFT_WALLS)
            || neighbor_right.has_tag(&tag::Block::MINECRAFT_WALLS);
    }

    fence_props
}
